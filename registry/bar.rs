// bar chart — grouped by default, stacked when you say so
use dither_core::color::DitherColor;
use dither_core::stack::StackMode;
use dither_kit_rune::cartesian::ChartKind;
use dither_kit_rune::chart::{Chart, ChartSeries, ChartSpec};

pub fn chart() -> Chart {
    Chart::new(
        ChartSpec::new(
            (840.0, 320.0),
            ["Mon", "Tue", "Wed", "Thu"].map(String::from).to_vec(),
            vec![
                ChartSeries::new("builds", DitherColor::Green),
                ChartSeries::new("deploys", DitherColor::Purple),
            ],
            vec![vec![42.0, 58.0, 61.0, 47.0], vec![31.0, 44.0, 39.0, 51.0]],
        )
        .kind(ChartKind::Bar)
        .stack(StackMode::Stacked),
    )
}
