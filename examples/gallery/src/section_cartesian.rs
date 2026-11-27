//! the cartesian half of the page: area, bars and sparklines

use crate::state::State;
use crate::widgets::{self, SeriesGroup};
use rune::tok::Theme;
use rune::{El, Key, col, row};

fn handlers(chart: El, area: bool) -> El {
    chart
        .on_pointer_move_local(move |state: &mut State, p, _| {
            if area {
                state.area.hover_at(p.x);
            } else {
                state.bar.hover_at(p.x);
            }
        })
        .on_hover_change(move |state: &mut State, inside, _| {
            if !inside {
                if area {
                    state.area.leave();
                } else {
                    state.bar.leave();
                }
            }
        })
        .on_key(move |state: &mut State, event, _| match event.key {
            Key::Left | Key::Right => {
                let delta = if event.key == Key::Left { -1 } else { 1 };
                if area {
                    state.area.step_hover(delta);
                } else {
                    state.bar.step_hover(delta);
                }
                true
            }
            Key::Escape => {
                if area {
                    state.area.leave();
                } else {
                    state.bar.leave();
                }
                true
            }
            _ => false,
        })
}

pub fn section(state: &State, theme: &Theme) -> El {
    let area = handlers(state.area.view(theme), true);
    let bar = handlers(state.bar.view(theme), false);

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
                .child(area),
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
                .child(bar),
        )
        .child(
            row()
                .gap(12.0)
                .child(widgets::card(
                    "sparkline a",
                    theme,
                    state
                        .spark_a
                        .view(theme)
                        .on_pointer_move_local(|state: &mut State, p, _| {
                            state.spark_a.hover_at(p.x)
                        }),
                ))
                .child(widgets::card(
                    "sparkline b",
                    theme,
                    state
                        .spark_b
                        .view(theme)
                        .on_pointer_move_local(|state: &mut State, p, _| {
                            state.spark_b.hover_at(p.x)
                        }),
                )),
        )
}
