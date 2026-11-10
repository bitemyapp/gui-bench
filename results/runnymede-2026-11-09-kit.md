# runnymede, 2026-11-09, evening: aizome-kit before and after Sekka PR 373

The scenes where aizome-kit did worst in `runnymede-2026-11-09.md`, measured again on the same desktop between 20:32 and 20:43: once with the kit as it was on Sekka's `main` (`a98a4ac7`), once with the branch of [bitemyapp/sekka#373](https://github.com/bitemyapp/sekka/pull/373) (`f89eb41d`). GPUI-CE's and zgui's numbers are the afternoon's, from `runnymede-2026-11-09.md`; neither changed.

All times are milliseconds: the headline passes' median frame interval, then the diagnostic passes' median CPU work for a frame. A frame of 4.16 means the work fit in one of the display's (240 Hz). Rows and 1,024- and 64-node scenes are two repetitions, 8,192-node scenes one.

## Two corrections to the afternoon's kit numbers

**Items out of sight had no height.** The adapter put its tree straight into a container. A kit column gives a child that doesn't fit the room that is left, which past the window's bottom is none, so in the wide, balanced and deep scenes every item below the window was laid out with no height, where the other two adapters' keep theirs. In the decorated scenes each of those still drew a shadow at the window's bottom edge: thousands of them at 8,192 nodes. The adapter lays its content out as tall as it comes now.

That changes the afternoon's decorated numbers and nothing else:

| Scene | Afternoon | Same kit, content laid out whole |
|---|---|---|
| 64 decorated, every fill | 8.78 / 8.45 | 8.29 / 7.99 |
| 1,024 decorated, every fill | 48.2 / 48.3 | 12.4 / 12.6 |
| 8,192 decorated, every fill | 314 (timed out) / 448 | 144 / 159 |
| 1,000 rows, one label | 4.17 / 0.894 | 4.13 / 0.893 |
| 10,000 rows, one label | 167 / 170 | 172 / 174 |
| 1,024 text, every label | 30.3 / 29.5 | 29.8 / 29.9 |
| 8,192 layout, every node | 142 / 146 | 143 / 145 |

So the afternoon's "soft shadows cost 47 ms at 1,024 cards" was mostly that mistake: laid out as the others lay it out, the unchanged kit took 12.4 ms there, as GPUI-CE did.

**The window's size.** Aizome keeps a window's geometry by its app id and takes the binary's name for a program that gives none, so the afternoon's fix (no app id) fixed nothing: a run after a resize scene still opened at the size that scene ended on. The adapter asks for the scenario's size before a run begins now, and starts when the window has it. Every run below was at 1200 × 800.

## One row of a live column changes (`rows`)

| Rows, change | kit before | kit after | GPUI-CE | zgui |
|---|---|---|---|---|
| 1,000, label | 4.13 / 0.89 | 4.15 / 0.55 | 4.16 / 2.44 | 0.16 / 0.06 |
| 1,000, fill | 4.15 / 0.85 | 4.16 / 0.53 | 4.16 / 2.42 | 0.13 / 0.01 |
| 1,000, size | 4.16 / 0.85 | 4.17 / 0.53 | 4.16 / 2.43 | 0.32 / 0.01 |
| 3,000, label | 4.16 / 2.35 | 4.16 / 1.55 | 8.38 / 6.77 | 0.24 / 0.12 |
| 3,000, fill | 4.16 / 2.34 | 4.16 / 1.51 | 8.40 / 6.75 | 0.23 / 0.01 |
| 3,000, size | 4.17 / 2.33 | 4.17 / 1.53 | 8.43 / 6.92 | 0.80 / 0.01 |
| 10,000, label | 172 / 174 | 4.96 / 4.87 | 29.6 / 23.9 | 0.49 / 0.33 |
| 10,000, fill | 170 / 166 | 4.91 / 4.92 | 29.8 / 23.7 | 0.60 / 0.01 |
| 10,000, size | 170 / 170 | 4.96 / 4.84 | 29.6 / 24.0 | 0.17 / 0.01 |

## 1,024 and 64 nodes

