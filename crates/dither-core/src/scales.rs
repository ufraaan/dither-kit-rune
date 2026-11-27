//! d3-compatible scales: point, band and linear (with d3's `nice` and `ticks`).

/// `scalePoint` over `0..len` mapped onto `[0, width]`.
#[derive(Clone, Copy, Debug)]
pub struct PointScale {
    len: usize,
    step: f64,
}

impl PointScale {
    pub fn new(len: usize, width: f64) -> Self {
        Self {
            len,
            step: width / (len.saturating_sub(1)).max(1) as f64,
        }
    }

    pub fn position(&self, i: usize) -> f64 {
        self.step * i.min(self.len.saturating_sub(1)) as f64
    }

    pub fn step(&self) -> f64 {
        self.step
    }
}

/// `scaleBand` over `0..len` with upstream's padding, mapped onto `[0, width]`.
#[derive(Clone, Copy, Debug)]
pub struct BandScale {
    len: usize,
    step: f64,
    bandwidth: f64,
    offset: f64,
}

impl BandScale {
    pub const PADDING_INNER: f64 = 0.28;
    pub const PADDING_OUTER: f64 = 0.18;

    pub fn new(len: usize, width: f64) -> Self {
        let n = len as f64;
        let step = width / (n - Self::PADDING_INNER + Self::PADDING_OUTER * 2.0).max(1.0);
        let bandwidth = step * (1.0 - Self::PADDING_INNER);
        let offset = (width - step * (n - Self::PADDING_INNER)) * 0.5;
        Self {
            len,
            step,
            bandwidth,
            offset,
        }
    }

    /// Left edge of band `i`.
    pub fn position(&self, i: usize) -> f64 {
        self.offset + self.step * i.min(self.len.saturating_sub(1)) as f64
    }

    pub fn bandwidth(&self) -> f64 {
        self.bandwidth
    }

    pub fn step(&self) -> f64 {
        self.step
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Index of the category whose band a horizontal pixel offset falls in.
pub fn index_at_band(px: f64, len: usize, plot_width: f64) -> usize {
    if len == 0 || plot_width <= 0.0 {
        return 0;
    }
    let t = (px / plot_width).clamp(0.0, 0.999);
    ((t * len as f64).floor() as usize).min(len - 1)
}

/// Index of the row nearest a horizontal pixel offset within the plot.
pub fn nearest_index(px: f64, len: usize, plot_width: f64) -> usize {
    if len <= 1 || plot_width <= 0.0 {
        return 0;
    }
    let t = (px / plot_width).clamp(0.0, 1.0);
    crate::js_round(t * (len - 1) as f64) as usize
}

const E10: f64 = 7.071_067_811_865_475_5;
const E5: f64 = 3.162_277_660_168_379_5;
const E2: f64 = std::f64::consts::SQRT_2;

/// d3-array `tickIncrement`.
pub fn tick_increment(start: f64, stop: f64, count: usize) -> f64 {
    let step = (stop - start) / count.max(1) as f64;
    if !step.is_finite() || step <= 0.0 {
        return 0.0;
    }
    let power = step.log10().floor();
    let error = step / 10f64.powf(power);
    let factor = if error >= E10 {
        10.0
    } else if error >= E5 {
        5.0
    } else if error >= E2 {
        2.0
    } else {
        1.0
    };
    10f64.powf(power) * factor
}

/// d3-array `ticks` over an ascending domain.
pub fn ticks(start: f64, stop: f64, count: usize) -> Vec<f64> {
    if start == stop {
        return vec![start];
    }
    let step = tick_increment(start, stop, count);
    if step <= 0.0 || !step.is_finite() {
        return Vec::new();
    }
    let first = (start / step).ceil();
    let last = (stop / step).floor();
    if last < first {
        return Vec::new();
    }
    let n = (last - first + 1.0).max(0.0) as usize;
    (0..n).map(|i| (first + i as f64) * step).collect()
}

/// d3-scale linear `nice` over an ascending domain.
pub fn nice_domain(start: f64, stop: f64, count: usize) -> (f64, f64) {
    let (mut lo, mut hi) = (start, stop);
    let mut prestep = 0.0;
    for _ in 0..10 {
        let step = tick_increment(lo, hi, count);
        if step == prestep {
            break;
        }
        if step > 0.0 {
            lo = (lo / step).floor() * step;
            hi = (hi / step).ceil() * step;
        } else {
            break;
        }
        prestep = step;
    }
    (lo, hi)
}

/// The value→pixel scale used by cartesian charts. The domain always includes
/// zero, so positive-only data floors at the plot bottom and diverging data
/// draws below a zero baseline inside the plot.
#[derive(Clone, Copy, Debug)]
pub struct LinearScale {
    d0: f64,
    d1: f64,
    height: f64,
}

impl LinearScale {
    pub fn new(min: f64, max: f64, height: f64) -> Self {
        let lo = min.min(0.0);
        let hi = max.max(0.0);
        let (d0, d1) = nice_domain(lo, if hi == lo { lo + 1.0 } else { hi }, 10);
        Self { d0, d1, height }
    }

    pub fn domain(&self) -> (f64, f64) {
        (self.d0, self.d1)
    }

    /// Pixel offset from the top of the plot, range `[height, 0]`.
    pub fn y(&self, value: f64) -> f64 {
        let span = self.d1 - self.d0;
        if span == 0.0 {
            return self.height;
        }
        (value - self.d0) * (0.0 - self.height) / span + self.height
    }

    pub fn ticks(&self, count: usize) -> Vec<f64> {
        ticks(self.d0, self.d1, count)
    }
}
