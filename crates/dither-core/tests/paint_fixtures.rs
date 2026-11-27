//! Byte-for-byte painter parity with upstream's `paintColumn`.
//!
//! Expected buffers were produced by running the pinned `dither-paint.ts` in
//! Node with a straight-alpha canvas shim; regenerate them with
//! `scripts/gen-fixtures.mjs` when the upstream pin moves.

use dither_core::Surface;
use dither_core::color::DitherColor;
use dither_core::paint::{PaintOpts, Variant, paint_column};

struct Case {
    name: &'static str,
    expected: &'static [u8],
    x: i32,
    top: f64,
    floor: f64,
    variant: Variant,
    intensity: f64,
    dim: f64,
    stacked: bool,
    sparse: f64,
}

const COLS: u32 = 8;
const ROWS: u32 = 12;

fn cases() -> Vec<Case> {
    vec![
        Case {
            name: "gradient",
            expected: include_bytes!("fixtures/gradient.rgba"),
            x: 1,
            top: 1.4,
            floor: 9.6,
            variant: Variant::Gradient,
            intensity: 0.0,
            dim: 1.0,
            stacked: false,
            sparse: 0.0,
        },
        Case {
            name: "gradient-hover",
            expected: include_bytes!("fixtures/gradient-hover.rgba"),
            x: 2,
            top: 2.2,
            floor: 10.4,
            variant: Variant::Gradient,
            intensity: 0.7,
            dim: 1.0,
            stacked: false,
            sparse: 0.0,
        },
        Case {
            name: "dotted",
            expected: include_bytes!("fixtures/dotted.rgba"),
            x: 3,
            top: 1.6,
            floor: 10.2,
            variant: Variant::Dotted,
            intensity: 0.0,
            dim: 1.0,
            stacked: false,
            sparse: 0.0,
        },
        Case {
            name: "hatched",
            expected: include_bytes!("fixtures/hatched.rgba"),
            x: 4,
            top: 2.6,
            floor: 11.2,
            variant: Variant::Hatched,
            intensity: 0.0,
            dim: 1.0,
            stacked: false,
            sparse: 0.0,
        },
        Case {
            name: "solid",
            expected: include_bytes!("fixtures/solid.rgba"),
            x: 5,
            top: 3.4,
            floor: 11.6,
            variant: Variant::Solid,
            intensity: 0.0,
            dim: 1.0,
            stacked: false,
            sparse: 0.0,
        },
        Case {
            name: "stacked",
            expected: include_bytes!("fixtures/stacked.rgba"),
            x: 6,
            top: 1.2,
            floor: 8.8,
            variant: Variant::Gradient,
            intensity: 0.0,
            dim: 1.0,
            stacked: true,
            sparse: 0.0,
        },
        Case {
            name: "dimmed",
            expected: include_bytes!("fixtures/dimmed.rgba"),
            x: 0,
            top: 2.4,
            floor: 9.2,
            variant: Variant::Gradient,
            intensity: 0.0,
            dim: 0.3,
            stacked: false,
            sparse: 0.14,
        },
        Case {
            name: "zero-depth",
            expected: include_bytes!("fixtures/zero-depth.rgba"),
            x: 7,
            top: 5.6,
            floor: 5.6,
            variant: Variant::Gradient,
            intensity: 0.0,
            dim: 1.0,
            stacked: false,
            sparse: 0.0,
        },
        Case {
            name: "negative-top",
            expected: include_bytes!("fixtures/negative-top.rgba"),
            x: 2,
            top: -2.2,
            floor: 4.4,
            variant: Variant::Gradient,
            intensity: 0.0,
            dim: 1.0,
            stacked: false,
            sparse: 0.0,
        },
    ]
}

#[test]
fn every_painter_case_matches_the_upstream_canvas_bytes() {
    for case in cases() {
        let mut surface = Surface::new(COLS, ROWS);
        let opts = PaintOpts {
            variant: case.variant,
            intensity: case.intensity,
            dim: case.dim,
            stacked: case.stacked,
            sparse: case.sparse,
        };
        paint_column(
            &mut surface,
            case.x,
            case.top,
            case.floor,
            DitherColor::Blue.fill(),
            &opts,
        );
        assert_eq!(
            surface.rgba(),
            case.expected,
            "{} differs from upstream",
            case.name
        );
    }
}

#[test]
fn painting_stays_inside_the_buffer() {
    let mut surface = Surface::new(4, 4);
    paint_column(
        &mut surface,
        -3,
        -5.0,
        9.0,
        DitherColor::Red.fill(),
        &PaintOpts::default(),
    );
    paint_column(
        &mut surface,
        9,
        1.0,
        3.0,
        DitherColor::Red.fill(),
        &PaintOpts::default(),
    );
    assert_eq!(surface.rgba().len(), 4 * 4 * 4);
}
