//! pie and donut charts: per-pixel radial dither inside slice angles

use crate::rune_el::SurfaceTexture;
use crate::util::{format_tick, to_color};
use dither_core::Surface;
use dither_core::bayer::{OFF_TIER, clamp01, ease_in_out_cubic, threshold};
use dither_core::color::DitherColor;
use dither_core::paint::Variant;
use dither_core::polar::PieSlice;
use dither_core::polar::{TAU, TOP, pie_slices, polar_x, polar_y, slice_at_angle};
use rune::tok::{TextStyle, Theme};
use rune::{El, el, text};

use crate::chart::LegendEntry;

const AXIS: TextStyle = TextStyle::new(10.0, 500.0).mono();
const TIP: TextStyle = TextStyle::new(11.0, 500.0).mono();

/// points the hovered slice bulges outward
const POP: f64 = 6.0;
/// gap between the plot edge and the rim
const PAD: f64 = 6.0;

#[derive(Clone, Debug)]
pub struct PieSeries {
    pub name: String,
    pub label: String,
    pub color: DitherColor,
    pub variant: Variant,
    pub value: f64,
}

impl PieSeries {
    pub fn new(name: impl Into<String>, value: f64, color: DitherColor) -> Self {
        let name = name.into();
        Self {
            label: name.clone(),
            name,
            color,
            variant: Variant::Gradient,
            value,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }
}

#[derive(Clone, Debug)]
pub struct PieSpec {
    pub key: String,
    pub size: (f32, f32),
    pub slices: Vec<PieSeries>,
    /// inner radius as a fraction of the outer radius; 0 for a pie
    pub inner: f64,
    pub animate: bool,
    pub duration: f64,
    pub tooltip: bool,
}

impl PieSpec {
    pub fn new(size: (f32, f32), slices: Vec<PieSeries>) -> Self {
        Self {
            key: "dither-pie".to_string(),
            size,
            slices,
            inner: 0.0,
            animate: true,
            duration: 0.9,
            tooltip: true,
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }

    pub fn inner(mut self, inner: f64) -> Self {
        self.inner = inner.clamp(0.0, 0.95);
        self
    }

    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }
}

pub struct Pie {
    spec: PieSpec,
    slices: Vec<PieSlice>,
    texture: SurfaceTexture,
    cols: u32,
    rows: u32,
    center: (f64, f64),
    outer: f64,
    inner: f64,
    progress: f64,
    started: Option<f64>,
    hover: Option<usize>,
    selected: Option<usize>,
    cursor: (f32, f32),
}

impl Pie {
    pub fn new(spec: PieSpec) -> Self {
        let items: Vec<(String, f64)> = spec
            .slices
            .iter()
            .map(|s| (s.name.clone(), s.value))
            .collect();
        let (cols, rows) = dither_core::bayer::backing_size(spec.size.0 as f64, spec.size.1 as f64);
        let plot = (spec.size.0 as f64, spec.size.1 as f64);
        let outer = (plot.0.min(plot.1) / 2.0 - PAD).max(0.0);
        let mut pie = Self {
            slices: pie_slices(&items),
            texture: SurfaceTexture::new(),
            cols,
            rows,
            center: (plot.0 / 2.0, plot.1 / 2.0),
            outer,
            inner: outer * spec.inner,
            progress: if spec.animate { 0.0 } else { 1.0 },
            started: None,
            hover: None,
            selected: None,
            cursor: (0.0, 0.0),
            spec,
        };
        pie.repaint();
        pie
    }

    pub fn hover(&self) -> Option<usize> {
        self.hover
    }

    pub fn progress(&self) -> f64 {
        self.progress
    }

    pub fn texture(&self) -> &SurfaceTexture {
        &self.texture
    }

    pub fn tick(&mut self, clock: f64) -> bool {
        if !self.spec.animate || self.progress >= 1.0 {
            return false;
        }
        let start = *self.started.get_or_insert(clock);
        let progress = clamp01((clock - start) / self.spec.duration.max(0.001));
        if progress != self.progress {
            self.progress = progress;
            self.repaint();
        }
        progress < 1.0
    }

    /// settle the entrance without animating
    pub fn finish(&mut self) -> bool {
        if self.progress < 1.0 {
            self.progress = 1.0;
            self.repaint();
            true
        } else {
            false
        }
    }

    pub fn replay(&mut self) {
        if self.spec.animate {
            self.progress = 0.0;
            self.started = None;
            self.repaint();
        }
    }

    pub fn clear_selection(&mut self) {
        if self.selected.take().is_some() {
            self.repaint();
        }
    }

    fn index_at(&self, x: f32, y: f32) -> Option<usize> {
        let dx = x as f64 - self.center.0;
        let dy = y as f64 - self.center.1;
        let r = (dx * dx + dy * dy).sqrt();
        if r > self.outer || r < self.inner {
            return None;
        }
        slice_at_angle(&self.slices, dy.atan2(dx))
    }

    pub fn hover_at(&mut self, x: f32, y: f32) {
        self.cursor = (x, y);
        let index = self.index_at(x, y);
        if self.hover != index {
            self.hover = index;
            self.repaint();
        }
    }

    pub fn leave(&mut self) {
        if self.hover.take().is_some() {
            self.repaint();
        }
    }

    pub fn toggle_slice(&mut self, index: usize) {
        self.selected = if self.selected == Some(index) {
            None
        } else {
            Some(index)
        };
        self.repaint();
    }

