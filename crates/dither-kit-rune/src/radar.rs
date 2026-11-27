//! radar charts: polygons over evenly-spaced spokes, dithered by distance to
//! the polygon edge

use crate::chart::LegendEntry;
use crate::rune_el::SurfaceTexture;
use crate::util::{format_tick, to_color};
use dither_core::Surface;
use dither_core::bayer::{OFF_TIER, clamp01, ease_in_out_cubic, threshold};
use dither_core::color::{DitherColor, Rgb};
use dither_core::paint::Variant;
use dither_core::polar::{
    RadarAxis, axis_at_angle, dist_to_polygon_edge, point_in_polygon, polar_x, polar_y, radar_axes,
    radar_ring,
};
use rune::tok::{TextStyle, Theme};
use rune::{El, el, text};

const AXIS: TextStyle = TextStyle::new(10.0, 500.0).mono();
const TIP: TextStyle = TextStyle::new(11.0, 500.0).mono();
const FRAME: Rgb = Rgb::new(132, 132, 142);

#[derive(Clone, Debug)]
pub struct RadarSeries {
    pub name: String,
    pub label: String,
    pub color: DitherColor,
    pub variant: Variant,
}

impl RadarSeries {
    pub fn new(name: impl Into<String>, color: DitherColor) -> Self {
        let name = name.into();
        Self {
            label: name.clone(),
            name,
            color,
            variant: Variant::Gradient,
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
pub struct RadarSpec {
    pub key: String,
    pub size: (f32, f32),
    pub axes: Vec<String>,
    pub series: Vec<RadarSeries>,
    /// series-major values, `values[series][axis]`
    pub values: Vec<Vec<f64>>,
    pub animate: bool,
    pub duration: f64,
    pub frame: bool,
    pub tooltip: bool,
}

impl RadarSpec {
    pub fn new(
        size: (f32, f32),
        axes: Vec<String>,
        series: Vec<RadarSeries>,
        values: Vec<Vec<f64>>,
    ) -> Self {
        Self {
            key: "dither-radar".to_string(),
            size,
            axes,
            series,
            values,
            animate: true,
            duration: 0.9,
            frame: true,
            tooltip: true,
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }

    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }
}

pub struct Radar {
    spec: RadarSpec,
    axes: Vec<RadarAxis>,
    max: f64,
    texture: SurfaceTexture,
    cols: u32,
    rows: u32,
    center: (f64, f64),
    outer: f64,
    progress: f64,
    started: Option<f64>,
    hover: Option<usize>,
    selected: Option<usize>,
    cursor: (f32, f32),
}

impl Radar {
    pub fn new(mut spec: RadarSpec) -> Self {
        let count = spec.axes.len();
        spec.values.resize(spec.series.len(), Vec::new());
        for values in &mut spec.values {
            values.resize(count, 0.0);
        }
        let max = spec
            .values
            .iter()
            .flatten()
            .filter(|v| v.is_finite())
            .fold(0.0_f64, |acc, v| acc.max(*v));
        let max = if max <= 0.0 { 1.0 } else { max };
        let (cols, rows) = dither_core::bayer::backing_size(spec.size.0 as f64, spec.size.1 as f64);
        let plot = (spec.size.0 as f64, spec.size.1 as f64);
        let mut radar = Self {
            axes: radar_axes(&spec.axes),
            max,
            texture: SurfaceTexture::new(),
            cols,
            rows,
            center: (plot.0 / 2.0, plot.1 / 2.0),
            outer: (plot.0.min(plot.1) / 2.0 - 20.0).max(0.0),
            progress: if spec.animate { 0.0 } else { 1.0 },
            started: None,
            hover: None,
            selected: None,
            cursor: (0.0, 0.0),
            spec,
        };
        radar.repaint();
        radar
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

    pub fn hover_at(&mut self, x: f32, y: f32) {
        self.cursor = (x, y);
        let dx = x as f64 - self.center.0;
        let dy = y as f64 - self.center.1;
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        let index = axis_at_angle(&self.axes, dy.atan2(dx));
        if self.hover != Some(index) {
            self.hover = Some(index);
            self.repaint();
        }
    }

    pub fn leave(&mut self) {
        if self.hover.take().is_some() {
            self.repaint();
        }
    }

    pub fn toggle_series(&mut self, index: usize) {
        self.selected = if self.selected == Some(index) {
            None
        } else {
            Some(index)
        };
        self.repaint();
    }

    pub fn legend(&self) -> Vec<LegendEntry> {
        self.spec
            .series
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

    fn polygons(&self, prog: f64) -> Vec<Vec<f64>> {
        self.spec
            .values
            .iter()
            .map(|series| {
                let mut poly = Vec::with_capacity(self.axes.len() * 2);
                for (i, axis) in self.axes.iter().enumerate() {
                    let value = series.get(i).copied().unwrap_or(0.0);
                    let r = (value / self.max) * self.outer * prog;
                    poly.push(polar_x(self.center.0, r, axis.angle));
                    poly.push(polar_y(self.center.1, r, axis.angle));
                }
                poly
            })
            .collect()
    }

    fn repaint(&mut self) {
        let mut surface = Surface::new(self.cols, self.rows);
        let width = self.spec.size.0 as f64;
        let height = self.spec.size.1 as f64;
        let progress = ease_in_out_cubic(self.progress);
        let fx = self.cols as f64 / width.max(1.0);
        let fy = self.rows as f64 / height.max(1.0);

        if self.spec.frame {
            for level in 1..=4 {
                let ring = radar_ring(
                    &self.axes,
                    self.center.0,
                    self.center.1,
                    self.outer * level as f64 / 4.0,
                );
                for i in 0..self.axes.len() {
                    let j = (i + 1) % self.axes.len();
                    surface.line(
                        ring[i * 2] * fx,
                        ring[i * 2 + 1] * fy,
                        ring[j * 2] * fx,
                        ring[j * 2 + 1] * fy,
                        FRAME,
                        0.14,
                    );
                }
            }
            for axis in &self.axes {
                surface.line(
                    self.center.0 * fx,
                    self.center.1 * fy,
                    polar_x(self.center.0, self.outer, axis.angle) * fx,
                    polar_y(self.center.1, self.outer, axis.angle) * fy,
                    FRAME,
                    0.14,
                );
            }
        }

        let polys = self.polygons(progress);
        let intensity = if self.hover.is_some() { 1.0 } else { 0.0 };
        let band = (self.outer * 0.45).max(1.0);
        let emphasis = self.selected;

        for y in 0..self.rows {
            let py = (y as f64 + 0.5) * height / self.rows as f64;
            for x in 0..self.cols {
                let px = (x as f64 + 0.5) * width / self.cols as f64;
                let mut covered = false;
                for (pi, poly) in polys.iter().enumerate() {
                    if !point_in_polygon(px, py, poly) {
                        continue;
                    }
                    let spec = &self.spec.series[pi];
                    let dim = match emphasis {
                        Some(selected) if selected != pi => 0.3,
                        _ => 1.0,
                    };
                    let dist = dist_to_polygon_edge(px, py, poly);
                    if dist < 1.4 {
                        surface.paint(x as i32, y as i32, spec.color.fill(), dim);
                        covered = true;
                        continue;
                    }
                    let density = 1.0 - (dist / band).min(1.0);
                    let bias = if spec.variant == Variant::Dotted {
                        0.12
                    } else {
                        0.0
                    };
                    let sparse = pi as f64 * 0.2;
                    if spec.variant == Variant::Hatched && ((x as i32 + y as i32) & 3) >= 2 {
                        continue;
                    }
                    let lit = spec.variant == Variant::Solid
                        || density
                            > threshold(x as i32, y as i32) - 0.1 * intensity - bias + sparse;
                    if !lit && (spec.variant == Variant::Dotted || covered) {
                        continue;
                    }
                    let k = (0.32 + density * 0.68) * (1.0 + 0.22 * intensity);
                    let alpha = clamp01((if lit { k } else { k * OFF_TIER }) * dim);
                    surface.paint(x as i32, y as i32, spec.color.fill(), alpha);
                    covered = true;
                }
            }
        }

        for (pi, poly) in polys.iter().enumerate() {
            let spec = &self.spec.series[pi];
            let dim = match emphasis {
                Some(selected) if selected != pi => 0.3,
                _ => 1.0,
            };
            for i in 0..self.axes.len() {
                let bx = (poly[i * 2] * fx).round() as i32;
                let by = (poly[i * 2 + 1] * fy).round() as i32;
                let size = if self.hover == Some(i) { 3 } else { 1 };
                surface.fill_rect(
                    bx - (size - 1),
                    by - (size - 1),
                    size * 2 - 1,
                    size * 2 - 1,
                    spec.color.fill(),
                    dim,
                );
            }
        }

        self.texture.upload_owned(surface);
    }

    pub fn view(&self, theme: &Theme) -> El {
        let (w, h) = self.spec.size;
        let mut root = el()
            .key(self.spec.key.clone())
            .size(w, h)
            .role("img")
            .label("Radar chart")
            .focusable(true)
            .child(self.texture.el().absolute(0.0, 0.0).size(w, h));

        for (i, axis) in self.axes.iter().enumerate() {
            let lx = polar_x(self.center.0, self.outer + 12.0, axis.angle) as f32;
            let ly = polar_y(self.center.1, self.outer + 12.0, axis.angle) as f32;
            let cos = axis.angle.cos();
            let hot = self.hover == Some(i);
            let color = if hot {
                theme.text
            } else {
                theme.text_secondary
            };
            let label = text(axis.label.clone()).font(AXIS).color(color);
            let slot = if cos.abs() < 0.3 {
                el().absolute(lx - 40.0, ly - 7.0).size(80.0, 14.0).center()
            } else if cos > 0.0 {
                el().absolute(lx + 4.0, ly - 7.0)
                    .size(80.0, 14.0)
                    .center_y()
            } else {
                el().absolute(lx - 84.0, ly - 7.0)
                    .size(80.0, 14.0)
                    .justify_end()
                    .center_y()
            };
            root = root.child(slot.child(label));
        }

        if let (true, Some(index)) = (self.spec.tooltip, self.hover) {
            root = root.child(self.tooltip(index, theme));
        }
        root
    }

    fn tooltip(&self, index: usize, theme: &Theme) -> El {
        let width = 140.0;
        let height = 20.0 + self.spec.series.len() as f32 * 15.0 + 6.0;
        let left = (self.cursor.0 + 12.0).clamp(4.0, self.spec.size.0 - width - 4.0);
        let top = (self.cursor.1 - height - 8.0).clamp(4.0, self.spec.size.1 - height - 4.0);
        let axis = &self.axes[index];

        let mut body = rune::col().gap(2.0).child(
            text(axis.label.clone())
                .font(AXIS)
                .color(theme.text_secondary),
        );
        for (si, series) in self.spec.series.iter().enumerate() {
            let value = self
                .spec
                .values
                .get(si)
                .and_then(|v| v.get(index))
                .copied()
                .unwrap_or(0.0);
            let dim = self.selected.is_some() && self.selected != Some(si);
            body = body.child(
                rune::row()
                    .gap(5.0)
                    .opacity(if dim { 0.4 } else { 1.0 })
                    .child(
                        el().size(7.0, 7.0)
                            .radius(1.0)
                            .fill(to_color(series.color.fill())),
                    )
                    .child(
                        text(series.label.clone())
                            .font(TIP)
                            .color(theme.text_secondary),
                    )
                    .child(text(format_tick(value)).font(TIP).color(theme.text)),
            );
        }

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
