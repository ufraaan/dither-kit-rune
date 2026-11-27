//! Deterministic mirrored avatars: an 8×8 cell glyph seeded by a name.

use crate::Surface;
use crate::bayer::{clamp01, threshold};
use crate::color::{Rgb, hue_fill};
use crate::rand::{XorShift32, fnv1a};

/// Cells per side.
pub const GRID: usize = 8;
/// Backing pixels per cell, so the surface is 32×32.
pub const CELL_PX: usize = 4;
/// Backing pixels per side.
pub const PIXELS: u32 = (GRID * CELL_PX) as u32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarMirror {
    /// Picks the axis from the name: half the avatars fold left/right, half top/bottom.
    #[default]
    Auto,
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AvatarModel {
    /// Row-major cells.
    pub on: [bool; GRID * GRID],
    /// Per-cell dither density for on cells.
    pub density: [f64; GRID * GRID],
    pub fill: Rgb,
}

/// Derive the grid, mirror axis, hue and densities from one PRNG stream.
///
/// Every draw happens unconditionally, so overriding the hue or mirror never
/// changes the pattern.
pub fn avatar_model(name: &str, hue: Option<f64>, mirror: AvatarMirror) -> AvatarModel {
    let mut rand = XorShift32::new(fnv1a(name));
    let mut bits = [false; 32];
    for bit in &mut bits {
        *bit = rand.next_f64() < 0.5;
    }
    let drawn_vertical = rand.next_f64() < 0.5;
    let drawn_hue = (rand.next_f64() * 180.0).floor() * 2.0;
    let mut half_density = [0.0; 32];
    for density in &mut half_density {
        *density = 0.55 + rand.next_f64() * 0.45;
    }

    let vertical = match mirror {
        AvatarMirror::Auto => drawn_vertical,
        AvatarMirror::Horizontal => false,
        AvatarMirror::Vertical => true,
    };
    let hue = hue.unwrap_or(drawn_hue);

    let mut on = [false; GRID * GRID];
    let mut density = [0.0; GRID * GRID];
    for r in 0..GRID {
        for c in 0..GRID {
            let i = if vertical {
                r.min(GRID - 1 - r) * GRID + c
            } else {
                r * (GRID / 2) + c.min(GRID - 1 - c)
            };
            on[r * GRID + c] = bits[i];
            density[r * GRID + c] = half_density[i];
        }
    }
    AvatarModel {
        on,
        density,
        fill: hue_fill(hue),
    }
}

/// Paint the avatar into a `PIXELS`×`PIXELS` surface. `progress` drives the
/// Bayer-ordered materialize entrance; pass 1 for the settled glyph.
pub fn paint_avatar(surface: &mut Surface, model: &AvatarModel, progress: f64) {
    let progress = clamp01(progress);
    for r in 0..GRID {
        for c in 0..GRID {
            if !model.on[r * GRID + c] {
                continue;
            }
            let start = threshold(c as i32, r as i32) * 0.7;
            let cell_alpha = clamp01((progress - start) / 0.3);
            if cell_alpha <= 0.0 {
                continue;
            }
            let density = model.density[r * GRID + c];
            let base = 0.35 + 0.65 * density;
            for py in 0..CELL_PX {
                for px in 0..CELL_PX {
                    let gx = c * CELL_PX + px;
                    let gy = r * CELL_PX + py;
                    let lit = density > threshold(gx as i32, gy as i32);
                    let alpha = (if lit { base } else { base * 0.35 }) * cell_alpha;
                    surface.paint(gx as i32, gy as i32, model.fill, alpha);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn avatars_match_the_upstream_model() {
        let text = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/avatars.txt"
        ))
        .expect("avatar fixtures");
        let mut checked = 0;
        for block in text.split("\n\n") {
            let Some(start) = block.find('[') else {
                continue;
            };
            let mut lines = block[start..].lines();
            let header = lines.next().unwrap_or_default().trim_matches(['[', ']']);
            let mut parts = header.split('|');
            let name: String = parts
                .next()
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();
            let mirror = match parts.next().unwrap_or("auto") {
                "horizontal" => AvatarMirror::Horizontal,
                "vertical" => AvatarMirror::Vertical,
                _ => AvatarMirror::Auto,
            };
            let fill: Vec<u8> = parts
                .next()
                .unwrap_or_default()
                .split(',')
                .filter_map(|v| v.parse().ok())
                .collect();

            let mut expected = [false; GRID * GRID];
            for r in 0..GRID {
                let row = lines.next().unwrap_or_default();
                for (c, ch) in row.chars().take(GRID).enumerate() {
                    expected[r * GRID + c] = ch == '#';
                }
            }
            let model = avatar_model(&name, None, mirror);
            assert_eq!(model.on, expected, "{name}");
            assert_eq!(
                [model.fill.r, model.fill.g, model.fill.b],
                fill[..3],
                "{name}"
            );

            let density_line = lines.next().unwrap_or_default();
            let expected_density: Vec<f64> = density_line
                .trim_start_matches("density=")
                .split(',')
                .filter_map(|v| v.parse().ok())
                .collect();
            for (i, want) in expected_density.iter().enumerate() {
                assert!((model.density[i] - want).abs() < 1e-4, "{name} cell {i}");
            }
            checked += 1;
        }
        assert_eq!(checked, 6, "every generated avatar case ran");
    }

    #[test]
    fn mirroring_folds_the_glyph_on_the_requested_axis() {
        let model = avatar_model("ada", None, AvatarMirror::Horizontal);
        for r in 0..GRID {
            for c in 0..GRID {
                assert_eq!(model.on[r * GRID + c], model.on[r * GRID + (GRID - 1 - c)]);
            }
        }
        let model = avatar_model("ada", None, AvatarMirror::Vertical);
        for r in 0..GRID {
            for c in 0..GRID {
                assert_eq!(model.on[r * GRID + c], model.on[(GRID - 1 - r) * GRID + c]);
            }
        }
    }

    #[test]
    fn paint_stays_inside_the_avatar_surface() {
        let model = avatar_model("Ada Lovelace", Some(200.0), AvatarMirror::Auto);
        let mut surface = Surface::new(PIXELS, PIXELS);
        paint_avatar(&mut surface, &model, 1.0);
        assert_eq!(surface.rgba().len(), (PIXELS * PIXELS * 4) as usize);
    }
}
