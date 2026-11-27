//! one component at a time, framed for screenshots

use crate::state::{Scene, State, canvas};
use crate::{interactions, widgets};
use rune::tok::{TextStyle, Theme};
use rune::{El, col, el, row, text};

const KICKER: TextStyle = TextStyle::new(10.0, 600.0).mono();
const TITLE: TextStyle = TextStyle::new(22.0, 600.0);
const BLURB: TextStyle = TextStyle::new(13.0, 400.0);
const FOOT: TextStyle = TextStyle::new(10.0, 500.0).mono();

pub fn page(state: &State, theme: &Theme, scene: Scene) -> El {
    let (title, blurb, content) = content(state, theme, scene);
    let kicker = text("DITHER KIT · RUNE")
        .font(KICKER)
        .color(theme.text_secondary);
    let heading = text(title).font(TITLE).color(theme.text);
    let blurb = text(blurb).font(BLURB).color(theme.text_secondary);
    let footer = text(format!(
        "dither-kit-rune · {} · {}/{} · arrow keys switch scene",
        scene.name(),
        scene.index() + 1,
        Scene::ALL.len()
    ))
    .font(FOOT)
    .color(theme.text_secondary);

    col()
        .w_full()
        .h_full()
        .padding(28.0)
        .gap(14.0)
        .fill(canvas(state))
        .child(col().gap(4.0).child(kicker).child(heading).child(blurb))
        .child(
            el().w_full().grow(1.0).center().child(
                el().padding(20.0)
                    .radius(14.0)
                    .fill(theme.card)
                    .border(1.0, theme.card_border)
                    .center()
                    .child(content),
            ),
        )
        .child(footer)
}

fn content(state: &State, theme: &Theme, scene: Scene) -> (&'static str, &'static str, El) {
    match scene {
        Scene::Area => (
            "Area chart",
            "Two series, a reference line, hover scrubbing and a left-to-right reveal.",
            col()
                .gap(10.0)
                .child(widgets::legend_row(
                    state.area.legend(),
                    theme,
                    widgets::SeriesGroup::Area,
                ))
                .child(interactions::cartesian(state.area.view(theme), true)),
        ),
        Scene::Bar => (
            "Bar chart",
            "Stacked bars with a staggered entrance; the hovered category lifts, the rest dim.",
            col()
                .gap(10.0)
                .child(widgets::legend_row(
                    state.bar.legend(),
                    theme,
                    widgets::SeriesGroup::Bar,
                ))
                .child(interactions::cartesian(state.bar.view(theme), false)),
        ),
        Scene::Sparkline => (
            "Sparkline",
            "The same chart with no chrome: one series, hover lift, 26 samples.",
            col()
                .gap(8.0)
                .child(interactions::sparkline(state.spark_a.view(theme), false))
                .child(widgets::caption("26 samples · hover to lift", theme)),
        ),
        Scene::Donut => (
            "Donut chart",
            "Per-pixel radial dither, clockwise sweep-in, slice hover pop and a tooltip.",
            col()
                .gap(10.0)
                .center()
                .child(interactions::pie(state.pie.view(theme)))
                .child(widgets::legend_wrapped(
                    state.pie.legend(),
                    theme,
                    widgets::SeriesGroup::Pie,
                    5,
                )),
        ),
        Scene::Radar => (
            "Radar chart",
            "Polygons over five spokes, dithered by distance to the edge, scaling in from the centre.",
            col()
                .gap(10.0)
                .center()
                .child(interactions::radar(state.radar.view(theme)))
                .child(widgets::legend_row(
                    state.radar.legend(),
                    theme,
                    widgets::SeriesGroup::Radar,
                )),
        ),
        Scene::Avatars => (
            "Avatars",
            "An 8×8 mirrored glyph seeded by name: same name, same avatar, every time.",
            row()
                .gap(20.0)
                .child(avatar(state, theme, 0))
                .child(avatar(state, theme, 1))
                .child(avatar(state, theme, 2))
                .child(avatar(state, theme, 3)),
        ),
        Scene::Buttons => (
            "Dither buttons",
            "Native focusable controls; the dither fill lifts on hover and brightens on press.",
            col()
                .gap(12.0)
                .child(
                    row()
                        .gap(12.0)
                        .child(interactions::button(state.buttons[0].view(theme), 0))
                        .child(interactions::button(state.buttons[1].view(theme), 1)),
                )
                .child(
                    row()
                        .gap(12.0)
                        .child(interactions::button(state.buttons[2].view(theme), 2))
                        .child(interactions::button(state.buttons[3].view(theme), 3)),
                ),
        ),
        Scene::Gradient => (
            "Gradient",
            "A quantized directional wash, from blue into pink across six levels.",
            col()
                .gap(12.0)
                .center()
                .child(el().radius(8.0).child(state.gradient.view()))
                .child(widgets::caption("blue → pink · 6 levels · up", theme)),
        ),
        Scene::Dashboard => ("Dashboard", "", row()),
    }
}

fn avatar(state: &State, theme: &Theme, index: usize) -> El {
    let name = ["ada", "Ada Lovelace", "🦀", "grace hopper"][index];
    col()
        .gap(8.0)
        .center()
        .child(state.avatars[index].view())
        .child(text(name).font(FOOT).color(theme.text_secondary))
}
