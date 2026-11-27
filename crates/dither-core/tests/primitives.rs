//! Deterministic primitives checked against values produced by the upstream
//! TypeScript at the pinned revision (`scripts/gen-fixtures.mjs`).

use dither_core::bayer::{backing_size, clamp01, ease_in_out_cubic, ease_out_cubic, threshold};
use dither_core::color::{DitherColor, PixelColor, Rgb, fill_of, hue_fill};
use dither_core::paint::resample;
use dither_core::rand::{XorShift32, fnv1a};
use dither_core::scales::{BandScale, LinearScale, PointScale, index_at_band, nearest_index};
use dither_core::stack::{StackMode, compute_bands};

fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{a} != {b}");
}

#[test]
fn hash_matches_js_over_utf16_code_units() {
    let names = ["", "ada", "Ada Lovelace", "jürgen", "🦀 rust"];
    let expected = [2166136261, 943263163, 4267605246, 473044655, 2052601259];
    for (name, want) in names.iter().zip(expected) {
        assert_eq!(fnv1a(name), want, "{name}");
    }
}

#[test]
fn xorshift32_matches_the_js_sequence() {
    let expected = [
        0.00006295018829405308,
        0.015747428173199296,
        0.6164041024167091,
        0.07161863497458398,
        0.5584883580449969,
    ];
    let mut rng = XorShift32::new(1);
    for want in expected {
        assert_eq!(rng.next_f64(), want);
    }

    let expected_zero = [
        0.31659353361465037,
        0.8757069851271808,
        0.4833001629449427,
        0.005915207089856267,
        0.899222202366218,
    ];
    let mut rng = XorShift32::new(0);
    for want in expected_zero {
        assert_eq!(rng.next_f64(), want);
    }
}

#[test]
fn hue_fill_matches_hsl_85_58() {
    assert_eq!(hue_fill(0.0), Rgb::new(239, 57, 57));
    assert_eq!(hue_fill(60.0), Rgb::new(239, 239, 57));
    assert_eq!(hue_fill(120.0), Rgb::new(57, 239, 57));
    assert_eq!(hue_fill(240.0), Rgb::new(57, 57, 239));
    assert_eq!(hue_fill(359.5), Rgb::new(239, 57, 58));
    assert_eq!(hue_fill(-30.0), Rgb::new(239, 57, 148));
}

#[test]
fn named_palette_fills_match_upstream() {
    assert_eq!(DitherColor::Blue.fill(), Rgb::new(53, 143, 243));
    assert_eq!(DitherColor::Grey.fill(), Rgb::new(92, 92, 100));
    assert_eq!(
        fill_of(PixelColor::Named(DitherColor::Green)),
        Rgb::new(40, 210, 110)
    );
    assert_eq!(DitherColor::from_name("purple"), Some(DitherColor::Purple));
    assert_eq!(DitherColor::from_name("mauve"), None);
}

#[test]
fn bayer_thresholds_are_normalized_and_wrap() {
    close(threshold(0, 0), 0.03125);
    close(threshold(1, 0), 0.53125);
    close(threshold(3, 3), 0.34375);
    close(threshold(4, 4), threshold(0, 0));
    close(threshold(-1, -1), threshold(3, 3));
    close(clamp01(-1.0), 0.0);
    close(clamp01(2.0), 1.0);
}

#[test]
fn easings_match_upstream() {
    close(ease_in_out_cubic(0.0), 0.0);
    close(ease_in_out_cubic(0.5), 0.5);
    close(ease_in_out_cubic(1.0), 1.0);
    close(ease_in_out_cubic(0.25), 0.0625);
    close(ease_out_cubic(0.5), 0.875);
}

#[test]
fn backing_size_halves_points_and_clamps() {
    assert_eq!(backing_size(560.0, 220.0), (280, 110));
    assert_eq!(backing_size(10.0, 10.0), (8, 8));
    assert_eq!(backing_size(4000.0, 4000.0), (520, 200));
}

#[test]
fn resample_matches_linear_interpolation() {
    assert_eq!(
        resample(&[0.0, 10.0, 30.0, 5.0], 7),
        vec![0.0, 5.0, 10.0, 20.0, 30.0, 17.5, 5.0]
    );
    assert_eq!(resample(&[], 4), vec![0.0; 4]);
    assert_eq!(resample(&[7.0], 5), vec![7.0, 7.0, 7.0, 7.0, 0.0]);
}

