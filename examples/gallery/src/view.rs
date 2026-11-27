//! scene dispatch: a 2×2 dashboard grid or a single component

use crate::state::{State, canvas};
use crate::{bare, interactions, showcase, widgets};
use rune::tok;
use rune::tok::Theme;
use rune::{Cx, El, col, row};

const CARD_W: f32 = 445.0;

pub fn page(state: &State, cx: &mut Cx<State>) -> El {
    if cx.is_reduced_motion() {
        state.reduced.set(true);
    }
    let theme = tok::theme(state.dark);
    if std::env::var_os("DITHER_BARE").is_some() && state.scene.is_showcase() {
        return bare::page(state, &theme, state.scene);
    }
    if state.scene.is_showcase() {
        showcase::page(state, &theme, state.scene)
    } else {
        dashboard(state, &theme)
    }
}

fn dashboard(state: &State, theme: &Theme) -> El {
    col()
        .w_full()
        .h_full()
        .scroll(rune::ScrollSpec::default())
        .fill(canvas(state))
        .child(
            col()
                .w_full()
                .padding(18.0)
                .gap(14.0)
                .child(widgets::header(state, theme))
                .child(
                    row()
                        .gap(14.0)
                        .child(area(state, theme))
                        .child(bar(state, theme)),
                )
                .child(
                    row()
                        .gap(14.0)
                        .child(donut(state, theme))
                        .child(radar(state, theme)),
                ),
        )
}

fn area(state: &State, theme: &Theme) -> El {
    widgets::panel(
        "area",
        theme,
        col()
            .gap(6.0)
            .child(interactions::cartesian(state.area.view(theme), true))
            .child(widgets::legend_row(
                state.area.legend(),
                theme,
                widgets::SeriesGroup::Area,
            )),
    )
    .w(CARD_W)
}

fn bar(state: &State, theme: &Theme) -> El {
    widgets::panel(
        "bar",
        theme,
        col()
            .gap(6.0)
            .child(interactions::cartesian(state.bar.view(theme), false))
            .child(widgets::legend_row(
                state.bar.legend(),
                theme,
                widgets::SeriesGroup::Bar,
            )),
    )
    .w(CARD_W)
}

fn donut(state: &State, theme: &Theme) -> El {
    widgets::panel(
        "donut",
        theme,
        col()
            .gap(6.0)
            .child(interactions::pie(state.pie.view(theme)))
            .child(widgets::legend_wrapped(
                state.pie.legend(),
                theme,
                widgets::SeriesGroup::Pie,
                5,
            )),
    )
    .w(CARD_W)
}

fn radar(state: &State, theme: &Theme) -> El {
    widgets::panel(
        "radar",
        theme,
        col()
            .gap(6.0)
            .child(interactions::radar(state.radar.view(theme)))
            .child(widgets::legend_row(
                state.radar.legend(),
                theme,
                widgets::SeriesGroup::Radar,
            )),
    )
    .w(CARD_W)
}
