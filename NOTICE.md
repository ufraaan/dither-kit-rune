# Notices and attribution

This project is an unofficial Rust port of [Dither Kit](https://github.com/Boring-Software-Inc/dither-kit)
by its upstream authors, targeted at the [Rune](https://github.com/anomalyco/rune) UI framework.
It is not affiliated with or endorsed by either project.

- Dither Kit is MIT licensed. Its palette, Bayer matrix, column painter, scale and stacking
  behavior, avatar seeding and bloom presets are reproduced here.
- Dither Kit credits [Evil Charts](https://evilcharts.com) as an inspiration; that credit is
  carried forward.
- Rune is MIT licensed and used as a pinned dependency. No Rune code is vendored in this
  repository; Cargo fetches the pinned revision listed in the workspace `Cargo.toml`.

Pinned revisions:

| Upstream | Revision |
| --- | --- |
| Dither Kit | `5bd036b437895f0adf1038e6ed025906a911bdc4` |
| Rune | `dc64314b816afdd77689209f14647407034940cb` |

Fixture regeneration reads the pinned Dither Kit sources from a temporary checkout:
`node scripts/gen-fixtures.mjs <checkout>/registry/dither-kit`.
