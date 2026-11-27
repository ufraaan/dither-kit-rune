//! Ordered-dither constants, the 4×4 Bayer matrix and easings.

use crate::js_round;

/// Raw 4×4 ordered (Bayer) matrix, exactly as upstream.
pub const BAYER4: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

/// Normalized threshold `(value + 0.5) / 16` at surface pixel `(x, y)`.
pub fn threshold(x: i32, y: i32) -> f64 {
    (BAYER4[(y & 3) as usize][(x & 3) as usize] as f64 + 0.5) / 16.0
}

/// Raw matrix value over 16, the grayscale the gradient's threshold map uses.
pub fn raw(x: i32, y: i32) -> f64 {
    BAYER4[(y & 3) as usize][(x & 3) as usize] as f64 / 16.0
}

pub fn clamp01(t: f64) -> f64 {
    t.clamp(0.0, 1.0)
}

/// CSS px per dither cell.
pub const CELL: f64 = 2.0;
pub const MAX_COLS: u32 = 520;
pub const MAX_ROWS: u32 = 200;
/// Opacity of the top border outline, just under solid.
pub const BORDER_ALPHA: f64 = 0.72;
/// Opacity of an "off" cell relative to an "on" cell of the same color.
pub const OFF_TIER: f64 = 0.4;

/// Backing resolution for a plot rect: low-res, scaled up `pixelated`.
pub fn backing_size(width: f64, height: f64) -> (u32, u32) {
    let cols = js_round(width / CELL).clamp(8.0, MAX_COLS as f64) as u32;
    let rows = js_round(height / CELL).clamp(8.0, MAX_ROWS as f64) as u32;
    (cols, rows)
}

pub fn ease_in_out_cubic(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

pub fn ease_out_cubic(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}
