# dither-kit-rune

A native Rust port of [Dither Kit](https://github.com/sortedcord/dither-kit) for the
[Rune](https://github.com/anomalyco/rune) UI framework. Unofficial; see [NOTICE.md](NOTICE.md).

macOS is the only supported target for now.

## Layout

| Path | What it is |
| --- | --- |
| `crates/dither-core` | Deterministic engine: palette, Bayer painter, scales, stacking, RGBA surface. No UI dependency. |
| `crates/dither-kit-rune` | Rune integration: data → surface, surface → nearest-neighbor `Pixels`. |
| `examples/gallery` | Deterministic scene; renders headless to PNG. |
| `crates/dither-core/tests/fixtures` | Byte-for-byte painter fixtures generated from the pinned upstream TypeScript. |

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
- Deterministic headless scenes: `--snap out.png --at 2` captures the settled
  frame, `--at 0.32` a mid-entrance one, `--scene light` the light theme.

Bloom is configured in `dither_core::bloom` but not yet drawn as a Rune layer.

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
