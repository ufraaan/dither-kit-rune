//! the polar half of the dashboard: donut and radar

use crate::interactions;
use crate::state::State;
use crate::widgets::{self, SeriesGroup};
use rune::tok::Theme;
use rune::{El, col};

pub fn section(state: &State, theme: &Theme) -> El {
    col()
        .gap(12.0)
        .child(
            col()
                .gap(4.0)
                .child(widgets::caption("donut", theme))
                .child(widgets::legend_wrapped(
                    state.pie.legend(),
                    theme,
                    SeriesGroup::Pie,
                    3,
                ))
                .child(interactions::pie(state.pie.view(theme))),
        )
        .child(
            col()
                .gap(4.0)
                .child(widgets::titled(
                    "radar",
                    state.radar.legend(),
                    theme,
                    SeriesGroup::Radar,
                ))
                .child(interactions::radar(state.radar.view(theme))),
        )
}
