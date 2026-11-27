//! the polar half of the page: donut and radar

use crate::state::State;
use crate::widgets::{self, SeriesGroup};
use rune::tok::Theme;
use rune::{El, col};

pub fn section(state: &State, theme: &Theme) -> El {
    let pie = state
        .pie
        .view(theme)
        .on_pointer_move_local(|state: &mut State, p, _| state.pie.hover_at(p.x, p.y))
        .on_hover_change(|state: &mut State, inside, _| {
            if !inside {
                state.pie.leave();
            }
        });
    let radar = state
        .radar
        .view(theme)
        .on_pointer_move_local(|state: &mut State, p, _| state.radar.hover_at(p.x, p.y))
        .on_hover_change(|state: &mut State, inside, _| {
            if !inside {
                state.radar.leave();
            }
        });

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
                .child(pie),
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
                .child(radar),
        )
}
