# dither-kit-rune

A native Rust port of [Dither Kit](https://github.com/Boring-Software-Inc/dither-kit) for the
[Rune](https://github.com/anomalyco/rune) UI framework. Ordered-dither charts, generative
avatars, dithered buttons and gradient washes from one deterministic engine.

This is Rune-specific and macOS-only for now. The pure engine lives in `dither-core` with no UI
dependency. The Rune integration and typed builders live in `dither-kit-rune`. Rune is pinned to `dc64314b816afdd77689209f14647407034940cb`
and the workspace carries its `[patch.crates-io]` block.

## Components

| Item | Description |
| --- | --- |
| area-chart | Two-series dithered area chart with a reference line and hover scrubbing. |
| bar-chart | Grouped or stacked bars with a staggered entrance and category hover. |
| sparkline | One series, no chrome: the same chart with axes and tooltip off. |
| donut | Pie or donut with per-pixel radial dither, hover pop and slice tooltips. |
| radar | Polygons over evenly-spaced spokes, dithered by distance to the edge. |
| avatar | Deterministic mirrored 8x8 glyph seeded by a name. |
| button | Native focusable control with a dithered fill that lifts on hover. |
| gradient | Quantized directional wash, one colour to another or to transparent. |

## Install

```sh
cargo install --path crates/dither-cli
dither add area-chart      # writes dither/area_chart.rs and prints the dependency
dither list
dither prompt              # the agent setup prompt
```

Workspace gallery:

```sh
cargo run -p gallery
cargo run -p gallery -- --scene donut
cargo run -p gallery -- --snap out.png --at 2 --scene area
```

## Registry

- `/r/index.json` — items, version, licence, source
- `/r/<item>.json` — per item: description, source paths, install lines and starter snippet
- `/r/prompt.txt` — the agent setup prompt
- `/llms.txt` — this site's agent index

## Options

- variant: `"gradient" | "dotted" | "hatched" | "solid"`
- stack: `"default" | "stacked" | "percent"`
- colours: green, blue, purple, pink, orange, red, grey
- animate: entrance and replay, settles under reduced motion
- interactive: `false` for a decorative spark, no crosshair or tooltip

## Licence

MIT. Unofficial port; see the repository `NOTICE.md` for attribution.
