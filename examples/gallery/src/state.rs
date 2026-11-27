//! the demo's state, scene list and per-scene sizing

use std::cell::Cell;

use dither_core::color::DitherColor;
use dither_core::gradient::{Direction, GradientSpec};
use dither_core::paint::Variant;
use dither_core::stack::StackMode;
use dither_kit_rune::avatar::Avatar;
use dither_kit_rune::button::{ButtonSpec, DitherButton};
use dither_kit_rune::cartesian::ChartKind;
use dither_kit_rune::chart::{Chart, ChartSeries, ChartSpec, Margins};
use dither_kit_rune::gradient::DitherGradient;
use dither_kit_rune::pie::{Pie, PieSeries, PieSpec};
use dither_kit_rune::radar::{Radar, RadarSeries, RadarSpec};

pub const BTN_REPLAY: usize = 0;
pub const BTN_CLEAR: usize = 1;
pub const BTN_THEME: usize = 2;
pub const BTN_DISABLED: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scene {
    Dashboard,
    Area,
    Bar,
    Sparkline,
    Donut,
    Radar,
    Avatars,
    Buttons,
    Gradient,
}

impl Scene {
    pub const ALL: [Scene; 9] = [
        Scene::Dashboard,
        Scene::Area,
        Scene::Bar,
        Scene::Sparkline,
        Scene::Donut,
        Scene::Radar,
        Scene::Avatars,
        Scene::Buttons,
        Scene::Gradient,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Scene::Dashboard => "dashboard",
            Scene::Area => "area",
            Scene::Bar => "bar",
            Scene::Sparkline => "sparkline",
            Scene::Donut => "donut",
            Scene::Radar => "radar",
            Scene::Avatars => "avatars",
            Scene::Buttons => "buttons",
            Scene::Gradient => "gradient",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }

    pub fn step(self, delta: isize) -> Self {
        let len = Self::ALL.len() as isize;
        let index = (self.index() as isize + delta).rem_euclid(len);
        Self::ALL[index as usize]
    }

    pub const fn is_showcase(self) -> bool {
        !matches!(self, Scene::Dashboard)
    }
}

pub struct State {
    pub scene: Scene,
    pub dark: bool,
    /// mirrors the OS setting, read from `Cx` each frame
    pub reduced: Cell<bool>,
    pub area: Chart,
    pub bar: Chart,
    pub spark_a: Chart,
    pub spark_b: Chart,
    pub pie: Pie,
    pub radar: Radar,
    pub avatars: [Avatar; 4],
    pub buttons: [DitherButton; 4],
    pub gradient: DitherGradient,
}

fn months() -> Vec<String> {
    [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]
    .map(String::from)
    .to_vec()
}

