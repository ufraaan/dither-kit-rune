//! the cartesian half of the dashboard: area, bars and sparklines

use crate::interactions;
use crate::state::State;
use crate::widgets::{self, SeriesGroup};
use rune::tok::Theme;
use rune::{El, col, row};

pub fn section(state: &State, theme: &Theme) -> El {
    col()
        .gap(12.0)
        .child(
            col()
                .gap(4.0)
                .child(widgets::titled(
                    "area",
                    state.area.legend(),
                    theme,
                    SeriesGroup::Area,
                ))
                .child(interactions::cartesian(state.area.view(theme), true)),
        )
        .child(
            col()
                .gap(4.0)
                .child(widgets::titled(
                    "bar",
                    state.bar.legend(),
                    theme,
                    SeriesGroup::Bar,
                ))
                .child(interactions::cartesian(state.bar.view(theme), false)),
        )
        .child(
            row()
                .gap(12.0)
                .child(widgets::card(
                    "sparkline a",
                    theme,
                    interactions::sparkline(state.spark_a.view(theme), false),
                ))
                .child(widgets::card(
                    "sparkline b",
                    theme,
                    interactions::sparkline(state.spark_b.view(theme), true),
                )),
        )
}
