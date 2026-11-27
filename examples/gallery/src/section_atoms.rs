//! avatars, buttons and the gradient wash

use crate::interactions;
use crate::state::State;
use crate::widgets;
use rune::tok::Theme;
use rune::{El, col, el, row};

fn avatars(state: &State) -> El {
    state
        .avatars
        .iter()
        .fold(row().gap(10.0), |line, avatar| line.child(avatar.view()))
}

fn buttons(state: &State, theme: &Theme) -> El {
    col()
        .gap(8.0)
        .child(
            row()
                .gap(8.0)
                .child(interactions::button(state.buttons[0].view(theme), 0))
                .child(interactions::button(state.buttons[1].view(theme), 1)),
        )
        .child(
            row()
                .gap(8.0)
                .child(interactions::button(state.buttons[2].view(theme), 2))
                .child(interactions::button(state.buttons[3].view(theme), 3)),
        )
}

pub fn section(state: &State, theme: &Theme) -> El {
    col()
        .gap(12.0)
        .child(avatars(state))
        .child(buttons(state, theme))
        .child(el().radius(8.0).child(state.gradient.view()))
        .child(widgets::caption("gradient · blue to pink, 6 levels", theme))
}
