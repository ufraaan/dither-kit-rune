//! small conversions shared by the components

use dither_core::color::Rgb;
use rune::Color;

/// an engine color as a rune color
pub fn to_color(rgb: Rgb) -> Color {
    Color {
        r: rgb.r as f32 / 255.0,
        g: rgb.g as f32 / 255.0,
        b: rgb.b as f32 / 255.0,
        a: 1.0,
    }
}

/// axis and tooltip number formatting: integers stay integers, the rest keep a
/// decimal
pub fn format_tick(value: f64) -> String {
    if value.abs() < 0.0001 {
        "0".to_string()
    } else if (value - value.round()).abs() < 0.05 {
        format!("{:.0}", value)
    } else {
        format!("{:.1}", value)
    }
}
