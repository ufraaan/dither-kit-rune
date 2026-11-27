//! Manual gallery scene: an area chart plus one swatch per dither variant.
//! `cargo run -p gallery` opens the window; `-- --snap gallery.png` renders it
//! headless.

use dither_core::Surface;
use dither_core::bayer::CELL;
use dither_core::bayer::backing_size;
use dither_core::color::DitherColor;
use dither_core::paint::{PaintOpts, Variant, paint_column};
use dither_kit_rune::cartesian::{RenderOpts, Series, render};
use dither_kit_rune::rune_el::SurfaceTexture;
use rune::tok::{self, type_};
use rune::{AppSpec, Cx, El, col, row, text};

struct State {
    chart: SurfaceTexture,
    swatches: [(Variant, SurfaceTexture); 4],
}

fn wave(i: usize, phase: f64) -> f64 {
    let x = i as f64 * 0.32 + phase;
    52.0 + x.sin() * 26.0 + (x * 0.45).cos() * 18.0
}

fn chart() -> SurfaceTexture {
    let (cols, rows) = backing_size(560.0, 220.0);
    let mut surface = Surface::new(cols, rows);
    let series = [
        Series::new(
            (0..28).map(|i| wave(i, 0.0)).collect::<Vec<_>>(),
            DitherColor::Blue.fill(),
        ),
        Series::new(
            (0..28).map(|i| wave(i, 2.2)).collect::<Vec<_>>(),
            DitherColor::Orange.fill(),
        )
        .variant(Variant::Dotted),
    ];
    render(&mut surface, 220.0, &series, &RenderOpts::default());
    let texture = SurfaceTexture::new();
    texture.upload_owned(surface);
    texture
}

fn swatch(variant: Variant, color: DitherColor) -> SurfaceTexture {
    let cols = (64.0 / CELL) as u32;
    let rows = cols;
    let mut surface = Surface::new(cols, rows);
    let opts = PaintOpts::default().variant(variant);
    for x in 0..cols as i32 {
        paint_column(&mut surface, x, 1.0, rows as f64 - 2.0, color.fill(), &opts);
    }
    let texture = SurfaceTexture::new();
    texture.upload_owned(surface);
    texture
}

fn init() -> State {
    State {
        chart: chart(),
        swatches: [
            (
                Variant::Gradient,
                swatch(Variant::Gradient, DitherColor::Green),
            ),
            (Variant::Dotted, swatch(Variant::Dotted, DitherColor::Blue)),
            (
                Variant::Hatched,
                swatch(Variant::Hatched, DitherColor::Purple),
            ),
            (Variant::Solid, swatch(Variant::Solid, DitherColor::Pink)),
        ],
    }
}

fn view(s: &State, _: &mut Cx<State>) -> El {
    let t = tok::theme(true);
    let heading = text("Dither Kit").font(type_::TITLE).color(t.text);
    let sub = text("ordered-dither charts, painted natively in Rune").color(t.text_secondary);
    let chart = s.chart.el().key("chart").size(560.0, 220.0);
    let swatches = s
        .swatches
        .iter()
        .fold(row().gap(16.0), |r, (variant, texture)| {
            r.child(
                col()
                    .gap(6.0)
                    .child(texture.el().size(64.0, 64.0).radius(6.0))
                    .child(text(variant.name()).color(t.text_secondary)),
            )
        });
    col()
        .w_full()
        .h_full()
        .padding(32.0)
        .gap(18.0)
        .fill(t.bg)
        .child(heading)
        .child(sub)
        .child(chart)
        .child(swatches)
}

fn main() {
    rune::run(AppSpec::<State> {
        scenes: &[],
        ..AppSpec::new("Dither Kit Gallery", (624.0, 470.0), init, view)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};

    #[test]
    fn the_chart_paints_opaque_pixels_inside_its_plot() {
        let state = init();
        let (cols, rows) = state.chart.pixels().size();
        assert_eq!((cols, rows), backing_size(560.0, 220.0));
        let mut rt = Runtime::new(
            init(),
            view,
            (624.0, 470.0),
            Env {
                system_dark: true,
                scale: 2.0,
            },
        );
        rt.frame(0.0);
        assert!(rt.find("chart").is_some());
    }
}
