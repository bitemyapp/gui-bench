//! The aizome-kit adapter: the neutral scenario as a kit program.
//!
//! The kit is Elm's architecture on a CPU rasterizer. A message runs
//! `update`, then `view` builds the whole window's elements again, the kit
//! lays them out, compares them with the last tree by key path, and
//! invalidates the rectangles that differ. Drawing records operations on
//! the program's thread; Sidestep's render thread rasterizes the damaged
//! rectangles and commits them, one frame in flight at a time.
//!
//! So a frame here is a display pass: the kit draws only once the frame
//! before has shown, which makes the start of a pass the kit's frame tick,
//! as `on_next_frame` is GPUI's. The window's root element is a widget of
//! this adapter's ([`FrameTick`]) that tells the program each time the
//! window draws; the program answers with the next mutation, as a message,
//! the way an app would.

use std::cell::Cell;
use std::io::BufRead;
use std::rc::Rc;
use std::time::{Duration, Instant};

use aizome_kit::hash::PaintHasher;
use aizome_kit::layout::{Constraints, Fill, Padding, Point, Rect, Size};
use aizome_kit::prelude::*;
use aizome_kit::sidestep_ui::kurbo::RoundedRect;
use aizome_kit::sidestep_ui::{Shadow, StrokeStyle};
use aizome_kit::widget::{LayoutCx, PaintCx, Widget};
use aizome_kit::widgets::layout::column;
use aizome_kit::{MsgSender, Vec2};
use anyhow::{Context, Result};
use bench_core::{
    AdapterEvent, FrameSample, Framework, MeasurementMode, PipelineSample, SamplePhase, Scenario,
    StyleComplexity, Topology, Workload,
};
use clap::{Parser, ValueEnum};
use sidestep_engine::stats as frames;

// The colours the other adapters use.
const GROUND: Color = Color::hex(0x0b1020);
const ITEM: Color = Color::hex(0x18243a);
const CHANGED: Color = Color::hex(0x357d74);
const TEXT: Color = Color::hex(0xe8eefc);
const BORDER: Color = Color::hex(0x3b4d6b);

#[derive(Parser)]
struct Args {
    #[arg(long)]
    scenario: String,
    #[arg(long, value_enum)]
    mode: ModeArg,
}

#[derive(Clone, Copy, ValueEnum)]
enum ModeArg {
    Headline,
    Diagnostic,
}

impl From<ModeArg> for MeasurementMode {
    fn from(v: ModeArg) -> Self {
        match v {
            ModeArg::Headline => Self::Headline,
            ModeArg::Diagnostic => Self::Diagnostic,
        }
    }
}

#[derive(Clone, Debug)]
enum Msg {
    /// The window began drawing a frame, at this instant.
    Frame(Instant),
}

/// What the window's root tells the program.
#[derive(Default)]
struct Shared {
    /// The window's size in points, as last laid out.
    size: Cell<Option<Size>>,
    /// How long laying the content out took, the last time.
    layout_ns: Cell<u64>,
}

/// The window's root. It lays its one child out in all the room, and does
/// two things for the benchmark with the kit's own means:
///
/// - each time the window draws, it sends [`Msg::Frame`]. Ancestors of
///   anything damaged are painted, so the root is painted in every pass.
/// - each time the window is built, it invalidates one point of itself
///   (`LayoutCx::invalidate_rect`), so that every build is followed by a
///   pass even when what changed is out of sight and the kit would draw
///   nothing. Passive and resize workloads invalidate all of it instead:
///   the redraw GPUI's animation frame and zgui's `request_redraw` ask
///   for.
struct FrameTick {
    frames: MsgSender<Msg>,
    shared: Rc<Shared>,
    redraw_all: bool,
}

impl Widget for FrameTick {
    fn name(&self) -> &'static str {
        "frame-tick"
    }

    fn layout(&mut self, cx: &mut LayoutCx, bc: Constraints) -> Size {
        let started = Instant::now();
        let size = cx.layout_child(0, bc);
        self.shared
            .layout_ns
            .set(started.elapsed().as_nanos() as u64);
        cx.place_child(0, Point::ZERO);
        self.shared.size.set(Some(size));
        let probe = if self.redraw_all {
            Rect::from_origin_size(Point::ZERO, size)
        } else {
            Rect::new(0.0, 0.0, 1.0, 1.0)
        };
        cx.invalidate_rect(probe);
        size
    }

    fn paint(&self, _cx: &mut PaintCx) {
        self.frames.send(Msg::Frame(Instant::now()));
    }
}

