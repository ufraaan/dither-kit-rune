//! Chart data → backing surface, mirroring upstream's canvas target pass.

use dither_core::Surface;
use dither_core::color::Rgb;
use dither_core::paint::{PaintOpts, Variant, paint_column, resample};
use dither_core::scales::LinearScale;
use dither_core::stack::{StackMode, compute_bands};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeriesKind {
    #[default]
    Area,
    Line,
}

#[derive(Clone, Debug)]
pub struct Series {
    pub values: Vec<f64>,
    pub fill: Rgb,
    pub variant: Variant,
    pub kind: SeriesKind,
}

impl Series {
    pub fn new(values: impl Into<Vec<f64>>, fill: Rgb) -> Self {
        Self {
            values: values.into(),
            fill,
            variant: Variant::Gradient,
            kind: SeriesKind::Area,
        }
    }

    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    pub fn kind(mut self, kind: SeriesKind) -> Self {
        self.kind = kind;
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RenderOpts {
    pub stack: StackMode,
    /// Hover lift, 0–1.
    pub intensity: f64,
    /// Series index to emphasize; others dim to 0.3. `None` leaves all normal.
    pub emphasis: Option<usize>,
    /// Fraction of the plot already revealed left-to-right, 0–1.
    pub reveal: f64,
}

impl Default for RenderOpts {
    fn default() -> Self {
        Self {
            stack: StackMode::None,
            intensity: 0.0,
            emphasis: None,
            reveal: 1.0,
        }
    }
}

/// Paint `series` into `surface`, whose backing rows correspond to a plot
/// `plot_height` points tall. Columns run left to right; every series is
/// resampled to the surface width.
pub fn render(surface: &mut Surface, plot_height: f64, series: &[Series], opts: &RenderOpts) {
    if series.is_empty() || surface.cols() == 0 || surface.rows() == 0 {
        return;
    }
    let rows = surface.rows() as f64;
    let cols = surface.cols();
    let h = if plot_height > 0.0 { plot_height } else { 1.0 };
    let values: Vec<Vec<f64>> = series.iter().map(|s| s.values.clone()).collect();
    let bands = compute_bands(&values, opts.stack);
    let y = LinearScale::new(bands.min, bands.max, h);
    let stacked = opts.stack.is_stacked();
    let glow = (rows * 0.16).round().max(6.0);
    let reveal_cols = (opts.reveal * cols as f64).ceil();

    for (i, spec) in series.iter().enumerate() {
        let Some(band) = bands.bands.get(i) else {
            continue;
        };
        let is_line = spec.kind == SeriesKind::Line;
        let top: Vec<f64> = band.iter().map(|b| y.y(b[1]) / h * (rows - 1.0)).collect();
        let floor: Vec<f64> = band
            .iter()
            .enumerate()
            .map(|(row, b)| {
                if is_line {
                    (top[row] + glow).min(rows - 1.0)
                } else {
                    y.y(b[0]) / h * (rows - 1.0)
                }
            })
            .collect();
        let top = resample(&top, cols as usize);
        let floor = resample(&floor, cols as usize);
        let dim = match opts.emphasis {
            Some(emphasized) if emphasized != i => 0.3,
            _ => 1.0,
        };
        let sparse = if stacked { 0.0 } else { i as f64 * 0.14 };
        let paint = PaintOpts {
            variant: spec.variant,
            intensity: opts.intensity,
            dim,
            stacked: stacked && !is_line,
            sparse,
        };
        for (x, (&a, &b)) in top.iter().zip(&floor).enumerate() {
            if x as f64 > reveal_cols {
                break;
            }
            paint_column(surface, x as i32, a.min(b), a.max(b), spec.fill, &paint);
        }
    }
}
