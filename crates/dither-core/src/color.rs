//! Named palette, hue escape hatch and the alpha-tier color model.

/// An 8-bit RGB color. Every dither pixel is one of these plus an alpha.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// The three upstream shades of a palette entry. The painter uses only `fill`;
/// `line` and `star` exist for the line/sparkle surfaces and for parity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seed {
    pub fill: Rgb,
    pub line: Rgb,
    pub star: Rgb,
}

macro_rules! color {
    ($name:ident, $fill:expr, $line:expr, $star:expr) => {
        Seed {
            fill: Rgb::new($fill[0], $fill[1], $fill[2]),
            line: Rgb::new($line[0], $line[1], $line[2]),
            star: Rgb::new($star[0], $star[1], $star[2]),
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DitherColor {
    Green,
    Blue,
    Purple,
    Pink,
    Orange,
    Red,
    Grey,
}

impl DitherColor {
    pub const ALL: [DitherColor; 7] = [
        DitherColor::Green,
        DitherColor::Blue,
        DitherColor::Purple,
        DitherColor::Pink,
        DitherColor::Orange,
        DitherColor::Red,
        DitherColor::Grey,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            DitherColor::Green => "green",
            DitherColor::Blue => "blue",
            DitherColor::Purple => "purple",
            DitherColor::Pink => "pink",
            DitherColor::Orange => "orange",
            DitherColor::Red => "red",
            DitherColor::Grey => "grey",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }

    pub const fn seed(self) -> Seed {
        match self {
            DitherColor::Green => color!(Green, [40, 210, 110], [150, 255, 180], [200, 255, 220]),
            DitherColor::Blue => color!(Blue, [53, 143, 243], [150, 200, 255], [205, 228, 255]),
            DitherColor::Purple => {
                color!(Purple, [150, 110, 255], [200, 175, 255], [225, 210, 255])
            }
            DitherColor::Pink => color!(Pink, [240, 90, 190], [255, 170, 220], [255, 205, 235]),
            DitherColor::Orange => color!(Orange, [255, 150, 50], [255, 195, 130], [255, 220, 175]),
            DitherColor::Red => color!(Red, [240, 70, 70], [255, 150, 140], [255, 195, 185]),
            DitherColor::Grey => color!(Grey, [92, 92, 100], [140, 140, 150], [165, 165, 175]),
        }
    }

    pub const fn fill(self) -> Rgb {
        self.seed().fill
    }
}

/// A named palette color or a raw hue in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PixelColor {
    Named(DitherColor),
    Hue(f64),
}

impl From<DitherColor> for PixelColor {
    fn from(value: DitherColor) -> Self {
        PixelColor::Named(value)
    }
}

/// Hue (0–360) → the fill color used by avatars and gradients: `hsl(h, 85%, 58%)`.
pub fn hue_fill(hue: f64) -> Rgb {
    let h = ((hue % 360.0) + 360.0) % 360.0;
    let s: f64 = 0.85;
    let l: f64 = 0.58;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let channel = |v: f64| crate::js_round((v + m) * 255.0).clamp(0.0, 255.0) as u8;
    Rgb::new(channel(r), channel(g), channel(b))
}

pub fn fill_of(color: PixelColor) -> Rgb {
    match color {
        PixelColor::Named(named) => named.fill(),
        PixelColor::Hue(hue) => hue_fill(hue),
    }
}
