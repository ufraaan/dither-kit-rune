//! scene dispatch: dashboard or a single component

use crate::state::State;
use crate::{section_atoms, section_cartesian, section_polar, showcase, widgets};
use rune::tok;
use rune::tok::Theme;
use rune::{Cx, El, col, row};

pub fn page(state: &State, cx: &mut Cx<State>) -> El {
    if cx.is_reduced_motion() {
        state.reduced.set(true);
    }
    let theme = tok::theme(state.dark);
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
        .fill(theme.bg)
        .child(
            col()
                .w_full()
                .padding(20.0)
                .gap(14.0)
                .child(widgets::header(state, theme))
                .child(
                    row()
                        .gap(16.0)
                        .child(
                            col()
                                .w(590.0)
                                .child(section_cartesian::section(state, theme)),
                        )
                        .child(
                            col()
                                .w(290.0)
                                .gap(12.0)
                                .child(section_polar::section(state, theme))
                                .child(section_atoms::section(state, theme)),
                        ),
                ),
        )
}