| Scene | kit before | kit after | GPUI-CE | zgui |
|---|---|---|---|---|
| 1,024 text, wide, 1% | 4.12 / 1.24 | 4.16 / 0.76 | 4.17 / 2.65 | 0.24 / 0.16 |
| 1,024 text, wide, 25% | 7.91 / 8.18 | 4.16 / 1.39 | 5.49 / 4.71 | 2.79 / 1.88 |
| 1,024 text, wide, 100% | 29.8 / 29.9 | 4.16 / 3.67 | 8.82 / 8.03 | 11.8 / 7.79 |
| 1,024 decorated, 100% | 12.4 / 12.6 | 6.59 / 6.84 | 12.4 / 11.6 | 8.33 / 2.90 |
| 64 decorated, 100% | 8.29 / 7.99 | 5.27 / 4.90 | 4.16 / 0.90 | 8.33 / 0.89 |

## 8,192 nodes

"Kit before" is the evening's run for the three scenes marked †, and the afternoon's for the rest: the flat scenes never had the layout mistake, and the plain wide and balanced ones measure the same with and without it (above). The afternoon's decorated latency and resize numbers were for the mistaken layout and are left out.

| Scene | kit before | kit after | GPUI-CE | zgui |
|---|---|---|---|---|
| Paint, flat, 1% | 139 / 138 | 4.16 / 3.88 | 27.9 / 23.3 | 17.9 / 17.7 |
| Layout, balanced, 1% | 144 / 144 | 4.55 / 4.51 | 179 / 174 | 1.21 / 0.92 |
| Text, wide, 1% | 228 / 227 | 4.45 / 4.49 | 25.1 / 20.0 | 1.23 / 0.98 |
| Paint, flat, 25% | 142 / 140 | 4.16 / 4.10 | 27.8 / 23.4 | 23.0 / 21.6 |
| Layout, balanced, 25% | 142 / 144 | 4.62 / 4.67 | 186 / 179 | 17.3 / 15.5 |
| Text, wide, 25% | 241 / 236 | 10.8 / 10.7 | 42.1 / 36.7 | 20.9 / 17.4 |
| Paint, flat, 100% | 143 / 143 | 5.34 / 6.07 | 27.9 / 23.4 | 20.9 / 17.6 |
| Layout, balanced, 100% † | 143 / 145 | 4.55 / 4.69 | 177 / 172 | 68.6 / 63.1 |
| Text, wide, 100% † | 240 / 236 | 29.2 / 29.7 | 72.8 / 66.9 | 83.8 / 71.0 |
| Structural, wide, 10% | 132 / 132 | 4.17 / 3.98 | 25.8 / 21.3 | 12.1 / 8.67 |
| Paint, decorated, 100% † | 144 / 159 | 7.29 / 11.4 | 180 / 174 | 20.8 / 17.0 |
| Input latency, decorated, 1% | | 4.41 / 4.40 | 180 / 174 | 0.81 / 0.51 |
| Resize step, decorated | | 7.80 / 11.7 | 184 / 175 | did not resize |
| Resize storm, decorated | | 8.93 / 10.6 | 198 / 179 | 48.4 / 46.5 |

## First frame and memory

The kit's "before" here is the afternoon's.

| Scene | | kit before | kit after | GPUI-CE | zgui |
|---|---|---|---|---|---|
| 1,000 rows | first frame, ms | 39 | 27 | 199 | 162 |
| 3,000 rows | first frame, ms | 81 | 36 | 219 | 218 |
| 10,000 rows | first frame, ms | 710 | 72 | 274 | 497 |
| 8,192 nodes, text | first frame, ms | 589 | 61 | 392 | 474 |
| 10,000 rows | peak PSS, MiB | 116 | 91 | 189 | 344 |
| 8,192 nodes, text | peak PSS, MiB | 94 | 105 | 186 | 386 |
| 8,192 nodes, resize storm | peak PSS, MiB | 1,209 | 286 | 207 | 349 |

## How to run them

```
cargo run -p gui-bench -- run --diagnostic --framework aizome-kit --repetitions 2 --preset rows
cargo run -p gui-bench -- run --diagnostic --framework aizome-kit --repetitions 2 --preset standard --scenario textupdate-n1024
cargo run -p gui-bench -- run --diagnostic --framework aizome-kit --repetitions 1 --preset standard --scenario n8192-
```

with `../sekka` on the revision to measure. The raw documents (`runnymede-base-*.json`, `runnymede-next-*.json`) are not in the repository.
