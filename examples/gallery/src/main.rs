//! Dither Kit gallery: one deterministic scene with every component.
//!
//! `cargo run -p gallery` opens the window; `-- --snap gallery.png --at 2`
//! renders it headless.

mod section_atoms;
mod section_cartesian;
mod section_polar;
mod state;
mod view;
mod widgets;

use std::time::Duration;

use rune::{AppSpec, Tx};

fn poll(state: &mut state::State, clock: f64, tx: &mut Tx) -> bool {
    if state.reduced.get() {
        let mut changed = state.area.finish();
        changed |= state.bar.finish();
        changed |= state.spark_a.finish();
        changed |= state.spark_b.finish();
        changed |= state.pie.finish();
        changed |= state.radar.finish();
        for avatar in &mut state.avatars {
            changed |= avatar.finish();
        }
        return changed;
    }

    let mut busy = state.area.tick(clock);
    busy |= state.bar.tick(clock);
    busy |= state.spark_a.tick(clock);
    busy |= state.spark_b.tick(clock);
    busy |= state.pie.tick(clock);
    busy |= state.radar.tick(clock);
    for avatar in &mut state.avatars {
        busy |= avatar.tick(clock);
    }
    if busy {
        tx.wake_after(Duration::ZERO);
    }
    busy
}

fn light(state: &mut state::State) {
    state.dark = false;
}

fn reduced(state: &mut state::State) {
    state.reduced.set(true);
}

fn main() {
    rune::run(AppSpec::<state::State> {
        scenes: &[("light", light), ("reduced", reduced)],
        poll: Some(poll),
        ..AppSpec::new(
            "Dither Kit Gallery",
            (1040.0, 720.0),
            state::init,
            view::page,
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};

    fn runtime() -> Runtime<state::State> {
        let mut rt = Runtime::new(
            state::init(),
            view::page,
            (1040.0, 720.0),
            Env {
                system_dark: true,
                scale: 2.0,
            },
        );
        rt.frame(0.0);
        rt
    }

    #[test]
    fn every_component_paints_at_its_own_backing_size() {
        let state = state::init();
        assert_eq!(state.area.texture().pixels().size(), (271, 77));
        assert_eq!(state.pie.texture().pixels().size(), (145, 90));
        assert_eq!(state.avatars[0].progress(), 0.0);
    }

    #[test]
    fn the_scene_builds_with_every_key() {
        let rt = runtime();
        for key in [
            "area-chart",
            "bar-chart",
            "pie-chart",
            "radar-chart",
            "spark-a",
            "spark-b",
        ] {
            assert!(rt.find(key).is_some(), "{key} missing");
        }
    }

    #[test]
    fn hovering_a_slice_and_a_spoke_opens_their_tooltips() {
        let mut rt = runtime();
        let (px, py) = rt.find("pie-chart").unwrap();
        rt.pointer_move(px + 60.0, py);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.pie.hover(), Some(0));
        assert!(rt.find("pie-chart-tooltip").is_some());

        let (rx, ry) = rt.find("radar-chart").unwrap();
        rt.pointer_move(rx + 50.0, ry);
        rt.frame(1.0 / 60.0);
        assert!(rt.state.radar.hover().is_some());
    }

    #[test]
    fn reduced_motion_settles_the_entrances_without_ticking() {
        let mut state = state::init();
        state.reduced.set(true);
        assert!(poll(&mut state, 1.0, &mut Tx::default()));
        assert_eq!(state.area.progress(), 1.0);
        assert_eq!(state.pie.progress(), 1.0);
        assert_eq!(state.avatars[0].progress(), 1.0);
    }

    #[test]
    fn the_entrances_settle_on_the_clock() {
        let mut state = state::init();
        assert!(state.area.tick(0.0));
        assert!(state.pie.tick(0.0));
        assert!(!state.area.tick(5.0));
        assert!(!state.pie.tick(5.0));
        assert_eq!(state.area.progress(), 1.0);
        assert_eq!(state.pie.progress(), 1.0);
    }
}
