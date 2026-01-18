use std::fmt;

/// AMD processor codenames supported by ryzen_smu
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Codename {
    Unsupported = 0,
    Colfax = 1,
    Renoir = 2,
    Picasso = 3,
    Matisse = 4,
    Threadripper = 5,
    CastlePeak = 6,
    Raven = 7,
    Raven2 = 8,
    SummitRidge = 9,
    PinnacleRidge = 10,
    Rembrandt = 11,
    Vermeer = 12,
    Vangogh = 13,
    Cezanne = 14,
    Milan = 15,
    Dali = 16,
    Lucienne = 17,
    Naples = 18,
    Chagall = 19,
    Raphael = 20,
    Phoenix = 21,
    HawkPoint = 22,
    GraniteRidge = 23,
    StrixPoint = 24,
    StormPeak = 25,
}

impl Codename {
    /// Parse codename from the numeric value in sysfs
    pub fn from_id(id: u32) -> Self {
        match id {
            1 => Self::Colfax,
            2 => Self::Renoir,
            3 => Self::Picasso,
            4 => Self::Matisse,
            5 => Self::Threadripper,
            6 => Self::CastlePeak,
            7 => Self::Raven,
            8 => Self::Raven2,
            9 => Self::SummitRidge,
            10 => Self::PinnacleRidge,
            11 => Self::Rembrandt,
            12 => Self::Vermeer,
            13 => Self::Vangogh,
            14 => Self::Cezanne,
            15 => Self::Milan,
            16 => Self::Dali,
            17 => Self::Lucienne,
            18 => Self::Naples,
            19 => Self::Chagall,
            20 => Self::Raphael,
            21 => Self::Phoenix,
            22 => Self::HawkPoint,
            23 => Self::GraniteRidge,
            24 => Self::StrixPoint,
            25 => Self::StormPeak,
            _ => Self::Unsupported,
        }
    }

    /// Get the number of cores per CCD for this processor family
    pub fn cores_per_ccd(&self) -> usize {
        match self {
            Self::Matisse | Self::Vermeer | Self::Milan | Self::Raphael | Self::GraniteRidge => 8,
            Self::Cezanne | Self::Rembrandt | Self::Phoenix | Self::HawkPoint | Self::StrixPoint => 8,
            Self::Renoir | Self::Lucienne => 8,
            _ => 8, // Default assumption
        }
    }

    /// Get max CCDs for this processor family
    pub fn max_ccds(&self) -> usize {
        match self {
            Self::Milan | Self::Naples | Self::Chagall | Self::StormPeak => 8,
            Self::Threadripper | Self::CastlePeak => 4,
            Self::Vermeer | Self::Matisse | Self::Raphael | Self::GraniteRidge => 2,
            _ => 1,
        }
    }
}

impl fmt::Display for Codename {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Unsupported => "Unsupported",
            Self::Colfax => "Colfax",
            Self::Renoir => "Renoir",
            Self::Picasso => "Picasso",
            Self::Matisse => "Matisse",
            Self::Threadripper => "Threadripper",
            Self::CastlePeak => "Castle Peak",
            Self::Raven => "Raven",
            Self::Raven2 => "Raven 2",
            Self::SummitRidge => "Summit Ridge",
            Self::PinnacleRidge => "Pinnacle Ridge",
            Self::Rembrandt => "Rembrandt",
            Self::Vermeer => "Vermeer",
            Self::Vangogh => "Van Gogh",
            Self::Cezanne => "Cezanne",
            Self::Milan => "Milan",
            Self::Dali => "Dali",
            Self::Lucienne => "Lucienne",
            Self::Naples => "Naples",
            Self::Chagall => "Chagall",
            Self::Raphael => "Raphael",
            Self::Phoenix => "Phoenix",
            Self::HawkPoint => "Hawk Point",
            Self::GraniteRidge => "Granite Ridge",
            Self::StrixPoint => "Strix Point",
            Self::StormPeak => "Storm Peak",
        };
        write!(f, "{}", name)
    }
}
