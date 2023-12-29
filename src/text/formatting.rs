#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Formatting {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    RED,
    LightPurple,
    Yellow,
    White,
    Obfuscated,
    Blod,
    StrikeThough,
    Underline,
    Italic,
    Reset,
}

impl From<u8> for Formatting {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Black,
            1 => Self::DarkBlue,
            2 => Self::DarkGreen,
            3 => Self::DarkAqua,
            4 => Self::DarkRed,
            5 => Self::DarkPurple,
            6 => Self::Gold,
            7 => Self::Gray,
            8 => Self::DarkGray,
            9 => Self::Blue,
            10 => Self::Green,
            11 => Self::Aqua,
            12 => Self::RED,
            13 => Self::LightPurple,
            14 => Self::Yellow,
            15 => Self::White,
            16 => Self::Obfuscated,
            17 => Self::Blod,
            18 => Self::StrikeThough,
            19 => Self::Underline,
            20 => Self::Italic,
            21 => Self::Reset,
            _ => Self::Black,
        }
    }
}
