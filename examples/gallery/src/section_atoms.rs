//! avatars, buttons and the gradient wash

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

fn button(state: &State, theme: &Theme, index: usize) -> El {
    state.buttons[index]
        .view(theme)
        .on_hover_change(move |state: &mut State, inside, _| state.buttons[index].hover(inside))
        .on_press(move |state: &mut State| state.press(index))
}

fn buttons(state: &State, theme: &Theme) -> El {
    col()
        .gap(8.0)
        .child(
            row()
                .gap(8.0)
                .child(button(state, theme, 0))
                .child(button(state, theme, 1)),
        )
        .child(
            row()
                .gap(8.0)
                .child(button(state, theme, 2))
                .child(button(state, theme, 3)),
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
