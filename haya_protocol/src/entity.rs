use mser::{Read, V21, Write};

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum FoxVariant {
    Red,
    Snow,
}

impl FoxVariant {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Snow => "snow",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum SalmonVariant {
    Small,
    Medium,
    Large,
}

impl SalmonVariant {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum ParrotVariant {
    RedBlue,
    Blue,
    Green,
    YellowBlue,
    Gray,
}

impl ParrotVariant {
    pub const fn name(self) -> &'static str {
        match self {
            Self::RedBlue => "red_blue",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::YellowBlue => "yellow_blue",
            Self::Gray => "gray",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum TropicalFishPattern {
    Kob,
    Sunstreak,
    Snooper,
    Dasher,
    Brinely,
    Spotty,
    Flopper,
    Stripey,
    Glitter,
    Blockfish,
    Betty,
    Clayfish,
}

impl TropicalFishPattern {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Kob => "kob",
            Self::Sunstreak => "sunstreak",
            Self::Snooper => "snooper",
            Self::Dasher => "dasher",
            Self::Brinely => "brinely",
            Self::Spotty => "spotty",
            Self::Flopper => "flopper",
            Self::Stripey => "stripey",
            Self::Glitter => "glitter",
            Self::Blockfish => "blockfish",
            Self::Betty => "betty",
            Self::Clayfish => "clayfish",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum MushroomCowVariant {
    Red,
    Brown,
}

impl MushroomCowVariant {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Brown => "brown",
        }
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum RabbitVariant {
    Brown,
    White,
    Black,
    WhiteSplotched,
    Gold,
    Salt,
    Evil = 99,
}

impl Write for RabbitVariant {
    unsafe fn write(&self, w: &mut mser::Writer) {
        unsafe { w.write_byte(*self as u8) }
    }

    fn len_s(&self) -> usize {
        1
    }
}

impl<'a> Read<'a> for RabbitVariant {
    fn read(buf: &mut mser::Reader<'a>) -> Result<Self, mser::Error> {
        Ok(match V21::read(buf)?.0 {
            1 => Self::White,
            2 => Self::Black,
            3 => Self::WhiteSplotched,
            4 => Self::Gold,
            5 => Self::Salt,
            99 => Self::Evil,
            _ => Self::Brown,
        })
    }
}

impl RabbitVariant {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Brown => "brown",
            Self::White => "white",
            Self::Black => "black",
            Self::WhiteSplotched => "white_splotched",
            Self::Gold => "gold",
            Self::Salt => "salt",
            Self::Evil => "evil",
        }
    }
}
