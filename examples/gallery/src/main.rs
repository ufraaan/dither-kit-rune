//! Dither Kit gallery: a dashboard plus one screenshot-ready scene per component.
//!
//! `cargo run -p gallery` opens the dashboard. `-- --scene area` (or any scene
//! name) opens one component; ← and → cycle scenes. `-- --snap out.png --at 2
//! --scene area` renders one headless.

mod interactions;
mod section_atoms;
mod section_cartesian;
mod section_polar;
mod showcase;
mod state;
mod view;
mod widgets;

use std::time::Duration;

use rune::{AppSpec, KeyEvent, Tx};

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

fn on_key(state: &mut state::State, event: &KeyEvent, _: &mut Tx) -> bool {
    let delta = match event.key {
        rune::Key::Left => -1,
        rune::Key::Right => 1,
        _ => return false,
    };
    let next = state.scene.step(delta);
    state.set_scene(next);
    true
}

macro_rules! scene {
    ($name:ident, $scene:expr) => {
        fn $name(state: &mut state::State) {
            state.set_scene($scene);
        }
    };
}

scene!(scene_dashboard, state::Scene::Dashboard);
scene!(scene_area, state::Scene::Area);
scene!(scene_bar, state::Scene::Bar);
scene!(scene_sparkline, state::Scene::Sparkline);
scene!(scene_donut, state::Scene::Donut);
scene!(scene_radar, state::Scene::Radar);
scene!(scene_avatars, state::Scene::Avatars);
scene!(scene_buttons, state::Scene::Buttons);
scene!(scene_gradient, state::Scene::Gradient);

fn light(state: &mut state::State) {
    state.dark = false;
}

fn reduced(state: &mut state::State) {
    state.reduced.set(true);
}

fn main() {
    rune::run(AppSpec::<state::State> {
        scenes: &[
            ("dashboard", scene_dashboard),
            ("area", scene_area),
            ("bar", scene_bar),
            ("sparkline", scene_sparkline),
            ("donut", scene_donut),
            ("radar", scene_radar),
            ("avatars", scene_avatars),
            ("buttons", scene_buttons),
            ("gradient", scene_gradient),
            ("light", light),
            ("reduced", reduced),
        ],
        poll: Some(poll),
        on_key: Some(on_key),
        ..AppSpec::new(
            "Dither Kit Gallery",
            (960.0, 640.0),
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
            (960.0, 640.0),
            Env {
                system_dark: true,
                scale: 2.0,
            },
        );
        rt.frame(0.0);
        rt
    }

    #[test]
    fn the_dashboard_composes_every_component() {
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
    fn entering_a_showcase_scene_resizes_its_component() {
        let mut state = state::init();
        state.set_scene(state::Scene::Area);
        assert_eq!(state.area.texture().pixels().size(), (386, 142));
        assert_eq!(state.scene, state::Scene::Area);

        let mut rt = Runtime::new(
            state,
            view::page,
            (960.0, 640.0),
            Env {
                system_dark: true,
                scale: 2.0,
            },
        );
        rt.frame(0.0);
        assert!(rt.find("area-chart").is_some());
    }

    #[test]
    fn scenes_cycle_in_both_directions() {
        assert_eq!(state::Scene::Dashboard.step(1), state::Scene::Area);
        assert_eq!(state::Scene::Dashboard.step(-1), state::Scene::Gradient);
        assert_eq!(state::Scene::Avatars.step(1), state::Scene::Buttons);
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
