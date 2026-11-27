// radar chart — one value per axis, series-major
use dither_core::color::DitherColor;
use dither_kit_rune::radar::{Radar, RadarSeries, RadarSpec};

pub fn chart() -> Radar {
    Radar::new(
        RadarSpec::new(
            (720.0, 560.0),
            ["speed", "size", "docs", "tests", "perf"].map(String::from).to_vec(),
            vec![
                RadarSeries::new("v1", DitherColor::Blue),
                RadarSeries::new("v2", DitherColor::Pink),
            ],
            vec![vec![7.0, 6.0, 5.0, 8.0, 6.0], vec![9.0, 8.0, 9.0, 7.0, 9.0]],
        ),
    )
}
