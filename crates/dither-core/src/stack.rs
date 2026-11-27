//! Series stacking, matching d3-shape's `stack` with the offsets upstream uses.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StackMode {
    /// Every series sits on the zero baseline.
    #[default]
    None,
    Stacked,
    Percent,
}

impl StackMode {
    pub const fn name(self) -> &'static str {
        match self {
            StackMode::None => "default",
            StackMode::Stacked => "stacked",
            StackMode::Percent => "percent",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "default" => Some(StackMode::None),
            "stacked" => Some(StackMode::Stacked),
            "percent" => Some(StackMode::Percent),
            _ => None,
        }
    }

    pub const fn is_stacked(self) -> bool {
        !matches!(self, StackMode::None)
    }
}

/// Per-series `[y0, y1]` bands for every row, plus the value range.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bands {
    /// Series-major: `bands[series][row] = [y0, y1]`.
    pub bands: Vec<Vec<[f64; 2]>>,
    pub max: f64,
    pub min: f64,
}

fn num(v: f64) -> f64 {
    if v.is_finite() { v } else { 0.0 }
}

/// `series` is series-major: `series[s][row]` is one series' value at one row.
/// Rows shorter than the longest series are treated as zero.
pub fn compute_bands(series: &[Vec<f64>], mode: StackMode) -> Bands {
    if series.is_empty() {
        return Bands { bands: Vec::new(), max: 1.0, min: 0.0 };
    }
    let rows = series.iter().map(Vec::len).max().unwrap_or(0);
    let mut bands: Vec<Vec<[f64; 2]>> = series
        .iter()
        .map(|s| {
            (0..rows)
                .map(|i| [0.0, num(s.get(i).copied().unwrap_or(0.0))])
                .collect()
        })
        .collect();

    if mode.is_stacked() {
        if mode == StackMode::Percent {
            for row in 0..rows {
                let total: f64 = bands.iter().map(|s| s[row][1]).sum();
                if total != 0.0 {
                    for s in &mut bands {
                        s[row][1] /= total;
                    }
                }
            }
        }
        // d3 offsetNone: y0 = previous y1, y1 = value + y0.
        for i in 1..bands.len() {
            let (head, tail) = bands.split_at_mut(i);
            let below = &head[i - 1];
            for (row, band) in tail[0].iter_mut().enumerate() {
                let y0 = if below[row][1].is_nan() { below[row][0] } else { below[row][1] };
                band[0] = y0;
                band[1] += y0;
            }
        }
    }

    let mut max: f64 = 0.0;
    let mut min: f64 = 0.0;
    for s in &bands {
        for band in s {
            if band[1] > max {
                max = band[1];
            }
            let low = if mode.is_stacked() { band[0] } else { band[1] };
            if low < min {
                min = low;
            }
        }
    }
    let flat = max == 0.0 && min == 0.0;
    Bands { bands, max: if flat { 1.0 } else { max }, min }
}
