// donut chart — inner radius is a fraction of the outer radius
use dither_core::color::DitherColor;
use dither_kit_rune::pie::{Pie, PieSeries, PieSpec};

pub fn chart() -> Pie {
    Pie::new(
        PieSpec::new(
            (640.0, 560.0),
            vec![
                PieSeries::new("rust", 38.0, DitherColor::Orange),
                PieSeries::new("typescript", 27.0, DitherColor::Blue),
                PieSeries::new("go", 19.0, DitherColor::Green),
                PieSeries::new("zig", 11.0, DitherColor::Purple),
            ],
        )
        .inner(0.52),
    )
}
