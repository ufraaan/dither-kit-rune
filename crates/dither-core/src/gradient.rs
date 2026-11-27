//! Dithered gradient washes: the quantized directional field upstream builds
//! with an SVG filter chain, rasterized directly.

use crate::{Surface, js_round};

use crate::bayer::raw;
use crate::color::Rgb;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Degrees(f64),
}

impl Direction {
    /// CSS-style angle where 0° runs right and 90° runs down.
    pub fn angle(self) -> f64 {
        match self {
            Direction::Right => 0.0,
            Direction::Down => 90.0,
            Direction::Left => 180.0,
            Direction::Up => 270.0,
            Direction::Degrees(deg) => deg,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientSpec {
    pub from: Rgb,
    pub to: Option<Rgb>,
    pub direction: Direction,
    /// Points per dither cell.
    pub cell: f64,
    pub opacity: f64,
    /// Quantization steps; the upstream default is 6.
    pub levels: u32,
}

impl GradientSpec {
    pub fn new(from: Rgb) -> Self {
        Self {
            from,
            to: None,
            direction: Direction::Up,
            cell: 3.0,
            opacity: 1.0,
            levels: 6,
        }
    }

    pub fn to(mut self, to: Rgb) -> Self {
        self.to = Some(to);
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn cell(mut self, cell: f64) -> Self {
        self.cell = cell;
        self
    }

    pub fn opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn levels(mut self, levels: u32) -> Self {
        self.levels = levels;
        self
    }

    /// Backing size for a wash covering `width`×`height` points.
    pub fn backing_size(&self, width: f64, height: f64) -> (u32, u32) {
        let cell = if self.cell > 0.0 { self.cell } else { 1.0 };
        (
            (js_round(width / cell).max(1.0)) as u32,
            (js_round(height / cell).max(1.0)) as u32,
        )
    }
}

/// Rasterize the wash into a new surface of `cols`×`rows` cells.
pub fn render_gradient(cols: u32, rows: u32, spec: &GradientSpec) -> Surface {
    let mut surface = Surface::new(cols, rows);
    if cols == 0 || rows == 0 {
        return surface;
    }
    let (w, h) = (cols as f64, rows as f64);
    let rad = spec.direction.angle().to_radians();
    let (dx, dy) = (rad.cos(), rad.sin());
    let project = |x: f64, y: f64| x * dx + y * dy;
    let corners = [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)];
    let mut min_p = f64::INFINITY;
    let mut max_p = f64::NEG_INFINITY;
    for (x, y) in corners {
        let p = project(x, y);
        min_p = min_p.min(p);
        max_p = max_p.max(p);
    }
    let span = if max_p - min_p == 0.0 {
        1.0
    } else {
        max_p - min_p
    };
    let levels = spec.levels.max(2) as f64;
    let k2 = (levels - 1.0) / levels;
    let k3 = 1.0 / levels;
    let steps = levels - 1.0;

    for y in 0..rows {
        for x in 0..cols {
            let value = project(x as f64 + 0.5, y as f64 + 0.5);
            let source = 1.0 - (value - min_p) / span;
            let composite = k2 * source + k3 * raw(x as i32, y as i32);
            let index = (composite.clamp(0.0, 1.0) * levels)
                .floor()
                .min(levels - 1.0);
            let q = index / steps;
            match spec.to {
                None => surface.paint(x as i32, y as i32, spec.from, spec.opacity * q),
                Some(to) => {
                    let mix = |a: u8, b: u8| js_round(b as f64 + (a as f64 - b as f64) * q) as u8;
                    let rgb = Rgb::new(
                        mix(spec.from.r, to.r),
                        mix(spec.from.g, to.g),
                        mix(spec.from.b, to.b),
                    );
                    surface.paint(x as i32, y as i32, rgb, spec.opacity);
                }
            }
        }
    }
    surface
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha_at(surface: &Surface, x: u32, y: u32, w: u32) -> u8 {
        surface.rgba()[((y * w + x) * 4 + 3) as usize]
    }

    #[test]
    fn the_field_rises_toward_the_direction_of_travel() {
        let spec = GradientSpec::new(Rgb::new(0, 200, 200)).direction(Direction::Up);
        let surface = render_gradient(16, 16, &spec);
        assert!(alpha_at(&surface, 8, 15, 16) > alpha_at(&surface, 8, 1, 16));

        let right = GradientSpec::new(Rgb::new(0, 200, 200)).direction(Direction::Right);
        let surface = render_gradient(16, 16, &right);
        assert!(alpha_at(&surface, 1, 8, 16) > alpha_at(&surface, 15, 8, 16));
    }

    #[test]
    fn two_tone_washes_stay_opaque_and_blend_between_the_colors() {
        let spec = GradientSpec::new(Rgb::new(240, 70, 70)).to(Rgb::new(53, 143, 243));
        let surface = render_gradient(8, 8, &spec);
        assert!(surface.rgba().chunks(4).all(|px| px[3] == 255));
        let distinct: std::collections::HashSet<[u8; 3]> = surface
            .rgba()
            .chunks(4)
            .map(|px| [px[0], px[1], px[2]])
            .collect();
        assert!(distinct.len() > 1);
    }

    #[test]
    fn backing_size_follows_the_cell_size() {
        let spec = GradientSpec::new(Rgb::new(0, 0, 0)).cell(3.0);
        assert_eq!(spec.backing_size(300.0, 90.0), (100, 30));
        let fine = spec.cell(1.0);
        assert_eq!(fine.backing_size(300.0, 90.0), (300, 90));
    }
}
