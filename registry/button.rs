// button — a real control; wire hover, press and the action yourself
use dither_core::color::DitherColor;
use dither_kit_rune::button::{ButtonSpec, DitherButton};

pub fn button() -> DitherButton {
    DitherButton::new(ButtonSpec::new("save changes", (200.0, 56.0)).color(DitherColor::Blue))
}

// in your view:
//
// button.view(theme)
//     .on_hover_change(|state: &mut App, inside, _| state.button.hover(inside))
//     .on_press(|state: &mut App| state.save())
