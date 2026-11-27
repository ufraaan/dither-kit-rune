//! Bloom recipes. The browser applies these as a CSS filter chain on a blurred
//! copy of the crisp canvas; a Rune host applies the same numbers to its glow
//! layer, and documents any approximation.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BloomConfig {
    /// Blur radius in points.
    pub blur: f64,
    pub brightness: f64,
    /// 0–1.
    pub opacity: f64,
    /// >1 keeps the glow in the series color instead of washing toward white.
    pub saturate: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BloomLevel {
    #[default]
    Off,
    Low,
    High,
    Aura,
}

impl BloomLevel {
    pub const fn name(self) -> &'static str {
        match self {
            BloomLevel::Off => "off",
            BloomLevel::Low => "low",
            BloomLevel::High => "high",
            BloomLevel::Aura => "aura",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "off" => Some(BloomLevel::Off),
            "low" => Some(BloomLevel::Low),
            "high" => Some(BloomLevel::High),
            "aura" => Some(BloomLevel::Aura),
            _ => None,
        }
    }

    pub const fn config(self) -> Option<BloomConfig> {
        match self {
            BloomLevel::Off => None,
            BloomLevel::Low => {
                Some(BloomConfig { blur: 3.0, brightness: 1.35, opacity: 0.7, saturate: 1.4 })
            }
            BloomLevel::High => {
                Some(BloomConfig { blur: 5.0, brightness: 1.5, opacity: 0.78, saturate: 1.5 })
            }
            BloomLevel::Aura => {
                Some(BloomConfig { blur: 15.0, brightness: 2.9, opacity: 0.1, saturate: 3.0 })
            }
        }
    }
}
