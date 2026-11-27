// area chart — build once, keep the Chart in your state
use dither_core::color::DitherColor;
use dither_kit_rune::cartesian::ChartKind;
use dither_kit_rune::chart::{Chart, ChartSeries, ChartSpec};

pub fn chart() -> Chart {
    Chart::new(
        ChartSpec::new(
            (900.0, 520.0),
            ["Jan", "Feb", "Mar", "Apr"].map(String::from).to_vec(),
            vec![ChartSeries::new("value", DitherColor::Blue)],
            vec![vec![12.0, 18.0, 15.0, 22.0]],
        )
        .kind(ChartKind::Area),
    )
}

// in your view, forward pointer input; the app's state owns the chart
//
// chart.view(theme)
//     .on_pointer_move_local(|state: &mut App, p, _| state.chart.hover_at(p.x))
//     .on_hover_change(|state: &mut App, inside, _| {
//         if !inside { state.chart.leave(); }
//     })
