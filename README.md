# dither-kit-rune

A native Rust port of [Dither Kit](https://github.com/Boring-Software-Inc/dither-kit) for the
[Rune](https://github.com/anomalyco/rune) UI framework. Unofficial; see [NOTICE.md](NOTICE.md).

macOS is the only supported target for now.

## Layout

| Path | What it is |
| --- | --- |
| `crates/dither-core` | Deterministic engine: palette, Bayer painter, scales, stacking, RGBA surface. No UI dependency. |
| `crates/dither-kit-rune` | Rune integration: data → surface, surface → nearest-neighbor `Pixels`. |
| `examples/gallery` | Deterministic scenes; renders headless to PNG. |
| `crates/dither-cli` | `dither` — lists registry items, scaffolds a starter module, emits the registry JSON. |
| `registry.json` + `registry/` | Component manifest and starter snippets, served as JSON from `www/r/`. |
| `crates/dither-core/tests/fixtures` | Byte-for-byte painter fixtures generated from the pinned upstream TypeScript. |
| `www/` | Landing page, WebP captures and the built registry. |

## What works

- Area, line and bar charts on a shared chart model (`ChartSpec` / `Chart`).
- Horizontal grid, x/y axes, reference line, hover crosshair and dot markers.
- Donut/pie charts with per-pixel radial dither, slice hover pop and tooltips.
- Radar charts with a ring/spoke frame, polygon dither and vertex markers.
- Deterministic mirrored avatars, dithered buttons, directional gradient washes.
- Sparklines: the same chart with `ChartSpec::sparkline()`.
- Pointer hover, arrow-key inspection, Escape to clear, clickable legends that
  isolate a series, light/dark themes.
- Entrances: left-to-right reveal, staggered bars, clockwise pie sweep, radar
  scale-in and Bayer-ordered avatar materialization.
- Reduced motion settles every entrance without animating.
- Scene switching: the dashboard is the default; arrow keys (or `--scene`) open
  one component per scene for screenshots.
- Deterministic headless scenes: `--snap out.png --at 2` captures the settled
  frame, `--at 0.32` a mid-entrance one, `--scene light` the light theme.

Scenes: `dashboard`, `area`, `bar`, `sparkline`, `donut`, `radar`, `avatars`,
`buttons`, `gradient`.

The landing page in [`www/`](www/index.html) uses 2× WebP captures from
[`www/shots/`](www/shots), dark and light. `DITHER_BARE=1` drops the scene
chrome, `DITHER_LIGHT=1` renders the light theme, and `--size` sets the canvas:

```sh
DITHER_BARE=1 cargo run -p gallery -- --snap /tmp/area.png --at 2 \
  --scene area --size 960x640
cwebp -q 86 /tmp/area.png -o www/shots/area-dark.webp
```

Bloom is configured in `dither_core::bloom` but not yet drawn as a Rune layer.

## Install and registry

```sh
cargo install --path crates/dither-cli
dither list                 # components in registry.json
dither add area-chart       # writes dither/area_chart.rs and prints the dependency
dither prompt               # the agent setup prompt
dither build --out www/r    # regenerate the registry JSON the site serves
```

`dither add` scaffolds a starter module; it never vendors the engine. Add
`dither-kit-rune` and `dither-core` as path or git dependencies, and copy Rune's
`[patch.crates-io]` block along with the pinned revision.

## Commands

```sh
cargo test -p dither-core          # primitives and painter parity, no GPU
cargo test -p gallery              # scene builds and paints
cargo run -p gallery               # open the window
cargo run -p gallery -- --snap gallery.png   # headless render
```

Rune is pinned to a `dev` revision in the workspace `Cargo.toml`, together with the
`[patch.crates-io]` block Rune requires. Bump both together.

## Fidelity

The painter reproduces upstream's alpha-tier model: every pixel is the series' single fill
color and only its alpha varies, so the same surface reads correctly on light and dark
backgrounds. Fixtures compare raw backing buffers before Rune scaling, so text and GPU
differences never hide an engine change.

`scripts/gen-fixtures.mjs` regenerates those buffers from the pinned Dither Kit checkout.
