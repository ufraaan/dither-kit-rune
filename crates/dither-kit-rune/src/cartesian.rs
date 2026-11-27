//! Chart data → backing surface, mirroring upstream's canvas target passes.

use dither_core::Surface;
use dither_core::bayer::{CELL, clamp01, ease_out_cubic};
use dither_core::color::Rgb;
use dither_core::paint::{PaintOpts, Variant, paint_column, resample};
use dither_core::scales::{BandScale, LinearScale, PointScale};
use dither_core::stack::{StackMode, compute_bands};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartKind {
    #[default]
    Area,
    Line,
    Bar,
}

impl ChartKind {
    pub const fn is_bar(self) -> bool {
        matches!(self, ChartKind::Bar)
    }
}

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
    pub kind: ChartKind,
    pub stack: StackMode,
    /// Hover lift, 0–1.
    pub intensity: f64,
    /// Series index to emphasize; others dim to 0.3.
    pub emphasis: Option<usize>,
    /// Fraction of the plot revealed left-to-right, 0–1 (continuous charts).
    pub reveal: f64,
    /// Entrance progress, 0–1 (bar stagger).
    pub progress: f64,
    /// Hovered category index; other categories dim in bar charts.
    pub hover: Option<usize>,
}

impl Default for RenderOpts {
    fn default() -> Self {
        Self {
            kind: ChartKind::Area,
            stack: StackMode::None,
            intensity: 0.0,
            emphasis: None,
            reveal: 1.0,
            progress: 1.0,
            hover: None,
        }
    }
}

/// Fraction of the timeline spent staggering bar starts.
pub const STAGGER: f64 = 0.55;

/// Paint `series` into `surface`, whose backing rows correspond to a plot
/// `plot` points wide and tall.
pub fn render(surface: &mut Surface, plot: (f64, f64), series: &[Series], opts: &RenderOpts) {
    if series.is_empty() || surface.cols() == 0 || surface.rows() == 0 {
        return;
    }
    let values: Vec<Vec<f64>> = series.iter().map(|s| s.values.clone()).collect();
    let bands = compute_bands(&values, opts.stack);
    let y = LinearScale::new(bands.min, bands.max, plot.1);
    if opts.kind.is_bar() {
        render_bars(surface, plot, series, &bands, &y, opts);
    } else {
        render_continuous(surface, plot, series, &bands, &y, opts);
    }
}

fn render_continuous(
    surface: &mut Surface,
    plot: (f64, f64),
    series: &[Series],
    bands: &dither_core::stack::Bands,
    y: &LinearScale,
    opts: &RenderOpts,
) {
    let rows = surface.rows() as f64;
    let cols = surface.cols();
    let h = if plot.1 > 0.0 { plot.1 } else { 1.0 };
    let stacked = opts.stack.is_stacked();
    let glow = (rows * 0.16).round().max(6.0);
    let reveal_cols = (opts.reveal * cols as f64).ceil();

    for (i, spec) in series.iter().enumerate() {
        let Some(band) = bands.bands.get(i) else {
            continue;
        };
        let is_line = spec.kind == SeriesKind::Line || opts.kind == ChartKind::Line;
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

/// Eased grow factor for bar `i` at global progress `prog`.
pub fn bar_progress(i: usize, len: usize, prog: f64) -> f64 {
    let start = if len > 1 {
        i as f64 / (len - 1) as f64 * STAGGER
    } else {
        0.0
    };
    ease_out_cubic(clamp01((prog - start) / (1.0 - STAGGER)))
}

/// Horizontal `(x, width)` of one bar in plot points.
pub fn bar_slot(
    band: &BandScale,
    i: usize,
    series_index: usize,
    series_count: usize,
    stacked: bool,
) -> (f64, f64) {
    let center = band.position(i) + band.bandwidth() / 2.0;
    if stacked {
        let width = band.bandwidth() * 0.9;
        (center - width / 2.0, width)
    } else {
        let slot = band.bandwidth() / series_count.max(1) as f64;
        (
            center - band.bandwidth() / 2.0 + series_index as f64 * slot + slot * 0.08,
            slot * 0.84,
        )
    }
}

fn render_bars(
    surface: &mut Surface,
    plot: (f64, f64),
    series: &[Series],
    bands: &dither_core::stack::Bands,
    y: &LinearScale,
    opts: &RenderOpts,
) {
    let rows = surface.rows() as f64;
    let cols = surface.cols() as f64;
    let h = if plot.1 > 0.0 { plot.1 } else { 1.0 };
    let len = bands.bands.first().map(Vec::len).unwrap_or(0);
    if len == 0 {
        return;
    }
    let band = BandScale::new(len, plot.0);
    let fx = cols / plot.0.max(1.0);
    let stacked = opts.stack.is_stacked();

    for (si, spec) in series.iter().enumerate() {
        let Some(series_band) = bands.bands.get(si) else {
            continue;
        };
        let selection_dim = match opts.emphasis {
            Some(emphasized) if emphasized != si => 0.3,
            _ => 1.0,
        };
        for (i, b) in series_band.iter().enumerate() {
            let base = y.y(b[0]) / h * (rows - 1.0);
            let value = y.y(b[1]) / h * (rows - 1.0);
            let grown = base + (value - base) * bar_progress(i, len, opts.progress);
            let hover_dim = if opts.hover.is_some() && opts.hover != Some(i) {
                0.5
            } else {
                1.0
            };
            let paint = PaintOpts {
                variant: spec.variant,
                intensity: opts.intensity,
                dim: selection_dim * hover_dim,
                stacked,
                sparse: 0.0,
            };
            let (x, width) = bar_slot(&band, i, si, series.len(), stacked);
            let c0 = (x * fx).round() as i32;
            let c1 = ((x + width) * fx).round() as i32;
            for col in c0..c1 {
                paint_column(
                    surface,
                    col,
                    grown.min(base),
                    grown.max(base),
                    spec.fill,
                    &paint,
                );
            }
        }
    }
}

/// Backing geometry for a plot rect in points.
pub fn backing_size(plot_width: f64, plot_height: f64) -> (u32, u32) {
    dither_core::bayer::backing_size(plot_width, plot_height)
}

/// Points per backing cell.
pub const CELL_SIZE: f64 = CELL;

/// Point and band scales for a plot, shared by painting and hit testing.
pub fn x_scales(len: usize, plot_width: f64) -> (PointScale, BandScale) {
    (
        PointScale::new(len, plot_width),
        BandScale::new(len, plot_width),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bars_split_a_band_when_grouped_and_share_it_when_stacked() {
        let band = BandScale::new(4, 400.0);
        let (x0, w0) = bar_slot(&band, 0, 0, 2, false);
        let (x1, _) = bar_slot(&band, 0, 1, 2, false);
        assert!((x1 - x0 - band.bandwidth() / 2.0).abs() < 1e-9);
        assert!(w0 < band.bandwidth() / 2.0 + 1e-9);

        let (_, sw) = bar_slot(&band, 0, 0, 2, true);
        assert!(sw > w0 * 2.0);
    }

    #[test]
    fn bar_entrance_staggers_left_to_right() {
        assert_eq!(bar_progress(0, 5, 1.0), 1.0);
        assert_eq!(bar_progress(4, 5, 0.0), 0.0);
        assert!(bar_progress(0, 5, 0.55) > bar_progress(4, 5, 0.55));
    }
}