/// The window's content, as tall as it comes: what CSS's `overflow: hidden`
/// does for a column whose children keep their heights. A kit column gives
/// a child that doesn't fit the room that is left, which past the window's
/// bottom is none; without this the items out of sight are laid out with
/// no height, where the other adapters' keep theirs, and a decorated one
/// still draws its shadow there. (An app would put a column this long in
/// the kit's `scroll`, which lays its content out the same way.)
struct Tall;

impl Widget for Tall {
    fn name(&self) -> &'static str {
        "tall"
    }

    fn layout(&mut self, cx: &mut LayoutCx, bc: Constraints) -> Size {
        cx.layout_child(
            0,
            Constraints::loose(Size::new(bc.max.width, f64::INFINITY)),
        );
        cx.place_child(0, Point::ZERO);
        bc.max
    }
}

/// Children in rows, left to right, wrapping at the width given, `gap`
/// apart: CSS's `flex-wrap`, which the kit's rows don't have.
struct Wrap {
    gap: f64,
}

impl Widget for Wrap {
    fn name(&self) -> &'static str {
        "wrap"
    }

    fn layout(&mut self, cx: &mut LayoutCx, bc: Constraints) -> Size {
        let room = bc.max.width;
        let (mut x, mut y, mut row, mut width) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        for i in 0..cx.child_count() {
            let s = cx.layout_child(i, Constraints::loose(Size::new(room, f64::INFINITY)));
            if x > 0.0 && x + s.width > room {
                x = 0.0;
                y += row + self.gap;
                row = 0.0;
            }
            cx.place_child(i, Point::new(x, y));
            x += s.width;
            width = width.max(x);
            x += self.gap;
            row = row.max(s.height);
        }
        bc.constrain(Size::new(width, y + row))
    }
}

/// The decorated item's box: a fill with rounded corners, a 1pt border and
/// a soft shadow, as zgui's style sheet gives it. The kit's own container
/// is square and its shadow hard, so this draws with Sidestep's canvas, as
/// a kit widget does for anything outside the kit's look.
struct Card {
    fill: Color,
    padding: Padding,
}

impl Widget for Card {
    fn name(&self) -> &'static str {
        "card"
    }

    fn layout(&mut self, cx: &mut LayoutCx, bc: Constraints) -> Size {
        let p = self.padding;
        let child = cx.layout_child(0, bc.shrink(p).loosen());
        cx.place_child(0, Point::new(p.left, p.top));
        bc.constrain(Size::new(
            child.width + p.horizontal(),
            child.height + p.vertical(),
        ))
    }

    fn paint(&self, cx: &mut PaintCx) {
        let r = cx.bounds();
        cx.canvas.set_shadow(Some(Shadow {
            offset: Vec2::new(0.0, 2.0),
            blur: 4.0,
            color: Color::rgba(0.0, 0.0, 0.0, 0.35),
        }));
        cx.canvas.fill(&RoundedRect::from_rect(r, 4.0), self.fill);
        cx.canvas.set_shadow(None);
        cx.canvas.stroke(
            &RoundedRect::from_rect(r.inset(-0.5), 3.5),
            &StrokeStyle::new(1.0),
            BORDER,
        );
    }

    fn hash(&self, h: &mut PaintHasher) {
        h.color(self.fill);
    }

    fn overflow(&self) -> f64 {
        // The shadow: its blur, and its offset down.
        6.0
    }
}

struct Bench {
    spec: Scenario,
    mode: MeasurementMode,
    epoch: Instant,
    frames: Option<MsgSender<Msg>>,
    shared: Rc<Shared>,
    window: WindowKey,

    // The model, as the zgui adapter keeps it: each node holds the tick
    // that last affected it, and shows its changed state while that tick
    // is odd. A retained toolkit draws what differs, so the model has to
    // differ from one tick to the next for there to be anything to
    // measure; GPUI's adapter, which rebuilds everything regardless, can
    // leave a node "changed" for many ticks running.
    /// The tick of the last mutation.
    tick: u64,
    /// The tick that last affected each node (0: none yet).
    ticks: Vec<u64>,
    /// Each node's children in the balanced topology.
    children: Vec<Vec<usize>>,