fn area_chart(size: (f32, f32)) -> Chart {
    let desktop = [
        38.0, 42.0, 55.0, 48.0, 63.0, 58.0, 71.0, 66.0, 74.0, 62.0, 69.0, 57.0,
    ];
    let mobile = [
        22.0, 26.0, 31.0, 30.0, 38.0, 44.0, 41.0, 49.0, 46.0, 52.0, 47.0, 55.0,
    ];
    Chart::new(
        ChartSpec::new(
            size,
            months(),
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

fn bar_chart(size: (f32, f32)) -> Chart {
    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        .map(String::from)
        .to_vec();
    let builds = [42.0, 58.0, 61.0, 47.0, 73.0, 68.0, 52.0];
    let deploys = [31.0, 44.0, 39.0, 51.0, 46.0, 60.0, 41.0];
    Chart::new(
        ChartSpec::new(
            size,
            days,
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

fn sparkline(key: &str, color: DitherColor, phase: f64, size: (f32, f32)) -> Chart {
    let values: Vec<f64> = (0..26)
        .map(|i| {
            let x = i as f64 * 0.5 + phase;
            30.0 + x.sin() * 14.0 + (x * 0.6).cos() * 8.0
        })
        .collect();
    let categories: Vec<String> = (0..values.len()).map(|i| i.to_string()).collect();
    Chart::new(
        ChartSpec::new(
            size,
            categories,
            vec![ChartSeries::new(key, color)],
            vec![values],
        )
        .key(key)
        .sparkline(),
    )
}

fn pie_chart(size: (f32, f32)) -> Pie {
    Pie::new(
        PieSpec::new(
            size,
            vec![
                PieSeries::new("rust", 38.0, DitherColor::Orange),
                PieSeries::new("typescript", 27.0, DitherColor::Blue),
                PieSeries::new("go", 19.0, DitherColor::Green),
                PieSeries::new("zig", 11.0, DitherColor::Purple),
                PieSeries::new("elixir", 5.0, DitherColor::Pink),
            ],
        )
        .key("pie-chart")
        .inner(0.52),
    )
}

fn radar_chart(size: (f32, f32)) -> Radar {
    Radar::new(
        RadarSpec::new(
            size,
            ["speed", "size", "docs", "tests", "perf"]
                .map(String::from)
                .to_vec(),
            vec![
                RadarSeries::new("v1", DitherColor::Blue),
                RadarSeries::new("v2", DitherColor::Pink).variant(Variant::Dotted),
            ],
            vec![vec![7.0, 6.0, 5.0, 8.0, 6.0], vec![9.0, 8.0, 9.0, 7.0, 9.0]],
        )
        .key("radar-chart"),
    )
}

fn avatars(size: f32) -> [Avatar; 4] {
    [
        Avatar::new("ada", size),
        Avatar::new("Ada Lovelace", size),
        Avatar::new("🦀", size),
        Avatar::new("grace hopper", size),
    ]
}

fn buttons(size: (f32, f32)) -> [DitherButton; 4] {
    [
        DitherButton::new(ButtonSpec::new("replay", size).color(DitherColor::Blue)),
        DitherButton::new(
            ButtonSpec::new("clear", size)
                .color(DitherColor::Purple)
                .variant(Variant::Dotted),
        ),
        DitherButton::new(
            ButtonSpec::new("theme", size)
                .color(DitherColor::Pink)
                .variant(Variant::Hatched),
        ),
        DitherButton::new(
            ButtonSpec::new("disabled", size)
                .color(DitherColor::Grey)
                .variant(Variant::Solid)
                .disabled(true),
        ),
    ]
}

fn gradient(size: (f32, f32)) -> DitherGradient {
    DitherGradient::new(
        size,
        GradientSpec::new(DitherColor::Blue.fill())
            .to(DitherColor::Pink.fill())
            .direction(Direction::Up)
            .cell(3.0)
            .opacity(0.9),
    )
}

#[derive(Clone, Copy)]
struct Sizes {
    area: (f32, f32),
    bar: (f32, f32),
    spark: (f32, f32),
    pie: (f32, f32),
    radar: (f32, f32),
    avatar: f32,
    button: (f32, f32),
    gradient: (f32, f32),
}

fn sizes(scene: Scene) -> Sizes {
    if scene.is_showcase() {
        Sizes {
            area: (820.0, 320.0),
            bar: (820.0, 300.0),
            spark: (820.0, 190.0),
            pie: (320.0, 300.0),
            radar: (380.0, 320.0),
            avatar: 88.0,
            button: (160.0, 44.0),
            gradient: (820.0, 220.0),
        }
    } else {
        Sizes {
            area: (417.0, 172.0),
            bar: (417.0, 172.0),
            spark: (417.0, 64.0),
            pie: (417.0, 172.0),
            radar: (417.0, 172.0),
            avatar: 64.0,
            button: (140.0, 32.0),
            gradient: (417.0, 110.0),
        }
    }
}

impl State {
    pub fn set_scene(&mut self, scene: Scene) {
        let sizes = sizes(scene);
        self.scene = scene;
        self.area = area_chart(sizes.area);
        self.bar = bar_chart(sizes.bar);
        self.spark_a = sparkline("spark-a", DitherColor::Green, 0.0, sizes.spark);
        self.spark_b = sparkline("spark-b", DitherColor::Pink, 1.7, sizes.spark);
        self.pie = pie_chart(sizes.pie);
        self.radar = radar_chart(sizes.radar);
        self.avatars = avatars(sizes.avatar);
        self.buttons = buttons(sizes.button);
        self.gradient = gradient(sizes.gradient);
    }

    pub fn press(&mut self, index: usize) {
        match index {
            BTN_REPLAY => self.replay_all(),
            BTN_CLEAR => {
                self.area.clear_selection();
                self.bar.clear_selection();
                self.pie.clear_selection();
                self.radar.clear_selection();
            }
            BTN_THEME => self.dark = !self.dark,
            BTN_DISABLED => {}
            _ => {}
        }
    }

    pub fn replay_all(&mut self) {
        self.area.replay();
        self.bar.replay();
        self.pie.replay();
        self.radar.replay();
        for avatar in &mut self.avatars {
            avatar.replay();
        }
    }
}

pub fn init() -> State {
    let sizes = sizes(Scene::Dashboard);
    State {
        scene: Scene::Dashboard,
        dark: true,
        reduced: Cell::new(false),
        area: area_chart(sizes.area),
        bar: bar_chart(sizes.bar),
        spark_a: sparkline("spark-a", DitherColor::Green, 0.0, sizes.spark),
        spark_b: sparkline("spark-b", DitherColor::Pink, 1.7, sizes.spark),
        pie: pie_chart(sizes.pie),
        radar: radar_chart(sizes.radar),
        avatars: avatars(sizes.avatar),
        buttons: buttons(sizes.button),
        gradient: gradient(sizes.gradient),
    }
}
