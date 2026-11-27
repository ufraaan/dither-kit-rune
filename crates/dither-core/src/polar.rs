//! Polar geometry for pie and radar charts. Angles start at the top and run
//! clockwise, matching how slices and spokes read on screen.

use std::f64::consts::{FRAC_PI_2, PI};

pub use std::f64::consts::TAU;

/// Angle of the first slice/spoke.
pub const TOP: f64 = -FRAC_PI_2;

#[derive(Clone, Debug, PartialEq)]
pub struct PieSlice {
    pub name: String,
    pub value: f64,
    pub start: f64,
    pub end: f64,
    pub mid: f64,
}

/// Slice angles from named values. Negative values clamp to zero and a zero
/// total leaves every slice empty instead of dividing by zero.
pub fn pie_slices(items: &[(String, f64)]) -> Vec<PieSlice> {
    let values: Vec<f64> = items.iter().map(|(_, value)| value.max(0.0)).collect();
    let total = values.iter().sum::<f64>();
    let total = if total == 0.0 { 1.0 } else { total };
    let mut angle = TOP;
    items
        .iter()
        .zip(&values)
        .map(|((name, _), value)| {
            let span = value / total * TAU;
            let slice = PieSlice {
                name: name.clone(),
                value: *value,
                start: angle,
                end: angle + span,
                mid: angle + span / 2.0,
            };
            angle += span;
            slice
        })
        .collect()
}

/// Which slice a pointer angle falls in.
pub fn slice_at_angle(slices: &[PieSlice], angle: f64) -> Option<usize> {
    let mut a = angle;
    while a < TOP {
        a += TAU;
    }
    while a >= TOP + TAU {
        a -= TAU;
    }
    slices.iter().position(|s| a >= s.start && a < s.end)
}

#[derive(Clone, Debug, PartialEq)]
pub struct RadarAxis {
    pub label: String,
    pub angle: f64,
}

/// Evenly-spaced spokes, one per label.
pub fn radar_axes(labels: &[String]) -> Vec<RadarAxis> {
    let n = labels.len().max(1) as f64;
    labels
        .iter()
        .enumerate()
        .map(|(i, label)| RadarAxis {
            label: label.clone(),
            angle: TOP + (i as f64 / n) * TAU,
        })
        .collect()
}

/// Nearest radar spoke to a pointer angle.
pub fn axis_at_angle(axes: &[RadarAxis], angle: f64) -> usize {
    let mut best = 0;
    let mut best_distance = f64::INFINITY;
    for (i, axis) in axes.iter().enumerate() {
        let d = (((angle - axis.angle + PI * 3.0) % TAU) - PI).abs();
        if d < best_distance {
            best_distance = d;
            best = i;
        }
    }
    best
}

pub fn polar_x(cx: f64, r: f64, angle: f64) -> f64 {
    cx + angle.cos() * r
}

pub fn polar_y(cy: f64, r: f64, angle: f64) -> f64 {
    cy + angle.sin() * r
}

/// Even-odd point-in-polygon test, polygon as flat `[x0, y0, x1, y1, …]`.
pub fn point_in_polygon(px: f64, py: f64, poly: &[f64]) -> bool {
    let mut inside = false;
    let n = poly.len() / 2;
    if n < 3 {
        return false;
    }
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = (poly[i * 2], poly[i * 2 + 1]);
        let (xj, yj) = (poly[j * 2], poly[j * 2 + 1]);
        if (yi > py) != (yj > py) && px < (xj - xi) * (py - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Distance from a point to the nearest polygon edge. Drives the radial dither
/// density: dense near the edge, thinning toward the centre.
pub fn dist_to_polygon_edge(px: f64, py: f64, poly: &[f64]) -> f64 {
    let mut best = f64::INFINITY;
    let n = poly.len() / 2;
    let mut j = n.saturating_sub(1);
    for i in 0..n {
        let (xi, yi) = (poly[i * 2], poly[i * 2 + 1]);
        let (xj, yj) = (poly[j * 2], poly[j * 2 + 1]);
        let (dx, dy) = (xj - xi, yj - yi);
        let len2 = dx * dx + dy * dy;
        let len2 = if len2 == 0.0 { 1.0 } else { len2 };
        let t = (((px - xi) * dx + (py - yi) * dy) / len2).clamp(0.0, 1.0);
        let ex = xi + t * dx - px;
        let ey = yi + t * dy - py;
        let d = (ex * ex + ey * ey).sqrt();
        if d < best {
            best = d;
        }
        j = i;
    }
    best
}

/// A closed polygon of `radius` at every axis.
pub fn radar_ring(axes: &[RadarAxis], cx: f64, cy: f64, radius: f64) -> Vec<f64> {
    let mut poly = Vec::with_capacity(axes.len() * 2);
    for axis in axes {
        poly.push(polar_x(cx, radius, axis.angle));
        poly.push(polar_y(cy, radius, axis.angle));
    }
    poly
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(values: &[(&str, f64)]) -> Vec<(String, f64)> {
        values
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect()
    }

    #[test]
    fn slices_cover_one_turn_starting_at_the_top() {
        let slices = pie_slices(&named(&[("a", 1.0), ("b", 1.0), ("c", 2.0)]));
        assert_eq!(slices[0].start, TOP);
        assert!((slices[2].end - (TOP + TAU)).abs() < 1e-12);
        assert!((slices[1].mid - slices[1].start).abs() < (slices[1].end - slices[1].start).abs());
    }

    #[test]
    fn negative_and_empty_values_stay_finite() {
        let slices = pie_slices(&named(&[("a", -4.0), ("b", 0.0)]));
        assert_eq!(slices[0].value, 0.0);
        assert!(
            slices
                .iter()
                .all(|s| s.start.is_finite() && s.end.is_finite())
        );
        let empty = pie_slices(&[]);
        assert!(empty.is_empty());
    }

    #[test]
    fn angle_hit_testing_wraps_around_the_top() {
        let slices = pie_slices(&named(&[("a", 1.0), ("b", 1.0)]));
        assert_eq!(slice_at_angle(&slices, TOP + 0.01), Some(0));
        assert_eq!(slice_at_angle(&slices, TOP + TAU - 0.01), Some(1));
        assert_eq!(slice_at_angle(&slices, TOP + TAU + 0.01), Some(0));
    }

    #[test]
    fn radar_axes_are_evenly_spaced_and_hit_the_nearest() {
        let axes = radar_axes(&["n".into(), "e".into(), "s".into(), "w".into()]);
        assert_eq!(axes[0].angle, TOP);
        assert!((axes[2].angle - (TOP + PI)).abs() < 1e-12);
        assert_eq!(axis_at_angle(&axes, TOP + 0.2), 0);
        assert_eq!(axis_at_angle(&axes, TOP + PI + 0.2), 2);
    }

    #[test]
    fn polygon_tests_agree_on_a_square() {
        let square = [0.0, 0.0, 10.0, 0.0, 10.0, 10.0, 0.0, 10.0];
        assert!(point_in_polygon(5.0, 5.0, &square));
        assert!(!point_in_polygon(15.0, 5.0, &square));
        assert!((dist_to_polygon_edge(5.0, 5.0, &square) - 5.0).abs() < 1e-12);
        assert!((dist_to_polygon_edge(-2.0, 5.0, &square) - 2.0).abs() < 1e-12);
    }
}