#[test]
fn point_and_band_scales_match_d3() {
    let point = PointScale::new(5, 500.0);
    assert_eq!(point.position(0), 0.0);
    assert_eq!(point.position(4), 500.0);
    close(point.position(2), 250.0);

    let band = BandScale::new(5, 500.0);
    close(band.step(), 98.4251968503937);
    close(band.bandwidth(), 70.86614173228347);
    close(band.position(0), 17.716535433070874);
    close(band.position(4), 411.4173228346457);
}

#[test]
fn hit_testing_prefers_the_nearest_row_and_band() {
    assert_eq!(nearest_index(0.0, 5, 500.0), 0);
    assert_eq!(nearest_index(500.0, 5, 500.0), 4);
    assert_eq!(nearest_index(255.0, 5, 500.0), 2);
    assert_eq!(index_at_band(0.0, 5, 500.0), 0);
    assert_eq!(index_at_band(499.0, 5, 500.0), 4);
    assert_eq!(index_at_band(500.0, 5, 500.0), 4);
}

#[test]
fn linear_scale_includes_zero_and_nices_like_d3() {
    let scale = LinearScale::new(0.0, 52.0, 200.0);
    assert_eq!(scale.domain(), (0.0, 55.0));
    close(scale.y(0.0), 200.0);
    close(scale.y(26.0), 105.45454545454547);
    close(scale.y(52.0), 10.909090909090914);

    let diverging = LinearScale::new(-2.0, 4.0, 100.0);
    assert_eq!(diverging.domain(), (-2.0, 4.0));

    // Upstream widens an all-zero domain to a unit span before nice().
    let flat = LinearScale::new(0.0, 0.0, 100.0);
    assert_eq!(flat.domain(), (0.0, 1.0));

    assert_eq!(
        LinearScale::new(0.0, 52.0, 200.0).ticks(10),
        vec![
            0.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0, 55.0
        ]
    );
}

#[test]
fn default_stacking_sits_every_series_on_zero() {
    let bands = compute_bands(&[vec![3.0, -2.0], vec![1.0, 4.0]], StackMode::None);
    assert_eq!(bands.bands[0], vec![[0.0, 3.0], [0.0, -2.0]]);
    assert_eq!(bands.bands[1], vec![[0.0, 1.0], [0.0, 4.0]]);
    assert_eq!((bands.min, bands.max), (-2.0, 4.0));
}

#[test]
fn stacked_bands_accumulate_positives_and_negatives() {
    let bands = compute_bands(
        &[vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]],
        StackMode::Stacked,
    );
    assert_eq!(bands.bands[0], vec![[0.0, 1.0], [0.0, 2.0], [0.0, 3.0]]);
    assert_eq!(bands.bands[1], vec![[1.0, 5.0], [2.0, 7.0], [3.0, 9.0]]);
    assert_eq!(bands.max, 9.0);

    let diverging = compute_bands(&[vec![3.0, -2.0], vec![1.0, 4.0]], StackMode::Stacked);
    assert_eq!(diverging.bands[1], vec![[3.0, 4.0], [-2.0, 2.0]]);
    assert_eq!((diverging.min, diverging.max), (-2.0, 4.0));
}

#[test]
fn percent_stacking_normalizes_each_row() {
    let bands = compute_bands(
        &[vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]],
        StackMode::Percent,
    );
    close(bands.bands[0][0][1], 0.2);
    close(bands.bands[1][0][1], 1.0);
    close(bands.bands[0][1][1], 2.0 / 7.0);
    close(bands.bands[1][1][1], 1.0);
    close(bands.bands[0][2][1], 1.0 / 3.0);
    close(bands.bands[1][2][1], 1.0);
    assert_eq!((bands.min, bands.max), (0.0, 1.0));
}

#[test]
fn empty_and_non_finite_data_stay_finite() {
    let empty = compute_bands(&[], StackMode::None);
    assert_eq!((empty.max, empty.min), (1.0, 0.0));
    let nan = compute_bands(&[vec![f64::NAN, f64::INFINITY]], StackMode::Stacked);
    assert_eq!(nan.bands[0], vec![[0.0, 0.0], [0.0, 0.0]]);
    assert_eq!((nan.max, nan.min), (1.0, 0.0));
}
