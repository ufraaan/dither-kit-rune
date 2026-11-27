# dither-kit-rune

A native Rust port of [Dither Kit](https://github.com/Boring-Software-Inc/dither-kit) for the
[Rune](https://github.com/anomalyco/rune) UI framework. Unofficial; see [NOTICE.md](NOTICE.md).

Build dithered charts and interface components in Rune with deterministic rendering, native input, light and dark themes, and reduced-motion support.

[View the component gallery](https://dither-rune.ufraan.dev)

## Components

- Area and line charts
- Grouped and stacked bar charts
- Pie and donut charts
- Radar charts
- Sparklines
- Deterministic mirrored avatars
- Dithered buttons
- Directional gradient washes

Charts support axes, grids, reference lines, tooltips, keyboard inspection, clickable legends, and entrance animations.

## Try the gallery

Rune currently requires Rust 1.99. Clone the repository and open the native gallery:

```sh
git clone https://github.com/ufraaan/dither-kit-rune.git
cd dither-kit-rune
cargo run -p gallery
```

The gallery opens with a dashboard. Use the arrow keys to browse individual components.

## Add a component

The included `dither` command lists the available starters and adds one to a Rune project:

```sh
cargo install --path crates/dither-cli
dither list
dither add area-chart --dir ../your_rune_app
```

Add `dither-kit-rune` and `dither-core` as Git or path dependencies. Copy the Rune revision and `[patch.crates-io]` entries from this repository's [`Cargo.toml`](Cargo.toml) so every Rune dependency uses the same revision.

## Platform support

macOS is the only supported target today. Bloom can be configured, but Rune does not draw the bloom layer yet.

## License

Licensed under the [MIT License](LICENSE). This project includes an independent port of Dither Kit; see [NOTICE.md](NOTICE.md) for attribution.