    // Sampling.
    sequence: u64,
    last_frame: Option<Instant>,
    stimulus: Option<Instant>,
    /// How long the last `view` took.
    view_ns: Cell<u64>,
    /// When the last `view` returned, until the kit's build of it is over.
    view_ended: Cell<Option<Instant>>,
    /// How long the kit took to build what the last `view` returned.
    build_ns: Cell<u64>,
    /// The frame before this one, waiting for the render thread's part of it.
    pending: Option<FrameSample>,
    /// Frames spent asking for the scenario's window size before the run.
    settling: u8,
    done: bool,
}

impl Bench {
    fn new(spec: Scenario, mode: MeasurementMode, epoch: Instant) -> Bench {
        let mut children = Vec::new();
        if spec.topology == Topology::Balanced {
            children = vec![Vec::new(); spec.nodes];
            for id in 1..spec.nodes {
                children[(id - 1) / 8].push(id);
            }
        }
        Bench {
            ticks: vec![0; spec.nodes],
            spec,
            mode,
            epoch,
            frames: None,
            shared: Rc::default(),
            window: WindowKey::NONE,
            tick: 0,
            children,
            sequence: 0,
            last_frame: None,
            stimulus: None,
            view_ns: Cell::new(0),
            view_ended: Cell::new(None),
            build_ns: Cell::new(0),
            pending: None,
            settling: 0,
            done: false,
        }
    }

    fn passive(&self) -> bool {
        matches!(self.spec.workload, Workload::Idle | Workload::ColdStart)
    }

    fn resizing(&self) -> bool {
        matches!(
            self.spec.workload,
            Workload::ResizeStep | Workload::ResizeStorm
        )
    }

    /// The mutation of tick `tick`.
    fn stimulate(&mut self, tick: u64) {
        self.tick = tick;
        // A structural update shows or hides the tail by the tick alone.
        if self.spec.workload != Workload::StructuralUpdate {
            for index in bench_core::affected_indices(&self.spec, tick) {
                self.ticks[index] = tick;
            }
        }
    }

    fn item(&self, id: usize, children: Vec<Element<Msg>>) -> Element<Msg> {
        let (workload, tick) = (self.spec.workload, self.ticks[id]);
        let changed = tick % 2 == 1;
        let fill = match workload {
            Workload::PaintUpdate | Workload::InputLatency if changed => CHANGED,
            _ => ITEM,
        };
        // zgui's `.item.layout-changed`.
        let (left, min_width) = match workload {
            Workload::LayoutUpdate if changed => (12.0, 38.0),
            _ => (4.0, 18.0),
        };
        let label = match workload {
            Workload::TextUpdate if tick > 0 => format!("item {id:05} · {tick:04}"),
            _ => format!("item {id:05}"),
        };
        let label = text(label).size(12.0).color(TEXT);
        let body: Element<Msg> = if children.is_empty() {
            label.into()
        } else {
            let mut all = Vec::with_capacity(children.len() + 1);
            all.push(label.into());
            all.extend(children);
            column(all).into()
        };
        let padding = Padding {
            top: 0.0,
            right: 4.0,
            bottom: 0.0,
            left,
        };
        let item: Element<Msg> = match self.spec.style {
            StyleComplexity::Minimal => container(body)
                .padding(padding)
                .fill(Tone::Color(fill))
                .into(),
            StyleComplexity::Decorated => Element::new(Card { fill, padding }, vec![body]),
        };
        item.min_width(min_width).min_height(16.0)
    }

    fn balanced(&self, id: usize, visible: usize) -> Element<Msg> {
        let nested = self.children[id]
            .iter()
            .filter(|&&c| c < visible)
            .map(|&c| self.balanced(c, visible))
            .collect();
        self.item(id, nested)
    }

    fn tree(&self) -> Element<Msg> {
        let nodes = self.spec.nodes;
        let visible = if self.spec.workload == Workload::StructuralUpdate && self.tick % 2 == 1 {
            nodes.saturating_sub((nodes / 10).max(1))
        } else {
            nodes
        };
        match self.spec.topology {
            Topology::Flat => Element::new(
                Wrap { gap: 4.0 },
                (0..visible).map(|id| self.item(id, Vec::new())).collect(),
            ),
            Topology::Wide => {
                if visible == 0 {
                    return space(0.0);
                }
                self.item(
                    0,
                    (1..visible).map(|id| self.item(id, Vec::new())).collect(),
                )
            }
            Topology::Balanced => {
                if visible == 0 {
                    return space(0.0);
                }
                self.balanced(0, visible)
            }
            Topology::Deep => {
                let mut current = self.item(visible.saturating_sub(1), Vec::new());
                for id in (0..visible.saturating_sub(1)).rev() {
                    current = self.item(id, vec![current]);
                }
                current
            }
        }
    }

