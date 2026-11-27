//! pointer and key handlers, shared by the dashboard and the showcase

use crate::state::State;
use rune::{El, Key};

pub fn cartesian(chart: El, area: bool) -> El {
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

pub fn sparkline(chart: El, second: bool) -> El {
    chart
        .on_pointer_move_local(move |state: &mut State, p, _| {
            if second {
                state.spark_b.hover_at(p.x);
            } else {
                state.spark_a.hover_at(p.x);
            }
        })
        .on_hover_change(move |state: &mut State, inside, _| {
            if !inside {
                if second {
                    state.spark_b.leave();
                } else {
                    state.spark_a.leave();
                }
            }
        })
}

pub fn pie(chart: El) -> El {
    chart
        .on_pointer_move_local(|state: &mut State, p, _| state.pie.hover_at(p.x, p.y))
        .on_hover_change(|state: &mut State, inside, _| {
            if !inside {
                state.pie.leave();
            }
        })
}

pub fn radar(chart: El) -> El {
    chart
        .on_pointer_move_local(|state: &mut State, p, _| state.radar.hover_at(p.x, p.y))
        .on_hover_change(|state: &mut State, inside, _| {
            if !inside {
                state.radar.leave();
            }
        })
}

pub fn button(chart: El, index: usize) -> El {
    chart
        .on_hover_change(move |state: &mut State, inside, _| state.buttons[index].hover(inside))
        .on_press(move |state: &mut State| state.press(index))
}
