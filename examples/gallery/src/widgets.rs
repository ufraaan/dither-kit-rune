//! shared chrome for the demo page

use crate::state::State;
use dither_kit_rune::chart::LegendEntry;
use dither_kit_rune::util::to_color;
use rune::tok::{TextStyle, Theme, type_};
use rune::{El, col, el, row, text};

const LEGEND: TextStyle = TextStyle::new(11.0, 500.0).mono();
const CAPTION: TextStyle = TextStyle::new(11.0, 500.0).mono();

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SeriesGroup {
    Area,
    Bar,
    Pie,
    Radar,
}

pub fn header(state: &State, theme: &Theme) -> El {
    let title = text("Dither Kit").font(type_::TITLE).color(theme.text);
    let sub = text("hover anything · click a legend to isolate · arrow keys inspect")
        .color(theme.text_secondary);
    let status = text(if state.dark { "dark" } else { "light" })
        .font(LEGEND)
        .color(theme.text_secondary);
    row()
        .justify_between()
        .center_y()
        .child(col().gap(3.0).child(title).child(sub))
        .child(status)
}

pub fn caption(label: &str, theme: &Theme) -> El {
    text(label).font(CAPTION).color(theme.text_secondary)
}

pub fn panel(title: &str, theme: &Theme, content: El) -> El {
    el().padding(14.0)
        .radius(12.0)
        .fill(theme.card)
        .border(1.0, theme.card_border)
        .child(
            col()
                .gap(8.0)
                .child(caption(title, theme))
                .child(el().w_full().grow(1.0).center().child(content)),
        )
}

pub fn legend_row(entries: Vec<LegendEntry>, theme: &Theme, group: SeriesGroup) -> El {
    entries
        .into_iter()
        .fold(row().gap(12.0).center_y(), |line, entry| {
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
            line.child(
                row()
                    .gap(5.0)
                    .hit()
                    .focusable(true)
                    .role("button")
                    .label("toggle series")
                    .on_press(move |state: &mut State| match group {
                        SeriesGroup::Area => state.area.toggle_series(index),
                        SeriesGroup::Bar => state.bar.toggle_series(index),
                        SeriesGroup::Pie => state.pie.toggle_slice(index),
                        SeriesGroup::Radar => state.radar.toggle_series(index),
                    })
                    .child(swatch)
                    .child(label),
            )
        })
}

pub fn legend_wrapped(
    entries: Vec<LegendEntry>,
    theme: &Theme,
    group: SeriesGroup,
    per_row: usize,
) -> El {
    let mut rows = col().gap(4.0);
    let mut chunk = Vec::new();
    for entry in entries {
        chunk.push(entry);
        if chunk.len() == per_row {
            rows = rows.child(legend_row(std::mem::take(&mut chunk), theme, group));
        }
    }
    if !chunk.is_empty() {
        rows = rows.child(legend_row(chunk, theme, group));
    }
    rows
}
