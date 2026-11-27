//! the demo's state and data

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

pub struct State {
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

fn area_chart() -> Chart {
    let desktop = [
        38.0, 42.0, 55.0, 48.0, 63.0, 58.0, 71.0, 66.0, 74.0, 62.0, 69.0, 57.0,
    ];
    let mobile = [
        22.0, 26.0, 31.0, 30.0, 38.0, 44.0, 41.0, 49.0, 46.0, 52.0, 47.0, 55.0,
    ];
    Chart::new(
        ChartSpec::new(
            (590.0, 190.0),
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

fn bar_chart() -> Chart {
    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        .map(String::from)
        .to_vec();
    let builds = [42.0, 58.0, 61.0, 47.0, 73.0, 68.0, 52.0];
    let deploys = [31.0, 44.0, 39.0, 51.0, 46.0, 60.0, 41.0];
    Chart::new(
        ChartSpec::new(
            (590.0, 170.0),
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

fn sparkline(key: &str, color: DitherColor, phase: f64) -> Chart {
    let values: Vec<f64> = (0..26)
        .map(|i| {
            let x = i as f64 * 0.5 + phase;
            30.0 + x.sin() * 14.0 + (x * 0.6).cos() * 8.0
        })
        .collect();
    let categories: Vec<String> = (0..values.len()).map(|i| i.to_string()).collect();
    Chart::new(
        ChartSpec::new(
            (270.0, 60.0),
            categories,
            vec![ChartSeries::new(key, color)],
            vec![values],
        )
        .key(key)
        .sparkline(),
    )
}

fn pie_chart() -> Pie {
    Pie::new(
        PieSpec::new(
            (290.0, 180.0),
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

fn radar_chart() -> Radar {
    Radar::new(
        RadarSpec::new(
            (290.0, 180.0),
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

fn avatars() -> [Avatar; 4] {
    [
        Avatar::new("ada", 40.0),
        Avatar::new("Ada Lovelace", 40.0),
        Avatar::new("🦀", 40.0),
        Avatar::new("grace hopper", 40.0),
    ]
}

fn buttons() -> [DitherButton; 4] {
    [
        DitherButton::new(ButtonSpec::new("replay", (90.0, 28.0)).color(DitherColor::Blue)),
        DitherButton::new(
            ButtonSpec::new("clear", (90.0, 28.0))
                .color(DitherColor::Purple)
                .variant(Variant::Dotted),
        ),
        DitherButton::new(
            ButtonSpec::new("theme", (90.0, 28.0))
                .color(DitherColor::Pink)
                .variant(Variant::Hatched),
        ),
        DitherButton::new(
            ButtonSpec::new("disabled", (90.0, 28.0))
                .color(DitherColor::Grey)
                .variant(Variant::Solid)
                .disabled(true),
        ),
    ]
}

fn gradient() -> DitherGradient {
    DitherGradient::new(
        (290.0, 60.0),
        GradientSpec::new(DitherColor::Blue.fill())
            .to(DitherColor::Pink.fill())
            .direction(Direction::Up)
            .cell(3.0)
            .opacity(0.9),
    )
}

impl State {
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
    State {
        dark: true,
        reduced: Cell::new(false),
        area: area_chart(),
        bar: bar_chart(),
        spark_a: sparkline("spark-a", DitherColor::Green, 0.0),
        spark_b: sparkline("spark-b", DitherColor::Pink, 1.7),
        pie: pie_chart(),
        radar: radar_chart(),
        avatars: avatars(),
        buttons: buttons(),
        gradient: gradient(),
    }
}