    fn phase(&self, sequence: u64) -> SamplePhase {
        if sequence == 0 {
            SamplePhase::Startup
        } else if sequence <= u64::from(self.spec.warmup_frames) {
            SamplePhase::Warmup
        } else {
            SamplePhase::Measurement
        }
    }

    /// What the build of the frame that is drawing now took: `view`, the
    /// layout of what it returned, and the rest of the kit's build under
    /// "prepaint" (mounting the elements, placing and hashing every node,
    /// the comparison with the last tree, the invalidation).
    fn built(&self) -> PipelineSample {
        let layout = self.shared.layout_ns.get();
        PipelineSample {
            view_build_ns: Some(self.view_ns.get()),
            layout_ns: Some(layout),
            prepaint_ns: Some(self.build_ns.get().saturating_sub(layout)),
            ..PipelineSample::default()
        }
    }

    /// A display pass began at `at`: sample it, and send the next mutation.
    fn frame(&mut self, at: Instant, cx: &mut Cx<Msg>) {
        if self.done {
            // The frame after a resize run put the window back at the
            // scenario's size: the size the compositor remembers it by.
            cx.quit();
            return;
        }
        // A compositor that remembers where a program's windows were opens
        // this one at the size its last run left. Aizome does, by app id,
        // and takes the binary's name for a program that gives none. Ask for
        // the scenario's size, and begin once the window has it.
        let want = Size::new(f64::from(self.spec.width), f64::from(self.spec.height));
        let sized =
            |s: Size| (s.width - want.width).abs() < 0.5 && (s.height - want.height).abs() < 0.5;
        if self.sequence == 0
            && self.settling < 60
            && self.shared.size.get().is_some_and(|s| !sized(s))
        {
            self.settling += 1;
            cx.request_size(self.window, want);
            return;
        }
        let diagnostic = self.mode == MeasurementMode::Diagnostic;
        // The frame before this one has shown, so the render thread has
        // measured it: its sample is whole now.
        if let Some(mut before) = self.pending.take() {
            if diagnostic
                && let Some(f) = frames::latest()
                    .into_iter()
                    .find(|f| f.window != frames::OFFSCREEN)
            {
                // Sidestep keeps microseconds; a summary of one frame's
                // median is that frame.
                let ns = |micros: u32| Some(u64::from(micros) * 1_000);
                before.pipeline.paint_ns = f.record.and_then(|t| ns(t.p50));
                before.pipeline.rasterize_ns = ns(f.rasterize.p50);
                before.pipeline.present_ns = ns(f.present.p50);
            }
            let _ = bench_core::emit_event(&AdapterEvent::Frame { sample: before });
        }
        let sequence = self.sequence;
        let size = self.shared.size.get();
        let sample = FrameSample {
            sequence,
            phase: self.phase(sequence),
            timestamp_ns: at.duration_since(self.epoch).as_nanos() as u64,
            frame_time_ns: self
                .last_frame
                .replace(at)
                .map(|v| at.duration_since(v).as_nanos() as u64),
            stimulus_latency_ns: self
                .stimulus
                .take()
                .map(|v| at.saturating_duration_since(v).as_nanos() as u64),
            observed_width: size.map(|s| s.width.round() as u32),
            observed_height: size.map(|s| s.height.round() as u32),
            pipeline: if diagnostic {
                self.built()
            } else {
                PipelineSample::default()
            },
            framework_memory: None,
            valid: true,
            note: None,
        };
        let total = u64::from(self.spec.warmup_frames) + u64::from(self.spec.sample_frames);
        if sequence >= total {
            // The last frame goes without the render thread's part: the
            // run is over before it shows.
            let _ = bench_core::emit_event(&AdapterEvent::Frame { sample });
            let _ = bench_core::emit_event(&AdapterEvent::Complete {
                frames: sequence + 1,
            });
            self.done = true;
            if self.resizing() {
                cx.request_size(self.window, want);
            } else {
                cx.quit();
            }
            return;
        }
        self.pending = Some(sample);
        self.sequence = sequence + 1;
        if !self.passive() {
            self.stimulus = Some(Instant::now());
            self.stimulate(sequence + 1);
        }
        match self.spec.workload {
            Workload::ResizeStep => {
                let large = (sequence / 8) % 2 == 0;
                let (w, h) = if large {
                    (1_200.0, 800.0)
                } else {
                    (900.0, 620.0)
                };
                cx.request_size(self.window, Size::new(w, h));
            }
            Workload::ResizeStorm => {
                let d = (sequence % 120) as f64;
                cx.request_size(self.window, Size::new(840.0 + d * 3.0, 560.0 + d * 2.0));
            }
            _ => {}
        }
    }
}

