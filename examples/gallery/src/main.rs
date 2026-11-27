//! Interactive gallery: a live area chart and bar chart with hover, keyboard
//! inspection, clickable legends and the staggered entrance.
//!
//! `cargo run -p gallery` opens the window; `-- --snap gallery.png` renders it
//! headless.

use std::time::Duration;

use dither_core::color::DitherColor;
use dither_core::paint::Variant;
use dither_core::stack::StackMode;
use dither_kit_rune::cartesian::ChartKind;
use dither_kit_rune::chart::{Chart, ChartSeries, ChartSpec, LegendEntry, Margins, to_color};
use rune::tok::{self, TextStyle, Theme, type_};
use rune::{AppSpec, Cx, El, Key, Tx, col, el, row, text};

const LEGEND: TextStyle = TextStyle::new(11.0, 500.0).mono();

struct State {
    dark: bool,
    area: Chart,
    bar: Chart,
}

fn area_chart() -> Chart {
    let categories = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]
    .map(String::from)
    .to_vec();
    let desktop = [
        38.0, 42.0, 55.0, 48.0, 63.0, 58.0, 71.0, 66.0, 74.0, 62.0, 69.0, 57.0,
    ];
    let mobile = [
        22.0, 26.0, 31.0, 30.0, 38.0, 44.0, 41.0, 49.0, 46.0, 52.0, 47.0, 55.0,
    ];
    Chart::new(
        ChartSpec::new(
            (640.0, 220.0),
            categories,
            vec![
                ChartSeries::new("desktop", DitherColor::Blue),
                ChartSeries::new("mobile", DitherColor::Orange).variant(Variant::Dotted),
            ],
            vec![desktop.to_vec(), mobile.to_vec()],
        )
        .key("area-chart")
        .margins(Margins {
            top: 14.0,
            ..Margins::default()
        })
        .kind(ChartKind::Area)
        .reference(60.0),
    )
}

fn bar_chart() -> Chart {
    let categories = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        .map(String::from)
        .to_vec();
    let builds = [42.0, 58.0, 61.0, 47.0, 73.0, 68.0, 52.0];
    let deploys = [31.0, 44.0, 39.0, 51.0, 46.0, 60.0, 41.0];
    Chart::new(
        ChartSpec::new(
            (640.0, 210.0),
            categories,
            vec![
                ChartSeries::new("builds", DitherColor::Green),
                ChartSeries::new("deploys", DitherColor::Purple).variant(Variant::Hatched),
            ],
            vec![builds.to_vec(), deploys.to_vec()],
        )
        .key("bar-chart")
        .margins(Margins {
            top: 20.0,
            ..Margins::default()
        })
        .kind(ChartKind::Bar)
        .stack(StackMode::Stacked),
    )
}

fn init() -> State {
    State {
        dark: true,
        area: area_chart(),
        bar: bar_chart(),
    }
}

fn legend(entries: Vec<LegendEntry>, theme: &Theme, area: bool) -> El {
    entries
        .into_iter()
        .fold(row().gap(14.0).center_y(), |bar, entry| {
            let swatch = el()
                .size(8.0, 8.0)
                .radius(1.0)
                .fill(to_color(entry.color))
                .opacity(if entry.dimmed { 0.4 } else { 1.0 });
            let label = text(entry.label)
                .font(LEGEND)
                .color(if entry.selected {
                    theme.text
                } else {
                    theme.text_secondary
                })
                .opacity(if entry.dimmed { 0.4 } else { 1.0 });
            let index = entry.index;
            bar.child(
                row()
                    .gap(5.0)
                    .hit()
                    .focusable(true)
                    .role("button")
                    .label("toggle series")
                    .on_press(move |s: &mut State| {
                        if area {
                            s.area.toggle_series(index);
                        } else {
                            s.bar.toggle_series(index);
                        }
                    })
                    .child(swatch)
                    .child(label),
            )
        })
}

