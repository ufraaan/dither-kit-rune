// sparkline — the chart with its chrome turned off
use dither_core::color::DitherColor;
use dither_kit_rune::chart::{Chart, ChartSeries, ChartSpec};

pub fn chart() -> Chart {
    let values = [3.0, 7.0, 5.0, 9.0, 8.0, 12.0, 10.0, 14.0, 11.0, 16.0];
    let categories = (0..values.len()).map(|i| i.to_string()).collect();
    Chart::new(
        ChartSpec::new(
            (900.0, 300.0),
            categories,
            vec![ChartSeries::new("value", DitherColor::Green)],
            vec![values.to_vec()],
        )
        .sparkline(),
    )
}
