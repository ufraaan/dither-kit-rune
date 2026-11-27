//! Dither Kit for Rune.
//!
//! The crate is layered: [`dither_core`] holds deterministic math and painting
//! with no UI dependency, [`cartesian`] turns chart data into a backing surface,
//! and [`rune_el`] hands that surface to Rune as a nearest-neighbor `Pixels`
//! element. Parsing, layout and interaction sit on top.

pub use dither_core as core;

pub mod avatar;
pub mod button;
pub mod cartesian;
pub mod chart;
pub mod gradient;
pub mod pie;
pub mod radar;
pub mod rune_el;
pub mod util;

pub use avatar::Avatar;
pub use button::{ButtonSpec, DitherButton};
pub use cartesian::{ChartKind, Series, SeriesKind};
pub use chart::{Chart, ChartSeries, ChartSpec, LegendEntry, Margins};
pub use core::color::{DitherColor, Rgb, Seed};
pub use core::paint::{PaintOpts, Variant};
pub use gradient::DitherGradient;
pub use pie::{Pie, PieSeries, PieSpec};
pub use radar::{Radar, RadarSeries, RadarSpec};
pub use rune_el::SurfaceTexture;