fn view(s: &State, _: &mut Cx<State>) -> El {
    let theme = tok::theme(s.dark);
    let heading = text("Dither Kit").font(type_::TITLE).color(theme.text);
    let sub =
        text("hover, arrow keys, click a legend to isolate a series").color(theme.text_secondary);

    let toggle = el()
        .hit()
        .focusable(true)
        .role("button")
        .label(if s.dark {
            "Switch to light"
        } else {
            "Switch to dark"
        })
        .padding(6.0)
        .radius(6.0)
        .fill(theme.control_off)
        .on_press(|s: &mut State| s.dark = !s.dark)
        .child(
            text(if s.dark { "light" } else { "dark" })
                .font(LEGEND)
                .color(theme.text),
        );

    let area_chart = s
        .area
        .view(&theme)
        .on_pointer_move_local(|s: &mut State, p, _| s.area.hover_at(p.x))
        .on_hover_change(|s: &mut State, inside, _| {
            if !inside {
                s.area.leave();
            }
        })
        .on_key(|s: &mut State, ev, _| match ev.key {
            Key::Left => {
                s.area.step_hover(-1);
                true
            }
            Key::Right => {
                s.area.step_hover(1);
                true
            }
            Key::Escape => {
                s.area.leave();
                true
            }
            _ => false,
        });

    let bar_chart = s
        .bar
        .view(&theme)
        .on_pointer_move_local(|s: &mut State, p, _| s.bar.hover_at(p.x))
        .on_hover_change(|s: &mut State, inside, _| {
            if !inside {
                s.bar.leave();
            }
        })
        .on_key(|s: &mut State, ev, _| match ev.key {
            Key::Left => {
                s.bar.step_hover(-1);
                true
            }
            Key::Right => {
                s.bar.step_hover(1);
                true
            }
            Key::Escape => {
                s.bar.leave();
                true
            }
            _ => false,
        });

    col()
        .w_full()
        .h_full()
        .padding(28.0)
        .gap(16.0)
        .fill(theme.bg)
        .child(
            row()
                .justify_between()
                .center_y()
                .child(col().gap(4.0).child(heading).child(sub))
                .child(toggle),
        )
        .child(
            col()
                .gap(6.0)
                .child(legend(s.area.legend(), &theme, true))
                .child(area_chart),
        )
        .child(
            col()
                .gap(6.0)
                .child(legend(s.bar.legend(), &theme, false))
                .child(bar_chart),
        )
}

fn poll(s: &mut State, clock: f64, tx: &mut Tx) -> bool {
    let area = s.area.tick(clock);
    let bar = s.bar.tick(clock);
    if area || bar {
        tx.wake_after(Duration::ZERO);
    }
    area || bar
}

fn main() {
    rune::run(AppSpec::<State> {
        scenes: &[],
        poll: Some(poll),
        ..AppSpec::new("Dither Kit Gallery", (704.0, 660.0), init, view)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};

    #[test]
    fn charts_paint_and_share_the_plot_width() {
        let state = init();
        assert_eq!(state.area.progress(), 0.0);
        assert_eq!(state.area.texture().pixels().size(), (296, 92));
        assert_eq!(state.bar.texture().pixels().size(), (296, 84));

        let mut rt = Runtime::new(
            init(),
            view,
            (704.0, 660.0),
            Env {
                system_dark: true,
                scale: 2.0,
            },
        );
        rt.frame(0.0);
        assert!(rt.find("area-chart").is_some());
        assert!(rt.find("bar-chart").is_some());
    }

    #[test]
    fn hovering_picks_a_category_and_opens_the_tooltip() {
        let mut rt = Runtime::new(
            init(),
            view,
            (704.0, 660.0),
            Env {
                system_dark: true,
                scale: 2.0,
            },
        );
        rt.frame(0.0);
        let (x, y) = rt.find("area-chart").unwrap();
        rt.pointer_move(x, y);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.area.hover(), Some(5));
        assert!(rt.find("area-chart-tooltip").is_some());
    }

    #[test]
    fn the_entrance_advances_with_the_clock() {
        let mut state = init();
        assert!(state.area.tick(0.0));
        let first = state.area.texture().pixels().size();
        assert!(state.area.tick(0.45));
        assert_eq!(state.area.texture().pixels().size(), first);
        assert!(!state.area.tick(5.0));
        assert_eq!(state.area.progress(), 1.0);
    }
}
