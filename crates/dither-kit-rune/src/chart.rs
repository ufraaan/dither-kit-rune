//! A self-contained interactive cartesian chart: surface, chrome and state.

use crate::cartesian::{ChartKind, RenderOpts, Series, backing_size, render, x_scales};
use crate::rune_el::SurfaceTexture;
use crate::util::{format_tick, to_color};
use dither_core::Surface;
use dither_core::bayer::{clamp01, ease_in_out_cubic};
use dither_core::color::{DitherColor, Rgb};
use dither_core::paint::Variant;
use dither_core::scales::{
    BandScale, LinearScale, PointScale, index_at_band, nearest_index, ticks,
};
use dither_core::stack::{Bands, StackMode, compute_bands};
use rune::tok::{TextStyle, Theme};
use rune::{El, el, text};

const AXIS: TextStyle = TextStyle::new(10.0, 500.0).mono();
const TIP: TextStyle = TextStyle::new(11.0, 500.0).mono();

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Margins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            top: 10.0,
            right: 12.0,
            bottom: 22.0,
            left: 36.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChartSeries {
    pub name: String,
    pub label: String,
    pub color: DitherColor,
    pub variant: Variant,
}

impl ChartSeries {
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
pub struct ChartSpec {
    pub key: String,
    pub size: (f32, f32),
    pub categories: Vec<String>,
    pub series: Vec<ChartSeries>,
    /// Series-major values: `values[series][category]`.
    pub values: Vec<Vec<f64>>,
    pub kind: ChartKind,
    pub stack: StackMode,
    pub margins: Margins,
    pub animate: bool,
    pub duration: f64,
    pub interactive: bool,
    pub grid: bool,
    pub axes: bool,
    pub tooltip: bool,
    pub reference: Option<f64>,
    pub y_ticks: usize,
    pub x_max_ticks: usize,
}

impl ChartSpec {
    pub fn new(
        size: (f32, f32),
        categories: Vec<String>,
        series: Vec<ChartSeries>,
        values: Vec<Vec<f64>>,
    ) -> Self {
        Self {
            key: "dither-chart".to_string(),
            size,
            categories,
            series,
            values,
            kind: ChartKind::Area,
            stack: StackMode::None,
            margins: Margins::default(),
            animate: true,
            duration: 0.9,
            interactive: true,
            grid: true,
            axes: true,
            tooltip: true,
            reference: None,
            y_ticks: 4,
            x_max_ticks: 8,
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }

    pub fn kind(mut self, kind: ChartKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn stack(mut self, stack: StackMode) -> Self {
        self.stack = stack;
        self
    }

    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    pub fn margins(mut self, margins: Margins) -> Self {
        self.margins = margins;
        self
    }

    pub fn chrome(mut self, grid: bool, axes: bool, tooltip: bool) -> Self {
        self.grid = grid;
        self.axes = axes;
        self.tooltip = tooltip;
        self
    }

    pub fn reference(mut self, value: f64) -> Self {
        self.reference = Some(value);
        self
    }

    /// decorative sparkline: no chrome, no tooltip, no entrance
    pub fn sparkline(mut self) -> Self {
        self.margins = Margins {
            top: 3.0,
            right: 0.0,
            bottom: 3.0,
            left: 0.0,
        };
        self.grid = false;
        self.axes = false;
        self.tooltip = false;
        self.animate = false;
        self
    }
}

#[derive(Clone, Debug)]
pub struct LegendEntry {
    pub index: usize,
    pub label: String,
    pub color: Rgb,
    pub selected: bool,
    pub dimmed: bool,
}

pub struct Chart {
    spec: ChartSpec,
    engine: Vec<Series>,
    texture: SurfaceTexture,
    cols: u32,
    rows: u32,
    plot: (f64, f64),
    bands: Bands,
    y: LinearScale,
    point: PointScale,
    band: BandScale,
    progress: f64,
    started: Option<f64>,
    hover: Option<usize>,
    selected: Option<usize>,
}

impl Chart {
    pub fn new(mut spec: ChartSpec) -> Self {
        let count = spec.categories.len();
        spec.values.resize(spec.series.len(), Vec::new());
        for values in &mut spec.values {
            values.resize(count, 0.0);
        }
        let plot = (
            (spec.size.0 - spec.margins.left - spec.margins.right).max(0.0) as f64,
            (spec.size.1 - spec.margins.top - spec.margins.bottom).max(0.0) as f64,
        );
        let (cols, rows) = backing_size(plot.0, plot.1);
        let engine: Vec<Series> = spec
            .series
            .iter()
            .zip(&spec.values)
            .map(|(s, values)| Series::new(values.clone(), s.color.fill()).variant(s.variant))
            .collect();
        let bands = compute_bands(
            &engine.iter().map(|s| s.values.clone()).collect::<Vec<_>>(),
            spec.stack,
        );
        let y = LinearScale::new(bands.min, bands.max, plot.1);
        let (point, band) = x_scales(count, plot.0);
        let mut chart = Self {
            progress: if spec.animate { 0.0 } else { 1.0 },
            spec,
            engine,
            texture: SurfaceTexture::new(),
            cols,
            rows,
            plot,
            bands,
            y,
            point,
            band,
            started: None,
            hover: None,
            selected: None,
        };
        chart.repaint();
        chart
    }

    pub fn spec(&self) -> &ChartSpec {
        &self.spec
    }

    /// The shared Rune image, if a host wants to place the surface itself.
    pub fn texture(&self) -> &SurfaceTexture {
        &self.texture
    }

    pub fn hover(&self) -> Option<usize> {
        self.hover
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn progress(&self) -> f64 {
        self.progress
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

    /// play the entrance again
    pub fn replay(&mut self) {
        if self.spec.animate {
            self.progress = 0.0;
            self.started = None;
            self.repaint();
        }
    }

    /// Advance the entrance animation. Returns true while more frames are due.
    pub fn tick(&mut self, clock: f64) -> bool {
        if !self.spec.animate || self.progress >= 1.0 {
            return false;
        }
        let start = *self.started.get_or_insert(clock);
        let duration = self.spec.duration.max(0.001);
        let progress = clamp01((clock - start) / duration);
        if progress != self.progress {
            self.progress = progress;
            self.repaint();
        }
        progress < 1.0
    }

    /// Pointer moved to `x` in chart-root points (margins included).
    pub fn hover_at(&mut self, x: f32) {
        if !self.spec.interactive || self.spec.categories.is_empty() {
            return;
        }
        let px = x as f64 - self.spec.margins.left as f64;
        if px < -8.0 || px > self.plot.0 + 8.0 {
            self.leave();
            return;
        }
        let len = self.spec.categories.len();
        let index = if self.spec.kind.is_bar() {
            index_at_band(px, len, self.plot.0)
        } else {
            nearest_index(px, len, self.plot.0)
        };
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

    /// Move the keyboard hover by `delta` categories.
    pub fn step_hover(&mut self, delta: i32) {
        if !self.spec.interactive || self.spec.categories.is_empty() {
            return;
        }
        let last = self.spec.categories.len() - 1;
        let next = match self.hover {
            Some(current) => (current as i32 + delta).clamp(0, last as i32) as usize,
            None => {
                if delta < 0 {
                    last
                } else {
                    0
                }
            }
        };
        if self.hover != Some(next) {
            self.hover = Some(next);
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

    pub fn clear_selection(&mut self) {
        if self.selected.take().is_some() {
            self.repaint();
        }
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

    fn repaint(&mut self) {
        let mut surface = Surface::new(self.cols, self.rows);
        let reveal = if self.spec.animate {
            ease_in_out_cubic(self.progress)
        } else {
            1.0
        };
        let opts = RenderOpts {
            kind: self.spec.kind,
            stack: self.spec.stack,
            intensity: if self.hover.is_some() { 1.0 } else { 0.0 },
            emphasis: self.selected,
            reveal,
            progress: self.progress,
            hover: self.hover,
        };
        render(&mut surface, self.plot, &self.engine, &opts);
        self.texture.upload_owned(surface);
    }

    fn x_center(&self, index: usize) -> f64 {
        if self.spec.kind.is_bar() {
            self.band.position(index) + self.band.bandwidth() / 2.0
        } else {
            self.point.position(index)
        }
    }

    fn value_y(&self, series: usize, index: usize) -> Option<f64> {
        let band = self.bands.bands.get(series)?.get(index)?;
        Some(self.y.y(band[1]))
    }

    /// The full visual: grid, surface, chrome, hover marks and tooltip. The
    /// caller adds pointer and key handlers and composes its own legend.
    pub fn view(&self, theme: &Theme) -> El {
        let m = self.spec.margins;
        let (plot_w, plot_h) = (self.plot.0 as f32, self.plot.1 as f32);
        let mut root = el()
            .key(self.spec.key.clone())
            .size(self.spec.size.0, self.spec.size.1)
            .role("img")
            .label(format!("{} chart", chart_label(self.spec.kind)))
            .focusable(true);

        if self.spec.grid {
            for tick in self.ticks() {
                let y = m.top + self.y.y(tick) as f32;
                root = root.child(
                    el().absolute(m.left, y)
                        .size(plot_w, 1.0)
                        .fill(theme.separator)
                        .opacity(0.7),
                );
            }
        }

        if let Some(reference) = self.spec.reference {
            let y = m.top + self.y.y(reference) as f32;
            root = root.child(
                el().absolute(m.left, y)
                    .size(plot_w, 1.0)
                    .fill(theme.indicator)
                    .opacity(0.5),
            );
        }

        root = root.child(
            self.texture
                .el()
                .absolute(m.left, m.top)
                .size(plot_w, plot_h),
        );

        if self.spec.axes {
            root = self.x_labels(root, theme);
            root = self.y_labels(root, theme);
        }

        if let Some(index) = self.hover {
            let x = m.left + self.x_center(index) as f32;
            if self.spec.interactive {
                root = root.child(
                    el().absolute(x, m.top)
                        .size(1.0, plot_h)
                        .fill(theme.text_secondary)
                        .opacity(0.35),
                );
            }
            for (si, series) in self.spec.series.iter().enumerate() {
                if let Some(y) = self.value_y(si, index) {
                    let dim = self.selected.is_some() && self.selected != Some(si);
                    root = root.child(
                        el().absolute(x - 2.0, m.top + y as f32 - 2.0)
                            .size(4.0, 4.0)
                            .fill(to_color(series.color.fill()))
                            .opacity(if dim { 0.3 } else { 1.0 }),
                    );
                }
            }
            if self.spec.tooltip {
                root = root.child(self.tooltip(index, x, theme));
            }
        }

        root
    }

    fn ticks(&self) -> Vec<f64> {
        let mut ticks = ticks(self.y.domain().0, self.y.domain().1, self.spec.y_ticks);
        if ticks.len() > 12 {
            ticks.truncate(12);
        }
        ticks
    }

    fn x_labels(&self, mut root: El, theme: &Theme) -> El {
        let m = self.spec.margins;
        let len = self.spec.categories.len();
        if len == 0 {
            return root;
        }
        let step = (len as f64 / self.spec.x_max_ticks.max(1) as f64)
            .ceil()
            .max(1.0) as usize;
        for (i, label) in self.spec.categories.iter().enumerate() {
            if i % step != 0 {
                continue;
            }
            let x = m.left + self.x_center(i) as f32 - 34.0;
            root = root.child(
                el().absolute(x, m.top + self.plot.1 as f32 + 6.0)
                    .size(68.0, 14.0)
                    .center()
                    .child(text(label.clone()).font(AXIS).color(theme.text_secondary)),
            );
        }
        root
    }

    fn y_labels(&self, mut root: El, theme: &Theme) -> El {
        let m = self.spec.margins;
        for tick in self.ticks() {
            let y = m.top + self.y.y(tick) as f32 - 7.0;
            root = root.child(
                el().absolute(0.0, y)
                    .size(m.left - 6.0, 14.0)
                    .justify_end()
                    .center_y()
                    .child(
                        text(format_tick(tick))
                            .font(AXIS)
                            .color(theme.text_secondary),
                    ),
            );
        }
        root
    }

    fn tooltip(&self, index: usize, x: f32, theme: &Theme) -> El {
        let m = self.spec.margins;
        let width = 132.0;
        let rows = self.spec.series.len() as f32;
        let height = 20.0 + rows * 15.0 + 6.0;
        let top_value = (0..self.spec.series.len())
            .filter_map(|si| self.value_y(si, index))
            .fold(f64::INFINITY, f64::min);
        let anchor = if top_value.is_finite() {
            m.top + top_value as f32
        } else {
            m.top + 40.0
        };
        let left = (x - width / 2.0).clamp(4.0, self.spec.size.0 - width - 4.0);
        let top = (anchor - height - 8.0).max(4.0);

        let mut body = rune::col().gap(2.0).child(
            text(self.spec.categories[index].clone())
                .font(AXIS)
                .color(theme.text_secondary),
        );
        for (si, series) in self.spec.series.iter().enumerate() {
            let value = self
                .bands
                .bands
                .get(si)
                .and_then(|b| b.get(index))
                .map(|b| b[1])
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

fn chart_label(kind: ChartKind) -> &'static str {
    match kind {
        ChartKind::Area => "Area",
        ChartKind::Line => "Line",
        ChartKind::Bar => "Bar",
    }
}
