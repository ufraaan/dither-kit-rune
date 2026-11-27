// gradient — quantized directional wash
use dither_core::color::DitherColor;
use dither_core::gradient::{Direction, GradientSpec};
use dither_kit_rune::gradient::DitherGradient;

pub fn gradient() -> DitherGradient {
    DitherGradient::new(
        (900.0, 480.0),
        GradientSpec::new(DitherColor::Blue.fill())
            .direction(Direction::Up)
            .cell(3.0)
            .opacity(0.9),
    )
}

// two-tone: .to(DitherColor::Pink.fill())