impl Program for Bench {
    type Msg = Msg;

    /// No app id, as the other adapters' windows have none.
    fn info(&self) -> AppInfo {
        AppInfo::new("", "aizome-kit benchmark", "Benchmark")
    }

    fn launched(&mut self, cx: &mut Cx<Msg>) {
        self.frames = Some(cx.stream());
        let spec = WindowSpec::new("aizome-kit benchmark")
            .size(f64::from(self.spec.width), f64::from(self.spec.height));
        self.window = cx.open_window(spec);
    }

    fn update(&mut self, msg: Msg, cx: &mut Cx<Msg>) {
        match msg {
            Msg::Frame(at) => self.frame(at, cx),
        }
    }

    fn view(&self, _window: WindowKey) -> Element<Msg> {
        let started = Instant::now();
        let ground = container(Element::new(Tall, vec![self.tree()]))
            .padding(8.0)
            .fill(Tone::Color(GROUND))
            .clip()
            .width(Fill)
            .height(Fill);
        let tick = FrameTick {
            frames: self
                .frames
                .clone()
                .expect("the window opens after the program launched"),
            shared: self.shared.clone(),
            redraw_all: self.passive() || self.resizing(),
        };
        let root = Element::new(tick, vec![ground.into()])
            .width(Fill)
            .height(Fill);
        let ended = Instant::now();
        self.view_ns
            .set(ended.duration_since(started).as_nanos() as u64);
        self.view_ended.set(Some(ended));
        root
    }

    /// The standard menus, as every kit program has. The kit asks for them
    /// again as soon as it has built a window's view (and after each
    /// update, when no view is waiting), and gives a program no other word
    /// that a build is over: so the time from `view` returning to here is
    /// the build's.
    fn menus(&self) -> MenuBar<Msg> {
        if let Some(ended) = self.view_ended.take() {
            self.build_ns.set(ended.elapsed().as_nanos() as u64);
        }
        MenuBar::standard(&self.info())
    }

    // The window is the benchmark's content and nothing else: no menu bar
    // drawn along its top, and none published to a shell's.
    fn menus_to_shell(&self) -> bool {
        false
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let spec: Scenario = serde_json::from_str(&args.scenario).context("invalid scenario JSON")?;
    spec.validate().map_err(anyhow::Error::msg)?;
    let mode: MeasurementMode = args.mode.into();
    bench_core::emit_event(&AdapterEvent::Hello {
        protocol_version: bench_core::PROTOCOL_VERSION,
        framework: Framework::AizomeKit,
        framework_version: "aizome-kit 0.1.0 (local path)".into(),
        mode,
        pid: std::process::id(),
        capabilities: vec![
            "frame_tick".into(),
            "stimulus_to_frame_latency".into(),
            "window_resize".into(),
            "pipeline_cpu_stages".into(),
            "cpu_rasterizer".into(),
            "damage_tracking".into(),
        ],
    })?;
    let mut start = String::new();
    std::io::stdin().lock().read_line(&mut start)?;
    if start.trim() != "start" {
        anyhow::bail!("expected start command")
    }
    bench_core::emit_event(&AdapterEvent::Ready)?;
    let epoch = Instant::now();
    if mode == MeasurementMode::Diagnostic {
        // A summary a frame: frames are further apart than the shortest
        // period, so each is summarized alone.
        frames::set_period(Some(Duration::from_millis(1)));
    }
    aizome_kit::try_run(Bench::new(spec, mode, epoch)).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}
