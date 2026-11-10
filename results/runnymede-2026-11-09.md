# runnymede, 2026-11-09: zgui, GPUI-CE and aizome-kit

The `standard` preset and the `rows` preset on one desktop, measured on its live session between 16:27 and 17:36.

| | |
|---|---|
| CPU | AMD Ryzen 7 9800X3D |
| GPU | NVIDIA GeForce RTX 5090 (the display's), Radeon 610M |
| Display | 3840 × 2160 at 240 Hz, scale 2: a frame is 4.17 ms, and the 1200 × 800 window is 2400 × 1600 pixels |
| Compositor | Aizome (Wayland), kernel 7.3.3 |
| Compiler | rustc 1.97.0-nightly (e8e4541ff 2026-04-15), the `bench-run` profile |
| zgui | `082337db` (2026-11-06) |
| GPUI-CE | the vendored copy, from `a9f2c2f1` with the harness's probes |
| aizome-kit | Sekka `00a64e62` (2026-11-09) |

Repetitions: three for the 64- and 1,024-node scenes, two for `rows`, one for the 8,192-node scenes. The four raw documents (`runnymede-n64.json`, `runnymede-n1024.json`, `runnymede-rows.json` and `runnymede-n8192.json`, 143 MB together) are not in the repository; this file is what `gui-bench summary` prints for each, with tables by scene in front.

## Reading the tables

All times are milliseconds. In the tables by scene, "frame" and "p95" are the headline passes' frame interval (median and 95th percentile); "work" is the diagnostic passes' CPU time for a frame, the median of each frame's stages added up. A run count in brackets beside a frame time means not every repetition completed.

- **A frame interval of 4.16 means the work fit in a frame.** GPUI and aizome-kit draw on the display's frame callbacks: while a frame's work fits in 4.17 ms the interval is the display's, and past that it is about the work itself. zgui presents on every other frame here however little the work (8.3 ms), and when a tick changes nothing visible it presents nothing and its interval is the time its loop took, well under a frame.
- **zgui's "work" is often an empty frame's.** Its adapter's model changes nothing visible on about half the ticks of a sparse update, so the median frame is one in which nothing was laid out or drawn.
- **aizome-kit's stages overlap.** Its program thread builds the next frame while its render thread rasterizes the last, so its work can exceed its frame interval (the decorated scenes at 8,192 nodes).
- **Scenes cut off at the window.** The window clips its content, and most of a large scene is below its bottom edge. Every framework still builds and lays out all of it.

## What went wrong, and what it touches

- **aizome-kit's diagnostic passes at 8,192 nodes ran in an 840 × 560 window**, not 1200 × 800 (twelve conditions; the runner flags them). Aizome remembers a window's geometry by app id, and reopened the adapter's window at the size a timed-out resize run had left. Its headline passes at that size ran at 1200 × 800 and their frame times agree with the diagnostic ones to within 3% (the frame is all text shaping there), so the stage times stand; the paint and rasterize columns are for the smaller window. The adapter sets no app id now, as the other two set none.
- **GPUI-CE times out on the deep topology** at 64 and 1,024 nodes, every repetition. It does in the harness author's published results too.
- **One GPUI-CE run stalled** (`paintupdate-n1024-flat-minimal-u10`, diagnostic, one repetition of three): its window was covered while the desktop was in use. That row's diagnostic p95 is the stall.
- **aizome-kit times out on the 8,192-node resize scenes and once on the decorated one**: at about 330 ms a frame, 210 frames pass the runner's 66 s limit. The frames captured before it are in the tables.
- **zgui's resize-step scene did not resize**: the compositor granted one size.

## By scene

### 64 nodes
| scene | kit frame | kit p95 | kit work | GPUI frame | GPUI p95 | GPUI work | zgui frame | zgui p95 | zgui work |
|---|---|---|---|---|---|---|---|---|---|
| coldstart · flat | 4.16 | 4.25 | 0.270 | 4.17 | 4.24 | 0.453 | 4.16 | 4.25 | 0.010 |
| idle · flat | 4.16 | 4.24 | 0.272 | 4.16 | 4.24 | 0.446 | 4.16 | 4.25 | 0.010 |
| paint · flat · 1% | 4.16 | 4.25 | 0.069 | 4.16 | 4.24 | 0.444 | 4.17 | 8.13 | 0.011 |
| layout · balanced · 1% | 4.16 | 4.25 | 0.075 | 4.16 | 4.24 | 0.553 | 4.17 | 7.91 | 0.011 |
| text · wide · 1% | 4.16 | 4.25 | 0.127 | 4.16 | 4.25 | 0.462 | 8.11 | 8.46 | 0.260 |
| paint · flat · 25% | 4.16 | 4.26 | 0.092 | 4.17 | 4.24 | 0.443 | 8.33 | 8.47 | 0.330 |
| layout · balanced · 25% | 4.17 | 4.27 | 0.113 | 4.16 | 4.23 | 0.615 | 8.10 | 8.93 | 0.359 |
| text · wide · 25% | 4.16 | 4.26 | 0.699 | 4.16 | 4.26 | 0.577 | 8.27 | 8.55 | 0.551 |
| paint · flat · 100% | 4.16 | 5.15 | 0.287 | 4.17 | 4.25 | 0.451 | 8.33 | 8.48 | 0.760 |
| layout · balanced · 100% | 4.16 | 4.92 | 0.162 | 4.16 | 4.24 | 0.531 | 8.33 | 8.48 | 1.05 |
| text · wide · 100% | 4.16 | 4.25 | 2.16 | 4.17 | 4.24 | 0.796 | 8.33 | 8.48 | 1.19 |
| structural · wide · 10% | 4.16 | 4.26 | 0.068 | 4.16 | 4.23 | 0.432 | 4.18 | 4.40 | 0.107 |
| paint · balanced · decorated · 100% | 8.78 | 9.06 | 8.45 | 4.16 | 4.24 | 0.903 | 8.33 | 8.45 | 0.892 |
| layout · deep · 100% | 4.16 | 4.24 | 1.11 | — (0/3) | — | — | 8.33 | 8.53 | 1.43 |

### 1,024 nodes
| scene | kit frame | kit p95 | kit work | GPUI frame | GPUI p95 | GPUI work | zgui frame | zgui p95 | zgui work |
|---|---|---|---|---|---|---|---|---|---|
| paint · flat · 1% | 4.17 | 4.25 | 0.882 | 4.16 | 4.27 | 3.41 | 3.68 | 4.75 | 0.105 |
| layout · balanced · 1% | 4.16 | 4.25 | 0.973 | 12.0 | 12.9 | 11.2 | 0.168 | 1.42 | 0.080 |
| text · wide · 1% | 4.16 | 4.26 | 1.24 | 4.17 | 4.23 | 2.65 | 0.244 | 2.25 | 0.156 |
| paint · flat · 25% | 4.16 | 4.33 | 1.35 | 4.16 | 4.32 | 3.42 | 5.00 | 8.84 | 1.11 |
| layout · balanced · 25% | 4.17 | 4.27 | 1.02 | 12.3 | 13.5 | 11.5 | 3.46 | 11.1 | 1.40 |
| text · wide · 25% | 8.17 | 8.51 | 8.20 | 5.49 | 5.65 | 4.71 | 2.79 | 4.08 | 1.88 |
| paint · flat · 100% | 4.16 | 4.25 | 2.57 | 4.15 | 4.29 | 3.38 | 8.33 | 8.54 | 4.47 |
| layout · balanced · 100% | 4.17 | 4.24 | 1.08 | 11.7 | 12.0 | 11.0 | 8.20 | 9.06 | 7.07 |
| text · wide · 100% | 30.3 | 31.0 | 29.5 | 8.82 | 9.03 | 8.03 | 11.8 | 12.6 | 7.79 |
| structural · wide · 10% | 4.16 | 5.14 | 0.818 | 4.16 | 4.24 | 2.75 | 1.01 | 2.88 | 0.919 |
| paint · balanced · decorated · 100% | 48.2 | 48.7 | 48.3 | 12.4 | 12.8 | 11.6 | 8.33 | 8.47 | 2.90 |
| layout · deep · 100% | 4.17 | 4.23 | 2.62 | — (0/3) | — | — | 14.0 | 15.5 | 13.0 |

### 8,192 nodes
| scene | kit frame | kit p95 | kit work | GPUI frame | GPUI p95 | GPUI work | zgui frame | zgui p95 | zgui work |
|---|---|---|---|---|---|---|---|---|---|
| paint · flat · 1% | 139 | 143 | 138 | 27.9 | 28.8 | 23.3 | 17.9 | 19.4 | 17.7 |
| layout · balanced · 1% | 144 | 148 | 144 | 179 | 184 | 174 | 1.21 | 6.69 | 0.920 |
| text · wide · 1% | 228 | 240 | 227 | 25.1 | 26.0 | 20.0 | 1.23 | 2.17 | 0.982 |
| paint · flat · 25% | 142 | 144 | 140 | 27.8 | 28.7 | 23.4 | 23.0 | 26.7 | 21.6 |
| layout · balanced · 25% | 142 | 146 | 144 | 186 | 197 | 179 | 17.3 | 55.9 | 15.5 |
| text · wide · 25% | 241 | 247 | 236 | 42.1 | 57.6 | 36.7 | 20.9 | 24.2 | 17.4 |
| paint · flat · 100% | 143 | 145 | 143 | 27.9 | 45.9 | 23.4 | 20.9 | 24.8 | 17.6 |
| layout · balanced · 100% | 142 | 146 | 146 | 177 | 199 | 172 | 68.6 | 70.6 | 63.1 |
| text · wide · 100% | 240 | 245 | 233 | 72.8 | 98.4 | 66.9 | 83.8 | 87.2 | 71.0 |
| structural · wide · 10% | 132 | 144 | 132 | 25.8 | 33.7 | 21.3 | 12.1 | 34.5 | 8.67 |
| paint · balanced · decorated · 100% | 314 (0/1) | 317 | 448 | 180 | 200 | 174 | 20.8 | 21.3 | 17.0 |
| inputlatency · balanced · decorated · 1% | 286 | 297 | 425 | 180 | 196 | 174 | 0.806 | 3.64 | 0.509 |
| resizestep · balanced · decorated · 100% | 336 (0/1) | 521 | 472 | 184 | 213 | 175 | 0.109 | 35.1 | 0.004 |
| resizestorm · balanced · decorated · 100% | 330 (0/1) | 333 | 481 | 198 | 218 | 179 | 48.4 | 50.8 | 46.5 |

### One row of a column changes (`rows`)
| scene | kit frame | kit p95 | kit work | GPUI frame | GPUI p95 | GPUI work | zgui frame | zgui p95 | zgui work |
|---|---|---|---|---|---|---|---|---|---|
| 1000 rows · one label | 4.17 | 4.24 | 0.894 | 4.16 | 4.24 | 2.44 | 0.159 | 3.61 | 0.055 |
| 1000 rows · one fill | 4.16 | 4.25 | 0.840 | 4.16 | 4.21 | 2.42 | 0.125 | 1.63 | 0.010 |
| 1000 rows · one size | 4.17 | 4.25 | 0.828 | 4.16 | 4.24 | 2.43 | 0.324 | 0.828 | 0.012 |
| 3000 rows · one label | 4.16 | 4.25 | 2.38 | 8.38 | 8.52 | 6.77 | 0.241 | 0.618 | 0.119 |
| 3000 rows · one fill | 4.17 | 4.25 | 2.32 | 8.40 | 8.55 | 6.75 | 0.230 | 0.780 | 0.010 |
| 3000 rows · one size | 4.16 | 4.24 | 2.32 | 8.43 | 8.61 | 6.92 | 0.798 | 0.870 | 0.012 |
| 10000 rows · one label | 167 | 171 | 170 | 29.6 | 30.9 | 23.9 | 0.493 | 1.06 | 0.327 |
| 10000 rows · one fill | 169 | 173 | 167 | 29.8 | 32.7 | 23.7 | 0.603 | 0.681 | 0.007 |
| 10000 rows · one size | 169 | 171 | 173 | 29.6 | 34.4 | 24.0 | 0.170 | 2.43 | 0.013 |

## Every stage

The four runs as `gui-bench summary` prints them: every scenario and framework, the headline passes and then the diagnostic ones with each pipeline stage.

### `runnymede-n64.json`

- CPU: AMD Ryzen 7 9800X3D 8-Core Processor
- GPU: 01:00.0 VGA compatible controller: NVIDIA Corporation GB202 [GeForce RTX 5090] (rev a1)
- GPU: 78:00.0 VGA compatible controller: Advanced Micro Devices, Inc. [AMD/ATI] Granite Ridge [Radeon 610M] (rev cb)
- Display server: wayland; kernel 7.3.3
- aizome-kit: 00a64e622d
- gpui-ce-vendored-from: unknown
- zgui: 082337db72

#### Headline passes

Milliseconds, over the measurement frames of every completed repetition. "First" is the first frame's time from the adapter's start; PSS is the peak, in MiB.

| scenario | framework | runs | frame p50 | frame p95 | latency p50 | latency p95 | first | PSS |
|---|---|---|---|---|---|---|---|---|
| coldstart-n64-flat-minimal-u0 | zgui | 3/3 | 4.16 | 4.25 | 4.16 | 4.25 | 145 | 81 |
| coldstart-n64-flat-minimal-u0 | gpui | 3/3 | 4.17 | 4.24 | — | — | 195 | 93 |
| coldstart-n64-flat-minimal-u0 | aizome-kit | 3/3 | 4.16 | 4.25 | — | — | 20.2 | 35 |
| idle-n64-flat-minimal-u0 | zgui | 3/3 | 4.16 | 4.25 | 4.16 | 4.24 | 145 | 81 |
| idle-n64-flat-minimal-u0 | gpui | 3/3 | 4.16 | 4.24 | — | — | 191 | 92 |
| idle-n64-flat-minimal-u0 | aizome-kit | 3/3 | 4.16 | 4.24 | — | — | 20.1 | 27 |
| paintupdate-n64-flat-minimal-u10 | zgui | 3/3 | 4.17 | 8.13 | 4.17 | 8.12 | 137 | 81 |
| paintupdate-n64-flat-minimal-u10 | gpui | 3/3 | 4.16 | 4.24 | 4.16 | 4.24 | 199 | 93 |
| paintupdate-n64-flat-minimal-u10 | aizome-kit | 3/3 | 4.16 | 4.25 | 4.16 | 4.24 | 20.3 | 42 |
| layoutupdate-n64-balanced-minimal-u10 | zgui | 3/3 | 4.17 | 7.91 | 4.16 | 7.90 | 135 | 81 |
| layoutupdate-n64-balanced-minimal-u10 | gpui | 3/3 | 4.16 | 4.24 | 4.16 | 4.24 | 199 | 92 |
| layoutupdate-n64-balanced-minimal-u10 | aizome-kit | 3/3 | 4.16 | 4.25 | 4.16 | 4.25 | 20.3 | 35 |
| textupdate-n64-wide-minimal-u10 | zgui | 3/3 | 8.11 | 8.46 | 8.10 | 8.45 | 138 | 81 |
| textupdate-n64-wide-minimal-u10 | gpui | 3/3 | 4.16 | 4.25 | 4.16 | 4.24 | 191 | 92 |
| textupdate-n64-wide-minimal-u10 | aizome-kit | 3/3 | 4.16 | 4.25 | 4.15 | 4.24 | 20.3 | 28 |
| paintupdate-n64-flat-minimal-u250 | zgui | 3/3 | 8.33 | 8.47 | 8.33 | 8.46 | 134 | 82 |
| paintupdate-n64-flat-minimal-u250 | gpui | 3/3 | 4.17 | 4.24 | 4.16 | 4.24 | 197 | 93 |
| paintupdate-n64-flat-minimal-u250 | aizome-kit | 3/3 | 4.16 | 4.26 | 4.15 | 4.25 | 20.4 | 42 |
| layoutupdate-n64-balanced-minimal-u250 | zgui | 3/3 | 8.10 | 8.93 | 8.10 | 8.92 | 137 | 81 |
| layoutupdate-n64-balanced-minimal-u250 | gpui | 3/3 | 4.16 | 4.23 | 4.16 | 4.23 | 193 | 92 |
| layoutupdate-n64-balanced-minimal-u250 | aizome-kit | 3/3 | 4.17 | 4.27 | 4.16 | 4.26 | 20.4 | 34 |
| textupdate-n64-wide-minimal-u250 | zgui | 3/3 | 8.27 | 8.55 | 8.26 | 8.54 | 142 | 87 |
| textupdate-n64-wide-minimal-u250 | gpui | 3/3 | 4.16 | 4.26 | 4.16 | 4.26 | 192 | 92 |
| textupdate-n64-wide-minimal-u250 | aizome-kit | 3/3 | 4.16 | 4.26 | 4.15 | 4.25 | 20.2 | 33 |
| paintupdate-n64-flat-minimal-u1000 | zgui | 3/3 | 8.33 | 8.48 | 8.33 | 8.48 | 135 | 82 |
| paintupdate-n64-flat-minimal-u1000 | gpui | 3/3 | 4.17 | 4.25 | 4.16 | 4.24 | 200 | 93 |
| paintupdate-n64-flat-minimal-u1000 | aizome-kit | 3/3 | 4.16 | 5.15 | 4.14 | 5.13 | 20.0 | 41 |
| layoutupdate-n64-balanced-minimal-u1000 | zgui | 3/3 | 8.33 | 8.48 | 8.32 | 8.48 | 137 | 81 |
| layoutupdate-n64-balanced-minimal-u1000 | gpui | 3/3 | 4.16 | 4.24 | 4.16 | 4.24 | 201 | 92 |
| layoutupdate-n64-balanced-minimal-u1000 | aizome-kit | 3/3 | 4.16 | 4.92 | 4.15 | 4.91 | 20.7 | 28 |
| textupdate-n64-wide-minimal-u1000 | zgui | 3/3 | 8.33 | 8.48 | 8.33 | 8.48 | 141 | 89 |
| textupdate-n64-wide-minimal-u1000 | gpui | 3/3 | 4.17 | 4.24 | 4.16 | 4.24 | 198 | 93 |
| textupdate-n64-wide-minimal-u1000 | aizome-kit | 3/3 | 4.16 | 4.25 | 4.14 | 4.23 | 20.0 | 35 |
| structuralupdate-n64-wide-minimal-u100 | zgui | 3/3 | 4.18 | 4.40 | 4.17 | 4.39 | 136 | 81 |
| structuralupdate-n64-wide-minimal-u100 | gpui | 3/3 | 4.16 | 4.23 | 4.16 | 4.23 | 198 | 92 |
| structuralupdate-n64-wide-minimal-u100 | aizome-kit | 3/3 | 4.16 | 4.26 | 4.16 | 4.25 | 20.4 | 42 |
| paintupdate-n64-balanced-decorated-u1000 | zgui | 3/3 | 8.33 | 8.45 | 8.32 | 8.45 | 134 | 82 |
| paintupdate-n64-balanced-decorated-u1000 | gpui | 3/3 | 4.16 | 4.24 | 4.16 | 4.24 | 201 | 93 |
| paintupdate-n64-balanced-decorated-u1000 | aizome-kit | 3/3 | 8.78 | 9.06 | 8.69 | 8.97 | 20.1 | 38 |
| layoutupdate-n64-deep-minimal-u1000 | zgui | 3/3 | 8.33 | 8.53 | 8.32 | 8.52 | 136 | 82 |
| layoutupdate-n64-deep-minimal-u1000 | gpui | 0/3 | — | — | — | — | — | 90 |
| layoutupdate-n64-deep-minimal-u1000 | aizome-kit | 3/3 | 4.16 | 4.24 | 4.14 | 4.22 | 20.4 | 35 |

#### Diagnostic passes

Milliseconds, over the measurement frames of every completed repetition. "First" is the first frame's time from the adapter's start; PSS is the peak, in MiB.
Stages are medians of CPU time; "work" is the median of each frame's stages added up, and "gpu" is prepare, encode and submit together.

| scenario | framework | runs | frame p50 | frame p95 | latency p50 | latency p95 | first | PSS | view | style | layout | prepaint | paint | raster | gpu | present | work |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| coldstart-n64-flat-minimal-u0 | zgui | 3/3 | 4.16 | 4.25 | 4.16 | 4.24 | 134 | 82 | 0.000 | 0.001 | 0.007 | 0.001 | 0.001 | — | — | — | 0.010 |
| coldstart-n64-flat-minimal-u0 | gpui | 3/3 | 4.16 | 4.25 | — | — | 201 | 92 | 0.015 | — | 0.148 | 0.013 | 0.082 | — | 0.051 | 0.140 | 0.453 |
| coldstart-n64-flat-minimal-u0 | aizome-kit | 3/3 | 4.16 | 4.25 | — | — | 20.4 | 27 | 0.011 | — | 0.007 | 0.043 | 0.017 | 0.185 | — | 0.002 | 0.270 |
| idle-n64-flat-minimal-u0 | zgui | 3/3 | 4.16 | 4.23 | 4.16 | 4.23 | 133 | 82 | 0.000 | 0.001 | 0.007 | 0.001 | 0.001 | — | — | — | 0.010 |
| idle-n64-flat-minimal-u0 | gpui | 3/3 | 4.16 | 4.25 | — | — | 203 | 93 | 0.014 | — | 0.148 | 0.013 | 0.083 | — | 0.045 | 0.135 | 0.446 |
| idle-n64-flat-minimal-u0 | aizome-kit | 3/3 | 4.16 | 4.25 | — | — | 20.3 | 27 | 0.011 | — | 0.007 | 0.043 | 0.018 | 0.183 | — | 0.003 | 0.272 |
| paintupdate-n64-flat-minimal-u10 | zgui | 3/3 | 4.17 | 8.10 | 4.17 | 8.09 | 133 | 82 | 0.002 | 0.001 | 0.007 | 0.001 | 0.001 | — | 0.066 | 0.151 | 0.011 |
| paintupdate-n64-flat-minimal-u10 | gpui | 3/3 | 4.17 | 4.25 | 4.16 | 4.24 | 208 | 93 | 0.014 | — | 0.147 | 0.013 | 0.082 | — | 0.045 | 0.137 | 0.444 |
| paintupdate-n64-flat-minimal-u10 | aizome-kit | 3/3 | 4.16 | 4.25 | 4.16 | 4.25 | 20.2 | 42 | 0.012 | — | 0.008 | 0.043 | 0.003 | 0.000 | — | 0.002 | 0.069 |
| layoutupdate-n64-balanced-minimal-u10 | zgui | 3/3 | 4.17 | 7.91 | 4.16 | 7.89 | 140 | 82 | 0.002 | 0.001 | 0.007 | 0.001 | 0.001 | — | 0.065 | 0.159 | 0.011 |
| layoutupdate-n64-balanced-minimal-u10 | gpui | 3/3 | 4.16 | 4.25 | 4.16 | 4.24 | 200 | 92 | 0.016 | — | 0.237 | 0.013 | 0.057 | — | 0.043 | 0.178 | 0.553 |
| layoutupdate-n64-balanced-minimal-u10 | aizome-kit | 3/3 | 4.17 | 4.24 | 4.16 | 4.23 | 20.1 | 35 | 0.015 | — | 0.009 | 0.045 | 0.003 | 0.000 | — | 0.002 | 0.075 |
| textupdate-n64-wide-minimal-u10 | zgui | 3/3 | 8.10 | 8.45 | 8.09 | 8.44 | 133 | 82 | 0.004 | 0.001 | 0.050 | 0.003 | 0.020 | — | 0.063 | 0.144 | 0.260 |
| textupdate-n64-wide-minimal-u10 | gpui | 3/3 | 4.17 | 4.23 | 4.16 | 4.22 | 203 | 92 | 0.015 | — | 0.138 | 0.014 | 0.060 | — | 0.066 | 0.170 | 0.462 |
| textupdate-n64-wide-minimal-u10 | aizome-kit | 3/3 | 4.17 | 4.25 | 4.16 | 4.25 | 20.2 | 28 | 0.013 | — | 0.049 | 0.045 | 0.004 | 0.007 | — | 0.005 | 0.127 |
| paintupdate-n64-flat-minimal-u250 | zgui | 3/3 | 8.33 | 8.49 | 8.32 | 8.48 | 132 | 84 | 0.010 | 0.037 | 0.011 | 0.001 | 0.024 | — | 0.083 | 0.159 | 0.330 |
| paintupdate-n64-flat-minimal-u250 | gpui | 3/3 | 4.16 | 4.24 | 4.16 | 4.24 | 205 | 92 | 0.015 | — | 0.147 | 0.013 | 0.080 | — | 0.044 | 0.136 | 0.443 |
| paintupdate-n64-flat-minimal-u250 | aizome-kit | 3/3 | 4.16 | 4.23 | 4.16 | 4.22 | 20.2 | 41 | 0.011 | — | 0.008 | 0.044 | 0.006 | 0.015 | — | 0.008 | 0.092 |
| layoutupdate-n64-balanced-minimal-u250 | zgui | 3/3 | 8.09 | 8.93 | 8.09 | 8.92 | 133 | 83 | 0.010 | 0.037 | 0.061 | 0.002 | 0.021 | — | 0.082 | 0.159 | 0.359 |
| layoutupdate-n64-balanced-minimal-u250 | gpui | 3/3 | 4.16 | 4.25 | 4.16 | 4.25 | 206 | 92 | 0.017 | — | 0.293 | 0.013 | 0.058 | — | 0.048 | 0.180 | 0.615 |
| layoutupdate-n64-balanced-minimal-u250 | aizome-kit | 3/3 | 4.17 | 4.25 | 4.16 | 4.24 | 20.1 | 35 | 0.015 | — | 0.009 | 0.046 | 0.006 | 0.024 | — | 0.008 | 0.113 |
| textupdate-n64-wide-minimal-u250 | zgui | 3/3 | 8.27 | 8.55 | 8.26 | 8.55 | 133 | 87 | 0.015 | 0.001 | 0.185 | 0.008 | 0.097 | — | 0.093 | 0.165 | 0.551 |
| textupdate-n64-wide-minimal-u250 | gpui | 3/3 | 4.16 | 4.26 | 4.16 | 4.26 | 206 | 92 | 0.015 | — | 0.272 | 0.013 | 0.062 | — | 0.045 | 0.167 | 0.577 |
| textupdate-n64-wide-minimal-u250 | aizome-kit | 3/3 | 4.16 | 4.25 | 4.15 | 4.23 | 20.0 | 33 | 0.014 | — | 0.524 | 0.052 | 0.010 | 0.089 | — | 0.017 | 0.699 |
| paintupdate-n64-flat-minimal-u1000 | zgui | 3/3 | 8.33 | 8.49 | 8.32 | 8.48 | 132 | 84 | 0.049 | 0.096 | 0.093 | 0.001 | 0.139 | — | 0.144 | 0.237 | 0.760 |
| paintupdate-n64-flat-minimal-u1000 | gpui | 3/3 | 4.16 | 4.25 | 4.16 | 4.25 | 193 | 93 | 0.015 | — | 0.148 | 0.013 | 0.083 | — | 0.043 | 0.141 | 0.451 |
| paintupdate-n64-flat-minimal-u1000 | aizome-kit | 3/3 | 4.17 | 4.25 | 4.14 | 4.22 | 20.2 | 41 | 0.011 | — | 0.007 | 0.084 | 0.020 | 0.137 | — | 0.027 | 0.287 |
| layoutupdate-n64-balanced-minimal-u1000 | zgui | 3/3 | 8.32 | 8.48 | 8.31 | 8.47 | 144 | 83 | 0.051 | 0.111 | 0.475 | 0.003 | 0.108 | — | 0.116 | 0.183 | 1.05 |
| layoutupdate-n64-balanced-minimal-u1000 | gpui | 3/3 | 4.16 | 4.25 | 4.16 | 4.25 | 201 | 92 | 0.017 | — | 0.224 | 0.013 | 0.056 | — | 0.042 | 0.176 | 0.531 |
| layoutupdate-n64-balanced-minimal-u1000 | aizome-kit | 3/3 | 4.16 | 4.24 | 4.15 | 4.22 | 19.9 | 28 | 0.015 | — | 0.009 | 0.047 | 0.016 | 0.072 | — | 0.003 | 0.162 |
| textupdate-n64-wide-minimal-u1000 | zgui | 3/3 | 8.32 | 8.50 | 8.31 | 8.49 | 134 | 89 | 0.047 | 0.001 | 0.498 | 0.022 | 0.337 | — | 0.111 | 0.173 | 1.19 |
| textupdate-n64-wide-minimal-u1000 | gpui | 3/3 | 4.16 | 4.24 | 4.16 | 4.24 | 208 | 92 | 0.015 | — | 0.487 | 0.013 | 0.071 | — | 0.042 | 0.167 | 0.796 |
| textupdate-n64-wide-minimal-u1000 | aizome-kit | 3/3 | 4.16 | 4.24 | 4.14 | 4.22 | 20.1 | 35 | 0.014 | — | 1.91 | 0.068 | 0.018 | 0.118 | — | 0.025 | 2.16 |
| structuralupdate-n64-wide-minimal-u100 | zgui | 3/3 | 4.15 | 4.39 | 4.14 | 4.39 | 136 | 82 | 0.018 | 0.004 | 0.060 | 0.023 | 0.001 | — | — | — | 0.107 |
| structuralupdate-n64-wide-minimal-u100 | gpui | 3/3 | 4.16 | 4.25 | 4.16 | 4.24 | 202 | 92 | 0.014 | — | 0.127 | 0.013 | 0.058 | — | 0.049 | 0.166 | 0.432 |
| structuralupdate-n64-wide-minimal-u100 | aizome-kit | 3/3 | 4.16 | 4.26 | 4.16 | 4.25 | 20.1 | 42 | 0.012 | — | 0.009 | 0.042 | 0.003 | 0.000 | — | 0.002 | 0.068 |
| paintupdate-n64-balanced-decorated-u1000 | zgui | 3/3 | 8.33 | 8.44 | 8.32 | 8.44 | 138 | 83 | 0.052 | 0.122 | 0.020 | 0.001 | 0.096 | — | 0.162 | 0.436 | 0.892 |
| paintupdate-n64-balanced-decorated-u1000 | gpui | 3/3 | 4.16 | 4.25 | 4.16 | 4.24 | 197 | 93 | 0.019 | — | 0.225 | 0.014 | 0.137 | — | 0.076 | 0.429 | 0.903 |
| paintupdate-n64-balanced-decorated-u1000 | aizome-kit | 3/3 | 8.76 | 9.00 | 8.68 | 8.92 | 20.1 | 38 | 0.015 | — | 0.009 | 0.030 | 0.085 | 8.30 | — | 0.010 | 8.45 |
| layoutupdate-n64-deep-minimal-u1000 | zgui | 3/3 | 8.34 | 8.54 | 8.33 | 8.53 | 141 | 83 | 0.047 | 0.185 | 0.343 | 0.002 | 0.144 | — | 0.162 | 0.571 | 1.43 |
| layoutupdate-n64-deep-minimal-u1000 | gpui | 0/3 | — | — | — | — | — | 90 | — | — | — | — | — | — | — | — | — |
| layoutupdate-n64-deep-minimal-u1000 | aizome-kit | 3/3 | 4.16 | 4.26 | 4.14 | 4.23 | 20.3 | 35 | 0.014 | — | 0.016 | 0.059 | 0.020 | 1.01 | — | 0.003 | 1.11 |

#### Warnings

- aizome-kit: per-process VRAM unavailable from DRM fdinfo and NVML (×84)
- gpui: layoutupdate-n64-deep-minimal-u1000 timed out (×6)
- zgui: compositor chose {(2400, 1600), (3366, 2060)} instead of requested 1200x800; use the observed size for comparisons (×1)
- zgui: compositor chose {(2400, 1600)} instead of requested 1200x800; use the observed size for comparisons (×83)

### `runnymede-n1024.json`

- CPU: AMD Ryzen 7 9800X3D 8-Core Processor
- GPU: 01:00.0 VGA compatible controller: NVIDIA Corporation GB202 [GeForce RTX 5090] (rev a1)
- GPU: 78:00.0 VGA compatible controller: Advanced Micro Devices, Inc. [AMD/ATI] Granite Ridge [Radeon 610M] (rev cb)
- Display server: wayland; kernel 7.3.3
- aizome-kit: 00a64e622d
- gpui-ce-vendored-from: unknown
- zgui: 082337db72

#### Headline passes

Milliseconds, over the measurement frames of every completed repetition. "First" is the first frame's time from the adapter's start; PSS is the peak, in MiB.

| scenario | framework | runs | frame p50 | frame p95 | latency p50 | latency p95 | first | PSS |
|---|---|---|---|---|---|---|---|---|
| paintupdate-n1024-flat-minimal-u10 | zgui | 3/3 | 3.68 | 4.75 | 3.67 | 4.75 | 165 | 110 |
| paintupdate-n1024-flat-minimal-u10 | gpui | 3/3 | 4.16 | 4.27 | 4.16 | 4.27 | 211 | 109 |
| paintupdate-n1024-flat-minimal-u10 | aizome-kit | 3/3 | 4.17 | 4.25 | 4.15 | 4.24 | 40.1 | 48 |
| layoutupdate-n1024-balanced-minimal-u10 | zgui | 3/3 | 0.168 | 1.42 | 0.165 | 1.42 | 167 | 111 |
| layoutupdate-n1024-balanced-minimal-u10 | gpui | 3/3 | 12.0 | 12.9 | 12.0 | 12.9 | 207 | 106 |
| layoutupdate-n1024-balanced-minimal-u10 | aizome-kit | 3/3 | 4.16 | 4.25 | 4.16 | 4.25 | 40.6 | 48 |
| textupdate-n1024-wide-minimal-u10 | zgui | 3/3 | 0.244 | 2.25 | 0.242 | 2.25 | 165 | 113 |
| textupdate-n1024-wide-minimal-u10 | gpui | 3/3 | 4.17 | 4.23 | 4.16 | 4.23 | 197 | 102 |
| textupdate-n1024-wide-minimal-u10 | aizome-kit | 3/3 | 4.16 | 4.26 | 4.16 | 4.25 | 40.9 | 48 |
| paintupdate-n1024-flat-minimal-u250 | zgui | 3/3 | 5.00 | 8.84 | 5.00 | 8.84 | 167 | 111 |
| paintupdate-n1024-flat-minimal-u250 | gpui | 3/3 | 4.16 | 4.32 | 4.16 | 4.32 | 199 | 109 |
| paintupdate-n1024-flat-minimal-u250 | aizome-kit | 3/3 | 4.16 | 4.33 | 4.11 | 4.27 | 40.9 | 48 |
| layoutupdate-n1024-balanced-minimal-u250 | zgui | 3/3 | 3.46 | 11.1 | 3.46 | 11.1 | 165 | 113 |
| layoutupdate-n1024-balanced-minimal-u250 | gpui | 3/3 | 12.3 | 13.5 | 12.3 | 13.5 | 211 | 106 |
| layoutupdate-n1024-balanced-minimal-u250 | aizome-kit | 3/3 | 4.17 | 4.27 | 4.15 | 4.25 | 41.3 | 48 |
| textupdate-n1024-wide-minimal-u250 | zgui | 3/3 | 2.79 | 4.08 | 2.78 | 4.08 | 162 | 120 |
| textupdate-n1024-wide-minimal-u250 | gpui | 3/3 | 5.49 | 5.65 | 5.49 | 5.64 | 202 | 102 |
| textupdate-n1024-wide-minimal-u250 | aizome-kit | 3/3 | 8.17 | 8.51 | 8.16 | 8.51 | 40.3 | 48 |
| paintupdate-n1024-flat-minimal-u1000 | zgui | 3/3 | 8.33 | 8.54 | 8.32 | 8.54 | 165 | 112 |
| paintupdate-n1024-flat-minimal-u1000 | gpui | 3/3 | 4.15 | 4.29 | 4.15 | 4.29 | 206 | 109 |
| paintupdate-n1024-flat-minimal-u1000 | aizome-kit | 3/3 | 4.16 | 4.25 | 4.01 | 4.10 | 40.7 | 48 |
| layoutupdate-n1024-balanced-minimal-u1000 | zgui | 3/3 | 8.20 | 9.06 | 8.20 | 9.06 | 160 | 113 |
| layoutupdate-n1024-balanced-minimal-u1000 | gpui | 3/3 | 11.7 | 12.0 | 11.7 | 12.0 | 212 | 105 |
| layoutupdate-n1024-balanced-minimal-u1000 | aizome-kit | 3/3 | 4.17 | 4.24 | 4.14 | 4.22 | 40.5 | 41 |
| textupdate-n1024-wide-minimal-u1000 | zgui | 3/3 | 11.8 | 12.6 | 11.8 | 12.6 | 165 | 122 |
| textupdate-n1024-wide-minimal-u1000 | gpui | 3/3 | 8.82 | 9.03 | 8.81 | 9.03 | 200 | 103 |
| textupdate-n1024-wide-minimal-u1000 | aizome-kit | 3/3 | 30.3 | 31.0 | 30.3 | 31.0 | 40.8 | 43 |
| structuralupdate-n1024-wide-minimal-u100 | zgui | 3/3 | 1.01 | 2.88 | 1.00 | 2.87 | 169 | 113 |
| structuralupdate-n1024-wide-minimal-u100 | gpui | 3/3 | 4.16 | 4.24 | 4.16 | 4.24 | 204 | 101 |
| structuralupdate-n1024-wide-minimal-u100 | aizome-kit | 3/3 | 4.16 | 5.14 | 4.16 | 5.13 | 40.4 | 48 |
| paintupdate-n1024-balanced-decorated-u1000 | zgui | 3/3 | 8.33 | 8.47 | 8.32 | 8.46 | 166 | 111 |
| paintupdate-n1024-balanced-decorated-u1000 | gpui | 3/3 | 12.4 | 12.8 | 12.4 | 12.8 | 212 | 105 |
| paintupdate-n1024-balanced-decorated-u1000 | aizome-kit | 3/3 | 48.2 | 48.7 | 47.2 | 47.7 | 39.7 | 47 |
| layoutupdate-n1024-deep-minimal-u1000 | zgui | 3/3 | 14.0 | 15.5 | 14.0 | 15.5 | 178 | 121 |
| layoutupdate-n1024-deep-minimal-u1000 | gpui | 0/3 | — | — | — | — | — | 104 |
| layoutupdate-n1024-deep-minimal-u1000 | aizome-kit | 3/3 | 4.17 | 4.23 | 4.14 | 4.21 | 76.7 | 38 |

#### Diagnostic passes

Milliseconds, over the measurement frames of every completed repetition. "First" is the first frame's time from the adapter's start; PSS is the peak, in MiB.
Stages are medians of CPU time; "work" is the median of each frame's stages added up, and "gpu" is prepare, encode and submit together.

| scenario | framework | runs | frame p50 | frame p95 | latency p50 | latency p95 | first | PSS | view | style | layout | prepaint | paint | raster | gpu | present | work |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| paintupdate-n1024-flat-minimal-u10 | zgui | 3/3 | 3.68 | 4.78 | 3.66 | 4.77 | 164 | 110 | 0.007 | 0.051 | 0.025 | 0.001 | 0.001 | — | 0.093 | 0.170 | 0.105 |
| paintupdate-n1024-flat-minimal-u10 | gpui | 2/3 | 8.32 | 996 | 8.31 | 996 | 245 | 109 | 0.215 | — | 2.02 | 0.185 | 0.727 | — | 0.090 | 0.171 | 3.41 |
| paintupdate-n1024-flat-minimal-u10 | aizome-kit | 3/3 | 4.16 | 4.23 | 4.15 | 4.22 | 40.0 | 49 | 0.155 | — | 0.120 | 0.582 | 0.005 | 0.001 | — | 0.007 | 0.882 |
| layoutupdate-n1024-balanced-minimal-u10 | zgui | 3/3 | 0.175 | 1.41 | 0.172 | 1.40 | 162 | 112 | 0.006 | 0.017 | 0.051 | 0.004 | 0.000 | — | 0.134 | 0.183 | 0.080 |
| layoutupdate-n1024-balanced-minimal-u10 | gpui | 3/3 | 12.5 | 16.7 | 12.5 | 16.7 | 221 | 106 | 0.230 | — | 10.1 | 0.186 | 0.334 | — | 0.064 | 0.218 | 11.2 |
| layoutupdate-n1024-balanced-minimal-u10 | aizome-kit | 3/3 | 4.17 | 4.25 | 4.16 | 4.24 | 40.7 | 48 | 0.207 | — | 0.140 | 0.615 | 0.004 | 0.000 | — | 0.003 | 0.973 |
| textupdate-n1024-wide-minimal-u10 | zgui | 3/3 | 0.284 | 2.18 | 0.280 | 2.17 | 162 | 115 | 0.009 | 0.000 | 0.141 | 0.005 | 0.000 | — | 0.124 | 0.110 | 0.156 |
| textupdate-n1024-wide-minimal-u10 | gpui | 3/3 | 4.16 | 4.24 | 4.16 | 4.24 | 202 | 102 | 0.203 | — | 1.69 | 0.185 | 0.321 | — | 0.065 | 0.182 | 2.65 |
| textupdate-n1024-wide-minimal-u10 | aizome-kit | 3/3 | 4.16 | 4.24 | 4.16 | 4.23 | 41.0 | 48 | 0.176 | — | 0.484 | 0.564 | 0.003 | 0.000 | — | 0.002 | 1.24 |
| paintupdate-n1024-flat-minimal-u250 | zgui | 3/3 | 5.00 | 8.90 | 4.99 | 8.89 | 175 | 112 | 0.135 | 0.215 | 0.151 | 0.001 | 0.187 | — | 0.221 | 0.275 | 1.11 |
| paintupdate-n1024-flat-minimal-u250 | gpui | 3/3 | 8.32 | 8.41 | 8.31 | 8.41 | 209 | 109 | 0.217 | — | 2.04 | 0.184 | 0.726 | — | 0.087 | 0.167 | 3.42 |
| paintupdate-n1024-flat-minimal-u250 | aizome-kit | 3/3 | 4.17 | 4.32 | 4.12 | 4.26 | 40.5 | 48 | 0.150 | — | 0.120 | 0.667 | 0.054 | 0.288 | — | 0.042 | 1.35 |
| layoutupdate-n1024-balanced-minimal-u250 | zgui | 3/3 | 3.38 | 11.1 | 3.37 | 11.1 | 161 | 114 | 0.126 | 0.212 | 0.967 | 0.004 | 0.001 | — | 0.101 | 0.172 | 1.40 |
| layoutupdate-n1024-balanced-minimal-u250 | gpui | 3/3 | 12.5 | 16.7 | 12.5 | 16.7 | 218 | 106 | 0.228 | — | 10.5 | 0.186 | 0.336 | — | 0.068 | 0.216 | 11.5 |
| layoutupdate-n1024-balanced-minimal-u250 | aizome-kit | 3/3 | 4.17 | 4.27 | 4.15 | 4.25 | 40.9 | 48 | 0.210 | — | 0.142 | 0.616 | 0.006 | 0.001 | — | 0.004 | 1.02 |
| textupdate-n1024-wide-minimal-u250 | zgui | 3/3 | 2.81 | 4.07 | 2.80 | 4.06 | 165 | 120 | 0.151 | 0.000 | 1.64 | 0.065 | 0.001 | — | 0.115 | 0.172 | 1.88 |
| textupdate-n1024-wide-minimal-u250 | gpui | 3/3 | 8.33 | 8.45 | 8.33 | 8.44 | 205 | 102 | 0.221 | — | 3.75 | 0.184 | 0.324 | — | 0.050 | 0.174 | 4.71 |
| textupdate-n1024-wide-minimal-u250 | aizome-kit | 3/3 | 8.12 | 8.57 | 8.11 | 8.55 | 40.0 | 48 | 0.160 | — | 7.36 | 0.564 | 0.001 | 0.000 | — | 0.009 | 8.20 |
| paintupdate-n1024-flat-minimal-u1000 | zgui | 3/3 | 8.33 | 8.50 | 8.32 | 8.49 | 164 | 113 | 0.625 | 1.02 | 0.651 | 0.001 | 0.972 | — | 0.542 | 0.650 | 4.47 |
| paintupdate-n1024-flat-minimal-u1000 | gpui | 3/3 | 8.24 | 8.37 | 8.23 | 8.37 | 207 | 109 | 0.200 | — | 2.02 | 0.181 | 0.720 | — | 0.086 | 0.167 | 3.38 |
| paintupdate-n1024-flat-minimal-u1000 | aizome-kit | 3/3 | 4.17 | 4.26 | 4.01 | 4.10 | 40.6 | 48 | 0.146 | — | 0.119 | 1.25 | 0.153 | 0.802 | — | 0.085 | 2.57 |
| layoutupdate-n1024-balanced-minimal-u1000 | zgui | 3/3 | 8.23 | 9.03 | 8.22 | 9.02 | 162 | 115 | 0.594 | 1.09 | 4.94 | 0.020 | 0.114 | — | 0.123 | 0.192 | 7.07 |
| layoutupdate-n1024-balanced-minimal-u1000 | gpui | 3/3 | 12.5 | 14.1 | 12.5 | 14.1 | 214 | 105 | 0.218 | — | 9.94 | 0.181 | 0.326 | — | 0.063 | 0.215 | 11.0 |
| layoutupdate-n1024-balanced-minimal-u1000 | aizome-kit | 3/3 | 4.16 | 4.25 | 4.14 | 4.23 | 40.0 | 41 | 0.212 | — | 0.141 | 0.621 | 0.017 | 0.080 | — | 0.004 | 1.08 |
| textupdate-n1024-wide-minimal-u1000 | zgui | 3/3 | 12.0 | 12.6 | 12.0 | 12.6 | 161 | 123 | 0.629 | 0.001 | 6.23 | 0.272 | 0.358 | — | 0.109 | 0.178 | 7.79 |
| textupdate-n1024-wide-minimal-u1000 | gpui | 3/3 | 12.5 | 12.6 | 12.5 | 12.6 | 206 | 103 | 0.233 | — | 7.00 | 0.185 | 0.353 | — | 0.065 | 0.176 | 8.03 |
| textupdate-n1024-wide-minimal-u1000 | aizome-kit | 3/3 | 29.4 | 30.6 | 29.4 | 30.6 | 39.7 | 43 | 0.162 | — | 28.6 | 0.599 | 0.017 | 0.123 | — | 0.036 | 29.5 |
| structuralupdate-n1024-wide-minimal-u100 | zgui | 3/3 | 1.22 | 2.98 | 1.21 | 2.97 | 166 | 113 | 0.173 | 0.002 | 0.434 | 0.215 | 0.001 | — | — | — | 0.919 |
| structuralupdate-n1024-wide-minimal-u100 | gpui | 3/3 | 4.16 | 4.22 | 4.16 | 4.22 | 207 | 101 | 0.188 | — | 1.87 | 0.176 | 0.304 | — | 0.061 | 0.180 | 2.75 |
| structuralupdate-n1024-wide-minimal-u100 | aizome-kit | 3/3 | 4.16 | 4.26 | 4.16 | 4.26 | 40.5 | 48 | 0.147 | — | 0.113 | 0.548 | 0.003 | 0.000 | — | 0.004 | 0.818 |
| paintupdate-n1024-balanced-decorated-u1000 | zgui | 3/3 | 8.32 | 8.54 | 8.31 | 8.53 | 179 | 112 | 0.704 | 1.21 | 0.192 | 0.001 | 0.093 | — | 0.155 | 0.554 | 2.90 |
| paintupdate-n1024-balanced-decorated-u1000 | gpui | 3/3 | 12.5 | 16.7 | 12.5 | 16.7 | 215 | 105 | 0.256 | — | 9.90 | 0.212 | 0.493 | — | 0.102 | 0.587 | 11.6 |
| paintupdate-n1024-balanced-decorated-u1000 | aizome-kit | 3/3 | 48.1 | 48.6 | 47.1 | 47.6 | 39.5 | 47 | 0.168 | — | 0.124 | 0.310 | 1.01 | 46.6 | — | 0.139 | 48.3 |
| layoutupdate-n1024-deep-minimal-u1000 | zgui | 3/3 | 14.1 | 15.6 | 14.1 | 15.6 | 178 | 122 | 0.577 | 3.71 | 7.69 | 0.017 | 0.149 | — | 0.167 | 0.732 | 13.0 |
| layoutupdate-n1024-deep-minimal-u1000 | gpui | 0/3 | — | — | — | — | — | 104 | — | — | — | — | — | — | — | — | — |
| layoutupdate-n1024-deep-minimal-u1000 | aizome-kit | 3/3 | 4.16 | 4.25 | 4.14 | 4.22 | 76.6 | 38 | 0.187 | — | 0.309 | 0.767 | 0.020 | 1.33 | — | 0.003 | 2.62 |

#### Warnings

- aizome-kit: per-process VRAM unavailable from DRM fdinfo and NVML (×72)
- gpui: layoutupdate-n1024-deep-minimal-u1000 timed out (×6)
- gpui: paintupdate-n1024-flat-minimal-u10 timed out (×1)
- zgui: compositor chose {(2400, 1600)} instead of requested 1200x800; use the observed size for comparisons (×72)

### `runnymede-rows.json`

- CPU: AMD Ryzen 7 9800X3D 8-Core Processor
- GPU: 01:00.0 VGA compatible controller: NVIDIA Corporation GB202 [GeForce RTX 5090] (rev a1)
- GPU: 78:00.0 VGA compatible controller: Advanced Micro Devices, Inc. [AMD/ATI] Granite Ridge [Radeon 610M] (rev cb)
- Display server: wayland; kernel 7.3.3
- aizome-kit: 00a64e622d
- gpui-ce-vendored-from: unknown
- zgui: 082337db72

#### Headline passes

Milliseconds, over the measurement frames of every completed repetition. "First" is the first frame's time from the adapter's start; PSS is the peak, in MiB.

| scenario | framework | runs | frame p50 | frame p95 | latency p50 | latency p95 | first | PSS |
|---|---|---|---|---|---|---|---|---|
| rows-n1000-one-label | zgui | 2/2 | 0.159 | 3.61 | 0.154 | 3.61 | 162 | 108 |
| rows-n1000-one-label | gpui | 2/2 | 4.16 | 4.24 | 4.16 | 4.24 | 199 | 101 |
| rows-n1000-one-label | aizome-kit | 2/2 | 4.17 | 4.24 | 4.16 | 4.23 | 39.3 | 40 |
| rows-n1000-one-fill | zgui | 2/2 | 0.125 | 1.63 | 0.123 | 1.63 | 160 | 108 |
| rows-n1000-one-fill | gpui | 2/2 | 4.16 | 4.21 | 4.16 | 4.21 | 197 | 101 |
| rows-n1000-one-fill | aizome-kit | 2/2 | 4.16 | 4.25 | 4.16 | 4.24 | 39.6 | 48 |
| rows-n1000-one-size | zgui | 2/2 | 0.324 | 0.828 | 0.322 | 0.826 | 161 | 108 |
| rows-n1000-one-size | gpui | 2/2 | 4.16 | 4.24 | 4.16 | 4.24 | 200 | 101 |
| rows-n1000-one-size | aizome-kit | 2/2 | 4.17 | 4.25 | 4.16 | 4.24 | 39.5 | 40 |
| rows-n3000-one-label | zgui | 2/2 | 0.241 | 0.618 | 0.238 | 0.612 | 218 | 166 |
| rows-n3000-one-label | gpui | 2/2 | 8.38 | 8.52 | 8.37 | 8.51 | 219 | 120 |
| rows-n3000-one-label | aizome-kit | 2/2 | 4.16 | 4.25 | 4.16 | 4.24 | 80.9 | 46 |
| rows-n3000-one-fill | zgui | 2/2 | 0.230 | 0.780 | 0.228 | 0.778 | 216 | 166 |
| rows-n3000-one-fill | gpui | 2/2 | 8.40 | 8.55 | 8.40 | 8.54 | 215 | 121 |
| rows-n3000-one-fill | aizome-kit | 2/2 | 4.17 | 4.25 | 4.16 | 4.24 | 79.8 | 61 |
| rows-n3000-one-size | zgui | 2/2 | 0.798 | 0.870 | 0.796 | 0.868 | 216 | 166 |
| rows-n3000-one-size | gpui | 2/2 | 8.43 | 8.61 | 8.43 | 8.60 | 217 | 121 |
| rows-n3000-one-size | aizome-kit | 2/2 | 4.16 | 4.24 | 4.16 | 4.24 | 79.7 | 55 |
| rows-n10000-one-label | zgui | 2/2 | 0.493 | 1.06 | 0.490 | 1.06 | 497 | 344 |
| rows-n10000-one-label | gpui | 2/2 | 29.6 | 30.9 | 29.6 | 30.9 | 274 | 189 |
| rows-n10000-one-label | aizome-kit | 2/2 | 167 | 171 | 167 | 171 | 710 | 116 |
| rows-n10000-one-fill | zgui | 2/2 | 0.603 | 0.681 | 0.601 | 0.679 | 493 | 344 |
| rows-n10000-one-fill | gpui | 2/2 | 29.8 | 32.7 | 29.8 | 32.7 | 277 | 189 |
| rows-n10000-one-fill | aizome-kit | 2/2 | 169 | 173 | 169 | 173 | 704 | 126 |
| rows-n10000-one-size | zgui | 2/2 | 0.170 | 2.43 | 0.165 | 2.43 | 492 | 345 |
| rows-n10000-one-size | gpui | 2/2 | 29.6 | 34.4 | 29.6 | 34.4 | 276 | 188 |
| rows-n10000-one-size | aizome-kit | 2/2 | 169 | 171 | 169 | 171 | 712 | 116 |

#### Diagnostic passes

Milliseconds, over the measurement frames of every completed repetition. "First" is the first frame's time from the adapter's start; PSS is the peak, in MiB.
Stages are medians of CPU time; "work" is the median of each frame's stages added up, and "gpu" is prepare, encode and submit together.

| scenario | framework | runs | frame p50 | frame p95 | latency p50 | latency p95 | first | PSS | view | style | layout | prepaint | paint | raster | gpu | present | work |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| rows-n1000-one-label | zgui | 2/2 | 0.164 | 3.69 | 0.160 | 3.68 | 158 | 108 | 0.001 | 0.000 | 0.051 | 0.001 | 0.000 | — | 0.083 | 0.075 | 0.055 |
| rows-n1000-one-label | gpui | 2/2 | 4.16 | 4.25 | 4.16 | 4.25 | 197 | 101 | 0.185 | — | 1.55 | 0.176 | 0.302 | — | 0.046 | 0.174 | 2.44 |
| rows-n1000-one-label | aizome-kit | 2/2 | 4.18 | 4.85 | 4.17 | 4.84 | 39.6 | 48 | 0.151 | — | 0.164 | 0.567 | 0.004 | 0.000 | — | 0.002 | 0.894 |
| rows-n1000-one-fill | zgui | 2/2 | 0.129 | 0.820 | 0.126 | 0.809 | 156 | 108 | 0.001 | 0.001 | 0.007 | 0.000 | 0.000 | — | 0.118 | 0.156 | 0.010 |
| rows-n1000-one-fill | gpui | 2/2 | 4.16 | 4.23 | 4.16 | 4.23 | 198 | 101 | 0.185 | — | 1.55 | 0.176 | 0.299 | — | 0.042 | 0.173 | 2.42 |
| rows-n1000-one-fill | aizome-kit | 2/2 | 4.17 | 4.22 | 4.16 | 4.21 | 39.5 | 47 | 0.151 | — | 0.120 | 0.560 | 0.003 | 0.000 | — | 0.002 | 0.840 |
| rows-n1000-one-size | zgui | 2/2 | 0.333 | 2.41 | 0.329 | 2.41 | 159 | 108 | 0.001 | 0.001 | 0.007 | 0.001 | 0.000 | — | 0.114 | 0.160 | 0.012 |
| rows-n1000-one-size | gpui | 2/2 | 4.16 | 4.23 | 4.16 | 4.22 | 196 | 101 | 0.189 | — | 1.54 | 0.177 | 0.299 | — | 0.047 | 0.174 | 2.43 |
| rows-n1000-one-size | aizome-kit | 2/2 | 4.17 | 4.25 | 4.16 | 4.24 | 39.9 | 48 | 0.146 | — | 0.120 | 0.555 | 0.003 | 0.000 | — | 0.002 | 0.828 |
| rows-n3000-one-label | zgui | 2/2 | 0.250 | 2.33 | 0.246 | 2.32 | 215 | 167 | 0.002 | 0.000 | 0.115 | 0.002 | 0.000 | — | 0.265 | 0.091 | 0.119 |
| rows-n3000-one-label | gpui | 2/2 | 8.40 | 8.57 | 8.39 | 8.57 | 213 | 120 | 0.546 | — | 4.62 | 0.541 | 0.826 | — | 0.053 | 0.180 | 6.77 |
| rows-n3000-one-label | aizome-kit | 2/2 | 4.16 | 4.23 | 4.16 | 4.22 | 81.6 | 47 | 0.431 | — | 0.367 | 1.57 | 0.003 | 0.000 | — | 0.002 | 2.38 |
| rows-n3000-one-fill | zgui | 2/2 | 0.241 | 0.722 | 0.237 | 0.713 | 217 | 166 | 0.001 | 0.003 | 0.007 | 0.000 | 0.000 | — | 0.278 | 0.156 | 0.010 |
| rows-n3000-one-fill | gpui | 2/2 | 8.37 | 8.54 | 8.37 | 8.53 | 209 | 120 | 0.537 | — | 4.62 | 0.526 | 0.830 | — | 0.047 | 0.175 | 6.75 |
| rows-n3000-one-fill | aizome-kit | 2/2 | 4.17 | 4.22 | 4.16 | 4.22 | 79.6 | 61 | 0.437 | — | 0.323 | 1.56 | 0.002 | 0.000 | — | 0.003 | 2.32 |
| rows-n3000-one-size | zgui | 2/2 | 0.799 | 0.940 | 0.796 | 0.934 | 216 | 166 | 0.001 | 0.002 | 0.007 | 0.002 | 0.000 | — | 0.277 | 0.155 | 0.012 |
| rows-n3000-one-size | gpui | 2/2 | 8.55 | 8.80 | 8.54 | 8.79 | 212 | 120 | 0.554 | — | 4.72 | 0.527 | 0.833 | — | 0.051 | 0.179 | 6.92 |
| rows-n3000-one-size | aizome-kit | 2/2 | 4.17 | 4.21 | 4.16 | 4.20 | 82.8 | 55 | 0.435 | — | 0.324 | 1.55 | 0.004 | 0.000 | — | 0.002 | 2.32 |
| rows-n10000-one-label | zgui | 2/2 | 0.500 | 1.25 | 0.496 | 1.25 | 500 | 345 | 0.001 | 0.000 | 0.323 | 0.001 | 0.000 | — | 0.285 | 0.083 | 0.327 |
| rows-n10000-one-label | gpui | 2/2 | 29.7 | 31.2 | 29.7 | 31.2 | 277 | 189 | 1.88 | — | 16.1 | 2.20 | 3.45 | — | 0.108 | 0.192 | 23.9 |
| rows-n10000-one-label | aizome-kit | 2/2 | 170 | 174 | 170 | 174 | 708 | 117 | 1.38 | — | 159 | 8.59 | 0.005 | 0.002 | — | 0.011 | 170 |
| rows-n10000-one-fill | zgui | 2/2 | 0.600 | 0.640 | 0.597 | 0.637 | 495 | 345 | 0.001 | 0.002 | 0.005 | 0.000 | 0.000 | — | — | — | 0.007 |
| rows-n10000-one-fill | gpui | 2/2 | 29.4 | 31.1 | 29.4 | 31.1 | 274 | 189 | 1.79 | — | 16.1 | 2.17 | 3.33 | — | 0.107 | 0.188 | 23.7 |
| rows-n10000-one-fill | aizome-kit | 2/2 | 167 | 171 | 167 | 171 | 704 | 122 | 1.33 | — | 158 | 8.39 | 0.004 | 0.001 | — | 0.011 | 167 |
| rows-n10000-one-size | zgui | 2/2 | 0.211 | 2.47 | 0.205 | 2.47 | 496 | 345 | 0.001 | 0.004 | 0.007 | 0.001 | 0.000 | — | — | — | 0.013 |
| rows-n10000-one-size | gpui | 2/2 | 29.7 | 30.9 | 29.7 | 30.8 | 272 | 188 | 1.85 | — | 16.1 | 2.21 | 3.36 | — | 0.110 | 0.189 | 24.0 |
| rows-n10000-one-size | aizome-kit | 2/2 | 173 | 176 | 173 | 176 | 711 | 114 | 1.32 | — | 162 | 8.57 | 0.004 | 0.002 | — | 0.012 | 173 |

#### Warnings

- aizome-kit: per-process VRAM unavailable from DRM fdinfo and NVML (×36)
- zgui: compositor chose {(2400, 1600)} instead of requested 1200x800; use the observed size for comparisons (×36)

### `runnymede-n8192.json`

- CPU: AMD Ryzen 7 9800X3D 8-Core Processor
- GPU: 01:00.0 VGA compatible controller: NVIDIA Corporation GB202 [GeForce RTX 5090] (rev a1)
- GPU: 78:00.0 VGA compatible controller: Advanced Micro Devices, Inc. [AMD/ATI] Granite Ridge [Radeon 610M] (rev cb)
- Display server: wayland; kernel 7.3.3
- aizome-kit: 00a64e622d
- gpui-ce-vendored-from: unknown
- zgui: 082337db72

#### Headline passes

Milliseconds, over the measurement frames of every completed repetition. "First" is the first frame's time from the adapter's start; PSS is the peak, in MiB.

| scenario | framework | runs | frame p50 | frame p95 | latency p50 | latency p95 | first | PSS |
|---|---|---|---|---|---|---|---|---|
| paintupdate-n8192-flat-minimal-u10 | zgui | 1/1 | 17.9 | 19.4 | 17.9 | 19.4 | 463 | 309 |
| paintupdate-n8192-flat-minimal-u10 | gpui | 1/1 | 27.9 | 28.8 | 27.8 | 28.8 | 332 | 206 |
| paintupdate-n8192-flat-minimal-u10 | aizome-kit | 1/1 | 139 | 143 | 139 | 143 | 581 | 102 |
| layoutupdate-n8192-balanced-minimal-u10 | zgui | 1/1 | 1.21 | 6.69 | 1.20 | 6.69 | 366 | 331 |
| layoutupdate-n8192-balanced-minimal-u10 | gpui | 1/1 | 179 | 184 | 179 | 184 | 452 | 210 |
| layoutupdate-n8192-balanced-minimal-u10 | aizome-kit | 1/1 | 144 | 148 | 144 | 148 | 590 | 108 |
| textupdate-n8192-wide-minimal-u10 | zgui | 1/1 | 1.23 | 2.17 | 1.23 | 2.16 | 398 | 344 |
| textupdate-n8192-wide-minimal-u10 | gpui | 1/1 | 25.1 | 26.0 | 25.1 | 26.0 | 311 | 172 |
| textupdate-n8192-wide-minimal-u10 | aizome-kit | 1/1 | 228 | 240 | 228 | 240 | 585 | 101 |
| paintupdate-n8192-flat-minimal-u250 | zgui | 1/1 | 23.0 | 26.7 | 23.0 | 26.7 | 409 | 313 |
| paintupdate-n8192-flat-minimal-u250 | gpui | 1/1 | 27.8 | 28.7 | 27.8 | 28.7 | 291 | 206 |
| paintupdate-n8192-flat-minimal-u250 | aizome-kit | 1/1 | 142 | 144 | 142 | 144 | 586 | 96 |
| layoutupdate-n8192-balanced-minimal-u250 | zgui | 1/1 | 17.3 | 55.9 | 17.3 | 55.8 | 412 | 366 |
| layoutupdate-n8192-balanced-minimal-u250 | gpui | 1/1 | 186 | 197 | 186 | 197 | 440 | 208 |
| layoutupdate-n8192-balanced-minimal-u250 | aizome-kit | 1/1 | 142 | 146 | 142 | 146 | 599 | 104 |
| textupdate-n8192-wide-minimal-u250 | zgui | 1/1 | 20.9 | 24.2 | 20.9 | 24.2 | 450 | 369 |
| textupdate-n8192-wide-minimal-u250 | gpui | 1/1 | 42.1 | 57.6 | 42.1 | 57.6 | 308 | 177 |
| textupdate-n8192-wide-minimal-u250 | aizome-kit | 1/1 | 241 | 247 | 241 | 247 | 596 | 100 |
| paintupdate-n8192-flat-minimal-u1000 | zgui | 1/1 | 20.9 | 24.8 | 20.9 | 24.8 | 453 | 307 |
| paintupdate-n8192-flat-minimal-u1000 | gpui | 1/1 | 27.9 | 45.9 | 27.9 | 45.9 | 286 | 206 |
| paintupdate-n8192-flat-minimal-u1000 | aizome-kit | 1/1 | 143 | 145 | 143 | 145 | 586 | 94 |
| layoutupdate-n8192-balanced-minimal-u1000 | zgui | 1/1 | 68.6 | 70.6 | 68.5 | 70.6 | 422 | 383 |
| layoutupdate-n8192-balanced-minimal-u1000 | gpui | 1/1 | 177 | 199 | 177 | 199 | 436 | 200 |
| layoutupdate-n8192-balanced-minimal-u1000 | aizome-kit | 1/1 | 142 | 146 | 142 | 146 | 587 | 101 |
| textupdate-n8192-wide-minimal-u1000 | zgui | 1/1 | 83.8 | 87.2 | 83.7 | 87.2 | 474 | 386 |
| textupdate-n8192-wide-minimal-u1000 | gpui | 1/1 | 72.8 | 98.4 | 72.8 | 98.4 | 392 | 186 |
| textupdate-n8192-wide-minimal-u1000 | aizome-kit | 1/1 | 240 | 245 | 240 | 245 | 589 | 94 |
| structuralupdate-n8192-wide-minimal-u100 | zgui | 1/1 | 12.1 | 34.5 | 12.1 | 34.5 | 499 | 335 |
| structuralupdate-n8192-wide-minimal-u100 | gpui | 1/1 | 25.8 | 33.7 | 25.7 | 33.7 | 274 | 171 |
| structuralupdate-n8192-wide-minimal-u100 | aizome-kit | 1/1 | 132 | 144 | 132 | 144 | 591 | 113 |
| paintupdate-n8192-balanced-decorated-u1000 | zgui | 1/1 | 20.8 | 21.3 | 20.8 | 21.3 | 416 | 323 |
| paintupdate-n8192-balanced-decorated-u1000 | gpui | 1/1 | 180 | 200 | 180 | 200 | 477 | 200 |
| paintupdate-n8192-balanced-decorated-u1000 | aizome-kit | 0/1 | 314 | 317 | 307 | 309 | 602 | 120 |
| inputlatency-n8192-balanced-decorated-u10 | zgui | 1/1 | 0.806 | 3.64 | 0.802 | 3.64 | 420 | 321 |
| inputlatency-n8192-balanced-decorated-u10 | gpui | 1/1 | 180 | 196 | 180 | 196 | 449 | 200 |
| inputlatency-n8192-balanced-decorated-u10 | aizome-kit | 1/1 | 286 | 297 | 279 | 290 | 582 | 124 |
| resizestep-n8192-balanced-decorated-u1000 | zgui | 1/1 | 0.109 | 35.1 | 0.105 | 35.1 | 412 | 333 |
| resizestep-n8192-balanced-decorated-u1000 | gpui | 1/1 | 184 | 213 | 184 | 213 | 447 | 213 |
| resizestep-n8192-balanced-decorated-u1000 | aizome-kit | 0/1 | 336 | 521 | 328 | 514 | 585 | 142 |
| resizestorm-n8192-balanced-decorated-u1000 | zgui | 1/1 | 48.4 | 50.8 | 48.4 | 50.8 | 413 | 349 |
| resizestorm-n8192-balanced-decorated-u1000 | gpui | 1/1 | 198 | 218 | 198 | 218 | 414 | 207 |
| resizestorm-n8192-balanced-decorated-u1000 | aizome-kit | 0/1 | 330 | 333 | 322 | 326 | 578 | 1209 |

#### Diagnostic passes

Milliseconds, over the measurement frames of every completed repetition. "First" is the first frame's time from the adapter's start; PSS is the peak, in MiB.
Stages are medians of CPU time; "work" is the median of each frame's stages added up, and "gpu" is prepare, encode and submit together.

| scenario | framework | runs | frame p50 | frame p95 | latency p50 | latency p95 | first | PSS | view | style | layout | prepaint | paint | raster | gpu | present | work |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| paintupdate-n8192-flat-minimal-u10 | zgui | 1/1 | 18.1 | 20.0 | 18.1 | 20.0 | 478 | 309 | 0.051 | 0.339 | 17.3 | 0.011 | 0.001 | — | 0.252 | 0.244 | 17.7 |
| paintupdate-n8192-flat-minimal-u10 | gpui | 1/1 | 27.9 | 42.3 | 27.8 | 42.3 | 297 | 206 | 1.50 | — | 16.6 | 1.76 | 3.04 | — | 0.137 | 0.201 | 23.3 |
| paintupdate-n8192-flat-minimal-u10 | aizome-kit | 1/1 | 138 | 141 | 138 | 141 | 725 | 86 | 1.02 | — | 130 | 6.50 | 0.004 | 0.001 | — | 0.009 | 138 |
| layoutupdate-n8192-balanced-minimal-u10 | zgui | 1/1 | 1.21 | 6.54 | 1.20 | 6.53 | 370 | 330 | 0.054 | 0.125 | 0.708 | 0.022 | 0.001 | — | 0.244 | 0.239 | 0.920 |
| layoutupdate-n8192-balanced-minimal-u10 | gpui | 1/1 | 180 | 199 | 180 | 198 | 434 | 210 | 2.04 | — | 163 | 3.16 | 4.03 | — | 0.208 | 0.502 | 174 |
| layoutupdate-n8192-balanced-minimal-u10 | aizome-kit | 1/1 | 144 | 147 | 144 | 147 | 739 | 87 | 1.63 | — | 135 | 7.55 | 0.004 | 0.001 | — | 0.008 | 144 |
| textupdate-n8192-wide-minimal-u10 | zgui | 1/1 | 1.24 | 2.09 | 1.24 | 2.08 | 401 | 345 | 0.061 | 0.000 | 0.865 | 0.054 | 0.001 | — | 0.235 | 0.184 | 0.982 |
| textupdate-n8192-wide-minimal-u10 | gpui | 1/1 | 24.9 | 37.0 | 24.9 | 37.0 | 328 | 172 | 1.62 | — | 13.7 | 1.82 | 2.63 | — | 0.088 | 0.202 | 20.0 |
| textupdate-n8192-wide-minimal-u10 | aizome-kit | 1/1 | 226 | 241 | 226 | 241 | 723 | 92 | 1.43 | — | 216 | 8.55 | 0.004 | 0.001 | — | 0.011 | 227 |
| paintupdate-n8192-flat-minimal-u250 | zgui | 1/1 | 22.8 | 26.4 | 22.8 | 26.4 | 410 | 313 | 1.02 | 1.94 | 18.5 | 0.064 | 0.001 | — | 0.733 | 0.647 | 21.6 |
| paintupdate-n8192-flat-minimal-u250 | gpui | 1/1 | 27.9 | 28.7 | 27.9 | 28.7 | 324 | 206 | 1.48 | — | 16.9 | 1.73 | 2.97 | — | 0.133 | 0.186 | 23.4 |
| paintupdate-n8192-flat-minimal-u250 | aizome-kit | 1/1 | 140 | 144 | 140 | 144 | 718 | 82 | 1.05 | — | 132 | 6.64 | 0.004 | 0.001 | — | 0.013 | 140 |
| layoutupdate-n8192-balanced-minimal-u250 | zgui | 1/1 | 17.7 | 55.5 | 17.7 | 55.5 | 405 | 362 | 1.15 | 2.35 | 11.8 | 0.045 | 0.003 | — | 0.225 | 0.250 | 15.5 |
| layoutupdate-n8192-balanced-minimal-u250 | gpui | 1/1 | 186 | 198 | 186 | 198 | 434 | 208 | 2.04 | — | 169 | 3.20 | 3.94 | — | 0.213 | 0.522 | 179 |
| layoutupdate-n8192-balanced-minimal-u250 | aizome-kit | 1/1 | 144 | 146 | 144 | 146 | 743 | 93 | 1.72 | — | 134 | 8.24 | 0.015 | 0.093 | — | 0.019 | 144 |
| textupdate-n8192-wide-minimal-u250 | zgui | 1/1 | 20.8 | 24.0 | 20.8 | 24.0 | 446 | 369 | 1.49 | 0.001 | 14.5 | 1.29 | 0.001 | — | 0.232 | 0.204 | 17.4 |
| textupdate-n8192-wide-minimal-u250 | gpui | 1/1 | 42.6 | 60.5 | 42.6 | 60.5 | 310 | 177 | 1.71 | — | 29.3 | 2.24 | 2.97 | — | 0.124 | 0.217 | 36.7 |
| textupdate-n8192-wide-minimal-u250 | aizome-kit | 1/1 | 236 | 240 | 236 | 239 | 716 | 86 | 1.28 | — | 227 | 7.04 | 0.005 | 0.002 | — | 0.014 | 236 |
| paintupdate-n8192-flat-minimal-u1000 | zgui | 1/1 | 20.8 | 22.5 | 20.8 | 22.5 | 449 | 307 | 4.26 | 8.11 | 2.61 | 0.001 | 1.29 | — | 0.619 | 0.666 | 17.6 |
| paintupdate-n8192-flat-minimal-u1000 | gpui | 1/1 | 28.0 | 42.2 | 28.0 | 42.2 | 283 | 206 | 1.56 | — | 16.8 | 1.74 | 3.02 | — | 0.135 | 0.185 | 23.4 |
| paintupdate-n8192-flat-minimal-u1000 | aizome-kit | 1/1 | 142 | 144 | 142 | 144 | 731 | 82 | 1.13 | — | 134 | 7.05 | 0.101 | 0.482 | — | 0.072 | 143 |
| layoutupdate-n8192-balanced-minimal-u1000 | zgui | 1/1 | 68.8 | 72.2 | 68.8 | 72.2 | 385 | 384 | 5.24 | 9.85 | 46.8 | 0.188 | 0.216 | — | 0.273 | 0.690 | 63.1 |
| layoutupdate-n8192-balanced-minimal-u1000 | gpui | 1/1 | 178 | 193 | 178 | 193 | 447 | 200 | 1.91 | — | 162 | 2.91 | 3.97 | — | 0.196 | 0.505 | 172 |
| layoutupdate-n8192-balanced-minimal-u1000 | aizome-kit | 1/1 | 146 | 150 | 146 | 150 | 729 | 94 | 1.74 | — | 136 | 8.31 | 0.015 | 0.099 | — | 0.012 | 146 |
| textupdate-n8192-wide-minimal-u1000 | zgui | 1/1 | 84.0 | 87.8 | 84.0 | 87.8 | 473 | 387 | 5.78 | 0.002 | 60.3 | 3.38 | 0.674 | — | 0.238 | 0.492 | 71.0 |
| textupdate-n8192-wide-minimal-u1000 | gpui | 1/1 | 74.3 | 93.1 | 74.3 | 93.1 | 337 | 186 | 1.89 | — | 58.5 | 2.67 | 3.37 | — | 0.168 | 0.348 | 66.9 |
| textupdate-n8192-wide-minimal-u1000 | aizome-kit | 1/1 | 233 | 237 | 233 | 237 | 740 | 101 | 1.21 | — | 224 | 7.38 | 0.031 | 0.201 | — | 0.075 | 233 |
| structuralupdate-n8192-wide-minimal-u100 | zgui | 1/1 | 10.9 | 34.9 | 10.9 | 34.9 | 456 | 336 | 1.76 | 0.009 | 2.28 | 2.34 | 0.002 | — | — | — | 8.67 |
| structuralupdate-n8192-wide-minimal-u100 | gpui | 1/1 | 25.8 | 36.6 | 25.8 | 36.6 | 333 | 172 | 1.49 | — | 15.0 | 1.58 | 2.38 | — | 0.084 | 0.193 | 21.3 |
| structuralupdate-n8192-wide-minimal-u100 | aizome-kit | 1/1 | 132 | 141 | 132 | 141 | 719 | 93 | 1.05 | — | 123 | 6.29 | 0.003 | 0.001 | — | 0.008 | 132 |
| paintupdate-n8192-balanced-decorated-u1000 | zgui | 1/1 | 20.8 | 21.4 | 20.8 | 21.4 | 417 | 323 | 4.97 | 9.37 | 1.70 | 0.001 | 0.106 | — | 0.165 | 0.622 | 17.0 |
| paintupdate-n8192-balanced-decorated-u1000 | gpui | 1/1 | 180 | 186 | 180 | 186 | 445 | 200 | 2.16 | — | 162 | 3.42 | 4.80 | — | 0.262 | 0.918 | 174 |
| paintupdate-n8192-balanced-decorated-u1000 | aizome-kit | 1/1 | 310 | 313 | 302 | 305 | 718 | 101 | 1.43 | — | 132 | 5.74 | 7.23 | 300 | — | 1.16 | 448 |
| inputlatency-n8192-balanced-decorated-u10 | zgui | 1/1 | 0.833 | 3.56 | 0.828 | 3.55 | 451 | 318 | 0.055 | 0.122 | 0.294 | 0.016 | 0.000 | — | 0.221 | 0.569 | 0.509 |
| inputlatency-n8192-balanced-decorated-u10 | gpui | 1/1 | 180 | 187 | 180 | 187 | 464 | 199 | 2.16 | — | 162 | 3.45 | 4.74 | — | 0.268 | 0.956 | 174 |
| inputlatency-n8192-balanced-decorated-u10 | aizome-kit | 1/1 | 286 | 298 | 279 | 291 | 733 | 116 | 1.42 | — | 132 | 5.32 | 7.03 | 277 | — | 1.12 | 425 |
| resizestep-n8192-balanced-decorated-u1000 | zgui | 1/1 | 0.120 | 35.5 | 0.115 | 35.4 | 405 | 336 | 0.000 | 0.000 | 0.002 | 0.000 | 0.000 | — | 0.260 | 0.605 | 0.004 |
| resizestep-n8192-balanced-decorated-u1000 | gpui | 1/1 | 182 | 212 | 182 | 212 | 459 | 213 | 2.36 | — | 163 | 3.36 | 4.82 | — | 0.306 | 0.880 | 175 |
| resizestep-n8192-balanced-decorated-u1000 | aizome-kit | 0/1 | 335 | 520 | 328 | 513 | 715 | 143 | 1.61 | — | 131 | 6.14 | 7.31 | 324 | — | 1.29 | 472 |
| resizestorm-n8192-balanced-decorated-u1000 | zgui | 1/1 | 49.2 | 51.3 | 49.1 | 51.3 | 404 | 350 | 0.002 | 0.002 | 45.2 | 0.002 | 0.173 | — | 0.275 | 0.831 | 46.5 |
| resizestorm-n8192-balanced-decorated-u1000 | gpui | 1/1 | 199 | 205 | 199 | 205 | 454 | 212 | 2.31 | — | 166 | 3.53 | 4.73 | — | 1.56 | 0.837 | 179 |
| resizestorm-n8192-balanced-decorated-u1000 | aizome-kit | 0/1 | 329 | 334 | 321 | 326 | 590 | 1214 | 2.06 | — | 135 | 7.01 | 7.50 | 325 | — | 3.85 | 481 |

#### Warnings

- aizome-kit: compositor chose {(840, 560)} instead of requested 1200x800; use the observed size for comparisons (×12)
- aizome-kit: paintupdate-n8192-balanced-decorated-u1000 timed out (×1)
- aizome-kit: per-process VRAM unavailable from DRM fdinfo and NVML (×28)
- aizome-kit: resizestep-n8192-balanced-decorated-u1000 timed out (×2)
- aizome-kit: resizestorm-n8192-balanced-decorated-u1000 timed out (×2)
- zgui: compositor chose {(2400, 1600)} instead of requested 1200x800; use the observed size for comparisons (×24)
- zgui: compositor did not grant multiple observed window sizes; resize result is not valid for slope/latency conclusions (×4)

