//! Dither Kit for Rune.
//!
//! The crate is layered: [`dither_core`] holds deterministic math and painting
//! with no UI dependency, [`cartesian`] turns chart data into a backing surface,
//! and [`rune_el`] hands that surface to Rune as a nearest-neighbor `Pixels`
//! element. Parsing, layout and interaction sit on top.

pub use dither_core as core;

pub mod cartesian;
pub mod rune_el;

pub use cartesian::{Series, SeriesKind};
pub use core::color::{DitherColor, Rgb, Seed};
pub use core::paint::{PaintOpts, Variant};
pub use rune_el::SurfaceTexture;
