//! dithered buttons: a native rune control whose background is the same
//! ordered-dither fill the charts use

use crate::rune_el::SurfaceTexture;
use dither_core::Surface;
use dither_core::bayer::{OFF_TIER, clamp01, threshold};
use dither_core::color::DitherColor;
use dither_core::paint::Variant;
use rune::tok::Theme;
use rune::{El, el, text};

/// points per dither cell
const CELL: f64 = 2.0;

#[derive(Clone, Debug)]
pub struct ButtonSpec {
    pub label: String,
    pub color: DitherColor,
    pub variant: Variant,
    pub size: (f32, f32),
    pub disabled: bool,
}

impl ButtonSpec {
    pub fn new(label: impl Into<String>, size: (f32, f32)) -> Self {
        Self {
            label: label.into(),
            color: DitherColor::Blue,
            variant: Variant::Gradient,
            size,
            disabled: false,
        }
    }

    pub fn color(mut self, color: DitherColor) -> Self {
        self.color = color;
        self
    }

    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub struct DitherButton {
    spec: ButtonSpec,
    texture: SurfaceTexture,
    cols: u32,
    rows: u32,
    hovered: bool,
    intensity: f64,
}

impl DitherButton {
    pub fn new(spec: ButtonSpec) -> Self {
        let mut button = Self {
            cols: (spec.size.0 as f64 / CELL).round().max(4.0) as u32,
            rows: (spec.size.1 as f64 / CELL).round().max(4.0) as u32,
            texture: SurfaceTexture::new(),
            hovered: false,
            intensity: 0.0,
            spec,
        };
        button.repaint();
        button
    }

    pub fn label(&self) -> &str {
        &self.spec.label
    }

    /// hover lift; the app forwards pointer enter/leave
    pub fn hover(&mut self, inside: bool) {
        self.hovered = inside;
        self.intensity = if inside { 1.0 } else { 0.0 };
        self.repaint();
    }

    /// press lift; release returns to the hover level
    pub fn press(&mut self, pressed: bool) {
        self.intensity = if pressed {
            1.5
        } else if self.hovered {
            1.0
        } else {
            0.0
        };
        self.repaint();
    }

    fn repaint(&mut self) {
        let mut surface = Surface::new(self.cols, self.rows);
        let fill = self.spec.color.fill();
        let variant = self.spec.variant;
        let intensity = self.intensity;
        let bias = if variant == Variant::Dotted {
            0.12
        } else {
            0.0
        };
        for y in 0..self.rows {
            let density = match variant {
                Variant::Gradient => 0.25 + 0.75 * ((y as f64 + 0.5) / self.rows as f64),
                Variant::Dotted => 0.5,
                _ => 0.75,
            };
            for x in 0..self.cols {
                if variant == Variant::Hatched && ((x as i32 + y as i32) & 3) >= 2 {
                    continue;
                }
                let lit = variant == Variant::Solid
                    || density > threshold(x as i32, y as i32) - 0.1 * intensity - bias;
                if variant == Variant::Dotted && !lit {
                    continue;
                }
                let k = (0.3 + density * 0.7) * (1.0 + 0.22 * intensity);
                surface.paint(
                    x as i32,
                    y as i32,
                    fill,
                    clamp01(if lit { k } else { k * OFF_TIER }),
                );
            }
        }
        let edge = clamp01(0.5 + 0.25 * intensity);
        let (cols, rows) = (self.cols as i32, self.rows as i32);
        surface.fill_rect(0, 0, cols, 1, fill, edge);
        surface.fill_rect(0, rows - 1, cols, 1, fill, edge);
        surface.fill_rect(0, 0, 1, rows, fill, edge);
        surface.fill_rect(cols - 1, 0, 1, rows, fill, edge);
        self.texture.upload_owned(surface);
    }

    pub fn view(&self, theme: &Theme) -> El {
        el().size(self.spec.size.0, self.spec.size.1)
            .radius(6.0)
            .hit()
            .focusable(!self.spec.disabled)
            .disabled(self.spec.disabled)
            .role("button")
            .label(self.spec.label.clone())
            .opacity(if self.spec.disabled { 0.4 } else { 1.0 })
            .center()
            .child(
                self.texture
                    .el()
                    .absolute(0.0, 0.0)
                    .size(self.spec.size.0, self.spec.size.1),
            )
            .child(text(self.spec.label.clone()).color(theme.text))
    }
}
