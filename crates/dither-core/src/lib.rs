//! The deterministic dither engine shared by every Dither Kit component.
//!
//! Nothing here knows about Rune, windows or themes: input is data, output is
//! an RGBA buffer, and the same input always produces the same bytes.
//!
//! All math is `f64` and rounding follows JavaScript's `Math.round` (half up,
//! toward positive infinity) because the engine is a faithful port of the
//! upstream canvas implementation.

pub mod bayer;
pub mod bloom;
pub mod color;
pub mod paint;
pub mod rand;
pub mod scales;
pub mod stack;

/// JavaScript `Math.round`: halves round toward positive infinity.
pub fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// A straight-alpha RGBA surface, the port's replacement for a canvas.
///
/// Pixels are composited source-over as they are painted, exactly like
/// repeated `fillRect` calls on a transparent canvas. The buffer is handed to
/// Rune through `Pixels::set`, which takes straight-alpha sRGB.
#[derive(Clone, Debug)]
pub struct Surface {
    cols: u32,
    rows: u32,
    rgba: Vec<u8>,
}

impl Surface {
    pub fn new(cols: u32, rows: u32) -> Self {
        Self { cols, rows, rgba: vec![0; cols as usize * rows as usize * 4] }
    }

    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn rows(&self) -> u32 {
        self.rows
    }

    pub fn size(&self) -> (u32, u32) {
        (self.cols, self.rows)
    }

    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    pub fn into_rgba(self) -> Vec<u8> {
        self.rgba
    }

    pub fn clear(&mut self) {
        self.rgba.fill(0);
    }

    /// Source-over composite of `color` at `alpha` over one pixel.
    pub fn paint(&mut self, x: i32, y: i32, color: color::Rgb, alpha: f64) {
        if alpha <= 0.0 || x < 0 || y < 0 || x >= self.cols as i32 || y >= self.rows as i32 {
            return;
        }
        let sa = alpha.min(1.0);
        let i = (y as usize * self.cols as usize + x as usize) * 4;
        let da = self.rgba[i + 3] as f64 / 255.0;
        let out_a = sa + da * (1.0 - sa);
        if out_a <= 0.0 {
            self.rgba[i..i + 4].fill(0);
            return;
        }
        let over = |src: u8, dst: u8| -> u8 {
            let v = (src as f64 * sa + dst as f64 * da * (1.0 - sa)) / out_a;
            js_round(v).clamp(0.0, 255.0) as u8
        };
        self.rgba[i] = over(color.r, self.rgba[i]);
        self.rgba[i + 1] = over(color.g, self.rgba[i + 1]);
        self.rgba[i + 2] = over(color.b, self.rgba[i + 2]);
        self.rgba[i + 3] = js_round(out_a * 255.0).clamp(0.0, 255.0) as u8;
    }

    /// Fill an axis-aligned rectangle in surface pixel space, clipped.
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: color::Rgb, alpha: f64) {
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.cols as i32).max(0);
        let y1 = (y + h).min(self.rows as i32).max(0);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        for py in y0..y1 {
            for px in x0..x1 {
                self.paint(px, py, color, alpha);
            }
        }
    }
}