    pub fn legend(&self) -> Vec<LegendEntry> {
        self.spec
            .slices
            .iter()
            .enumerate()
            .map(|(index, series)| LegendEntry {
                index,
                label: series.label.clone(),
                color: series.color.fill(),
                selected: self.selected == Some(index),
                dimmed: self.selected.is_some() && self.selected != Some(index),
            })
            .collect()
    }

    fn repaint(&mut self) {
        let mut surface = Surface::new(self.cols, self.rows);
        let width = self.spec.size.0 as f64;
        let height = self.spec.size.1 as f64;
        let reveal = TOP + ease_in_out_cubic(self.progress) * TAU;
        let emphasis = self.selected;
        let hover = self.hover;

        for y in 0..self.rows {
            let py = (y as f64 + 0.5) * height / self.rows as f64;
            for x in 0..self.cols {
                let px = (x as f64 + 0.5) * width / self.cols as f64;
                let dx = px - self.center.0;
                let dy = py - self.center.1;
                let r = (dx * dx + dy * dy).sqrt();
                if r < self.inner {
                    continue;
                }
                let angle = dy.atan2(dx);
                let mut na = angle;
                while na < TOP {
                    na += TAU;
                }
                while na >= TOP + TAU {
                    na -= TAU;
                }
                if na > reveal {
                    continue;
                }
                let Some(index) = slice_at_angle(&self.slices, angle) else {
                    continue;
                };
                let spec = &self.spec.slices[index];
                let active = hover == Some(index);
                let local_outer = if active { self.outer + POP } else { self.outer };
                if r > local_outer {
                    continue;
                }
                let dim = match emphasis {
                    Some(selected) if selected != index => 0.3,
                    _ => 1.0,
                };
                let intensity = if hover.is_some() { 1.0 } else { 0.0 };
                let fill = spec.color.fill();

                if local_outer - r < if active { 2.4 } else { 1.4 } {
                    surface.paint(x as i32, y as i32, fill, dim);
                    continue;
                }
                let density = (r - self.inner) / (local_outer - self.inner).max(1.0);
                let bias = if spec.variant == Variant::Dotted {
                    0.12
                } else {
                    0.0
                };
                if spec.variant == Variant::Hatched && ((x as i32 + y as i32) & 3) >= 2 {
                    continue;
                }
                let lit = spec.variant == Variant::Solid
                    || density > threshold(x as i32, y as i32) - 0.1 * intensity - bias;
                if spec.variant == Variant::Dotted && !lit {
                    continue;
                }
                let k = (0.35 + density * 0.65) * (1.0 + 0.22 * intensity);
                let alpha = clamp01((if lit { k } else { k * OFF_TIER }) * dim);
                surface.paint(x as i32, y as i32, fill, alpha);
            }
        }
        self.texture.upload_owned(surface);
    }

    pub fn view(&self, theme: &Theme) -> El {
        let mut root = el()
            .key(self.spec.key.clone())
            .size(self.spec.size.0, self.spec.size.1)
            .role("img")
            .label("Donut chart")
            .focusable(true)
            .child(
                self.texture
                    .el()
                    .absolute(0.0, 0.0)
                    .size(self.spec.size.0, self.spec.size.1),
            );

        if let (true, Some(index)) = (self.spec.tooltip, self.hover) {
            root = root.child(self.tooltip(index, theme));
        }
        root
    }

    fn tooltip(&self, index: usize, theme: &Theme) -> El {
        let slice = &self.spec.slices[index];
        let width = 136.0;
        let height = 44.0;
        let left = (self.cursor.0 + 12.0).clamp(4.0, self.spec.size.0 - width - 4.0);
        let top = (self.cursor.1 - height - 8.0).clamp(4.0, self.spec.size.1 - height - 4.0);
        let total: f64 = self.slices.iter().map(|s| s.value).sum();
        let share = if total > 0.0 {
            slice.value / total * 100.0
        } else {
            0.0
        };

        let body = rune::col()
            .gap(3.0)
            .child(
                text(slice.name.clone())
                    .font(AXIS)
                    .color(theme.text_secondary),
            )
            .child(
                rune::row()
                    .gap(5.0)
                    .child(
                        el().size(7.0, 7.0)
                            .radius(1.0)
                            .fill(to_color(slice.color.fill())),
                    )
                    .child(
                        text(slice.label.clone())
                            .font(TIP)
                            .color(theme.text_secondary),
                    )
                    .child(
                        text(format!("{} · {:.0}%", format_tick(slice.value), share))
                            .font(TIP)
                            .color(theme.text),
                    ),
            );

        el().key(format!("{}-tooltip", self.spec.key))
            .absolute(left, top)
            .size(width, height)
            .radius(6.0)
            .fill(theme.card)
            .border(1.0, theme.card_border)
            .padding(6.0)
            .child(body)
    }
}

/// the center of a slice rim, for hosts that place their own labels
pub fn slice_rim(slice: &PieSlice, center: (f64, f64), radius: f64) -> (f64, f64) {
    (
        polar_x(center.0, radius, slice.mid),
        polar_y(center.1, radius, slice.mid),
    )
}

/// a value as a percent of the stack total, matching the slice geometry
pub fn share(value: f64, total: f64) -> f64 {
    if total == 0.0 { 0.0 } else { value / total }
}
