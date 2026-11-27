//! The column painter: one low-resolution column from the value line to its floor.

use crate::Surface;
use crate::bayer::{BORDER_ALPHA, OFF_TIER, clamp01, threshold};
use crate::color::Rgb;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    #[default]
    Gradient,
    Dotted,
    Hatched,
    Solid,
}

impl Variant {
    pub const fn name(self) -> &'static str {
        match self {
            Variant::Gradient => "gradient",
            Variant::Dotted => "dotted",
            Variant::Hatched => "hatched",
            Variant::Solid => "solid",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "gradient" => Some(Variant::Gradient),
            "dotted" => Some(Variant::Dotted),
            "hatched" => Some(Variant::Hatched),
            "solid" => Some(Variant::Solid),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PaintOpts {
    pub variant: Variant,
    /// Hover lift, 0–1.
    pub intensity: f64,
    /// Selection dim multiplier: 0.3 dimmed, 1 normal.
    pub dim: f64,
    /// Denser, with a solid floor, when layers stack.
    pub stacked: bool,
    /// Raise the dither threshold (thin the fill out) for front layers.
    pub sparse: f64,
}

impl Default for PaintOpts {
    fn default() -> Self {
        Self { variant: Variant::Gradient, intensity: 0.0, dim: 1.0, stacked: false, sparse: 0.0 }
    }
}

impl PaintOpts {
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    pub fn intensity(mut self, intensity: f64) -> Self {
        self.intensity = intensity;
        self
    }

    pub fn dim(mut self, dim: f64) -> Self {
        self.dim = dim;
        self
    }

    pub fn stacked(mut self, stacked: bool) -> Self {
        self.stacked = stacked;
        self
    }

    pub fn sparse(mut self, sparse: f64) -> Self {
        self.sparse = sparse;
        self
    }
}

/// Fill column `x` from `top` down to `floor`: dense at the floor, dissolving
/// upward toward the value line, capped by a soft border outline.
pub fn paint_column(
    surface: &mut Surface,
    x: i32,
    top: f64,
    floor: f64,
    fill: Rgb,
    opts: &PaintOpts,
) {
    let t = crate::js_round(top) as i32;
    let f = crate::js_round(floor) as i32;
    let depth = f - t;
    if depth <= 0 {
        surface.paint(x, t, fill, BORDER_ALPHA * opts.dim);
        return;
    }
    let bias = if opts.variant == Variant::Dotted { 0.12 } else { 0.0 }
        + if opts.stacked { 0.2 } else { 0.0 }
        - opts.sparse;
    for y in t..f {
        let mut density = (y - t) as f64 / depth as f64;
        if opts.stacked {
            density = 0.5 + 0.5 * density;
        }
        if opts.variant == Variant::Hatched && ((x + y) & 3) >= 2 {
            continue;
        }
        let lit = opts.variant == Variant::Solid
            || density > threshold(x, y) - 0.1 * opts.intensity - bias;
        if opts.variant == Variant::Dotted && !lit {
            continue;
        }
        let k = (0.3 + density * 0.7) * (1.0 + 0.22 * opts.intensity);
        let alpha = clamp01((if lit { k } else { k * OFF_TIER }) * opts.dim);
        surface.paint(x, y, fill, alpha);
    }
    surface.paint(x, t, fill, BORDER_ALPHA * opts.dim);
    if depth > 1 {
        surface.paint(x, t + 1, fill, BORDER_ALPHA * 0.5 * opts.dim);
    }
}

/// Linear-resample a per-index fraction array to `cols` columns.
pub fn resample(src: &[f64], cols: usize) -> Vec<f64> {
    let mut out = vec![0.0; cols];
    let last = (src.len() as f64 - 1.0).max(1.0);
    for (c, slot) in out.iter_mut().enumerate() {
        let t = (c as f64 / (cols as f64 - 1.0).max(1.0)) * last;
        let i = t.floor();
        let f = t - i;
        let i = i as usize;
        let a = src.get(i).copied().unwrap_or(0.0);
        let b = src.get((i + 1).min(src.len().saturating_sub(1))).copied().unwrap_or(a);
        *slot = a + (b - a) * f;
    }
    out
}
