//! chrome-free component captures for the landing page

use crate::state::{Scene, State};
use crate::{interactions, widgets};
use rune::tok::Theme;
use rune::{El, col, el, row};

pub fn page(state: &State, theme: &Theme, scene: Scene) -> El {
    el().w_full()
        .h_full()
        .fill(theme.bg)
        .center()
        .child(content(state, theme, scene))
}

fn content(state: &State, theme: &Theme, scene: Scene) -> El {
    match scene {
        Scene::Area => chart(
            interactions::cartesian(state.area.view(theme), true),
            widgets::legend_row(state.area.legend(), theme, widgets::SeriesGroup::Area),
        ),
        Scene::Bar => chart(
            interactions::cartesian(state.bar.view(theme), false),
            widgets::legend_row(state.bar.legend(), theme, widgets::SeriesGroup::Bar),
        ),
        Scene::Sparkline => interactions::sparkline(state.spark_a.view(theme), false),
        Scene::Donut => chart(
            interactions::pie(state.pie.view(theme)),
            widgets::legend_row(state.pie.legend(), theme, widgets::SeriesGroup::Pie),
        ),
        Scene::Radar => chart(
            interactions::radar(state.radar.view(theme)),
            widgets::legend_row(state.radar.legend(), theme, widgets::SeriesGroup::Radar),
        ),
        Scene::Avatars => state
            .avatars
            .iter()
            .fold(row().gap(14.0), |line, avatar| line.child(avatar.view())),
        Scene::Buttons => (0..4).fold(row().gap(16.0), |line, index| {
            line.child(interactions::button(
                state.buttons[index].view(theme),
                index,
            ))
        }),
        Scene::Gradient => el().radius(8.0).child(state.gradient.view()),
        Scene::Dashboard => row(),
    }
}

fn chart(view: El, legend: El) -> El {
    col()
        .gap(10.0)
        .child(row().w_full().justify_end().child(legend))
        .child(view)
}
