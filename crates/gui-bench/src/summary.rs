//! A run as text: one Markdown table of medians per measurement mode, a
//! row for each scenario and framework. The HTML report shows one scenario
//! at a time; this is the whole matrix at once, for reading in a terminal
//! and pasting into notes.

use bench_core::{
    ConditionResult, Framework, MeasurementMode, PipelineSample, RunDocument, RunStatus,
    SamplePhase,
};
use std::collections::BTreeMap;
use std::fmt::Write;

/// The same percentile the report takes: the sample at `p` of the way
/// through the sorted values.
fn percentile(mut values: Vec<f64>, p: f64) -> Option<f64> {
    values.retain(|v| v.is_finite());
    values.sort_by(f64::total_cmp);
    let last = values.len().checked_sub(1)?;
    Some(values[((last as f64) * p).floor() as usize])
}

fn ms(v: Option<f64>) -> String {
    match v {
        None => "—".into(),
        Some(ns) => {
            let ms = ns / 1e6;
            if ms >= 100.0 {
                format!("{ms:.0}")
            } else if ms >= 10.0 {
                format!("{ms:.1}")
            } else if ms >= 1.0 {
                format!("{ms:.2}")
            } else {
                format!("{ms:.3}")
            }
        }
    }
}

type Stage = (&'static str, fn(&PipelineSample) -> Option<u64>);

/// The pipeline's stages as the table groups them: the four GPU submission
/// stages together, since a framework has either them or a rasterizer.
const STAGES: [Stage; 8] = [
    ("view", |p| p.view_build_ns),
    ("style", |p| p.style_ns),
    ("layout", |p| p.layout_ns),
    ("prepaint", |p| p.prepaint_ns),
    ("paint", |p| p.paint_ns),
    ("raster", |p| p.rasterize_ns),
    ("gpu", |p| {
        let parts = [p.gpu_prepare_ns, p.command_encode_ns, p.queue_submit_ns];
        parts
            .iter()
            .any(Option::is_some)
            .then(|| parts.iter().flatten().sum())
    }),
    ("present", |p| p.present_ns),
];

/// What a frame's stages come to. A frame with no stage measured has no
/// total.
fn total(p: &PipelineSample) -> Option<u64> {
    let parts: Vec<u64> = STAGES.iter().filter_map(|(_, stage)| stage(p)).collect();
    (!parts.is_empty()).then(|| parts.iter().sum())
}

pub fn render(doc: &RunDocument) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# {}", doc.run_id);
    let _ = writeln!(out);
    let _ = writeln!(out, "- CPU: {}", doc.host.cpu);
    for gpu in &doc.host.gpu {
        let _ = writeln!(out, "- GPU: {gpu}");
    }
    let _ = writeln!(
        out,
        "- Display server: {}; kernel {}",
        doc.host.display_server, doc.host.kernel
    );
    for (name, source) in &doc.sources {
        let revision = source
            .revision
            .as_deref()
            .map_or("unknown", |r| &r[..r.len().min(10)]);
        let dirty = if source.dirty == Some(true) {
            ", dirty"
        } else {
            ""
        };
        let _ = writeln!(out, "- {name}: {revision}{dirty}");
    }
    for mode in [MeasurementMode::Headline, MeasurementMode::Diagnostic] {
        // Scenarios in the order they ran, frameworks in theirs.
        let mut order: Vec<&str> = Vec::new();
        let mut groups: BTreeMap<(usize, Framework), Vec<&ConditionResult>> = BTreeMap::new();
        for c in doc.conditions.iter().filter(|c| c.mode == mode) {
            let at = order
                .iter()
                .position(|id| *id == c.scenario.id)
                .unwrap_or_else(|| {
                    order.push(&c.scenario.id);
                    order.len() - 1
                });
            groups.entry((at, c.framework)).or_default().push(c);
        }
        if groups.is_empty() {
            continue;
        }
        let diagnostic = mode == MeasurementMode::Diagnostic;
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "## {} passes",
            if diagnostic { "Diagnostic" } else { "Headline" }
        );
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "Milliseconds, over the measurement frames of every completed repetition. \"First\" is the first \
             frame's time from the adapter's start; PSS is the peak, in MiB."
        );
        if diagnostic {
            let _ = writeln!(
                out,
                "Stages are medians of CPU time; \"work\" is the median of each frame's stages added up, \
                 and \"gpu\" is prepare, encode and submit together."
            );
        }
        let _ = writeln!(out);
        let mut head = "| scenario | framework | runs | frame p50 | frame p95 | latency p50 | latency p95 | first | PSS |".to_owned();
        let mut rule = "|---|---|---|---|---|---|---|---|---|".to_owned();
        if diagnostic {
            for (name, _) in STAGES {
                let _ = write!(head, " {name} |");
                rule.push_str("---|");
            }
            head.push_str(" work |");
            rule.push_str("---|");
        }
        let _ = writeln!(out, "{head}");
        let _ = writeln!(out, "{rule}");
        for ((at, framework), conditions) in &groups {
            let complete = conditions
                .iter()
                .filter(|c| matches!(c.status, RunStatus::Complete))
                .count();
            let measured = || {
                conditions
                    .iter()
                    .flat_map(|c| c.frames.iter())
                    .filter(|f| f.phase == SamplePhase::Measurement && f.valid)
            };
            let frame: Vec<f64> = measured()
                .filter_map(|f| f.frame_time_ns)
                .map(|v| v as f64)
                .collect();
            let latency: Vec<f64> = measured()
                .filter_map(|f| f.stimulus_latency_ns)
                .map(|v| v as f64)
                .collect();
            let first: Vec<f64> = conditions
                .iter()
                .filter_map(|c| c.frames.iter().find(|f| f.phase == SamplePhase::Startup))
                .map(|f| f.timestamp_ns as f64)
                .collect();
            let pss = conditions
                .iter()
                .flat_map(|c| c.resources.iter())
                .filter_map(|r| r.pss_bytes)
                .max()
                .map_or("—".to_owned(), |b| {
                    format!("{:.0}", b as f64 / 1_048_576.0)
                });
            let mut row = format!(
                "| {} | {} | {complete}/{} | {} | {} | {} | {} | {} | {pss} |",
                order[*at],
                framework.as_str(),
                conditions.len(),
                ms(percentile(frame.clone(), 0.5)),
                ms(percentile(frame, 0.95)),
                ms(percentile(latency.clone(), 0.5)),
                ms(percentile(latency, 0.95)),
                ms(percentile(first, 0.5)),
            );
            if diagnostic {
                for (_, stage) in STAGES {
                    let values: Vec<f64> = measured()
                        .filter_map(|f| stage(&f.pipeline))
                        .map(|v| v as f64)
                        .collect();
                    let _ = write!(row, " {} |", ms(percentile(values, 0.5)));
                }
                let totals: Vec<f64> = measured()
                    .filter_map(|f| total(&f.pipeline))
                    .map(|v| v as f64)
                    .collect();
                let _ = write!(row, " {} |", ms(percentile(totals, 0.5)));
            }
            let _ = writeln!(out, "{row}");
        }
    }
    let mut warnings: BTreeMap<String, usize> = BTreeMap::new();
    for c in &doc.conditions {
        for w in &c.warnings {
            *warnings
                .entry(format!("{}: {w}", c.framework.as_str()))
                .or_default() += 1;
        }
        match &c.status {
            RunStatus::Complete => {}
            RunStatus::TimedOut => {
                *warnings
                    .entry(format!(
                        "{}: {} timed out",
                        c.framework.as_str(),
                        c.scenario.id
                    ))
                    .or_default() += 1;
            }
            RunStatus::Failed { message } => {
                let line = format!(
                    "{}: {} failed: {message}",
                    c.framework.as_str(),
                    c.scenario.id
                );
                *warnings.entry(line).or_default() += 1;
            }
        }
    }
    if !warnings.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "## Warnings");
        let _ = writeln!(out);
        for (warning, count) in warnings {
            let _ = writeln!(out, "- {warning} (×{count})");
        }
    }
    out
}
