#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum DisplaySlot {
    List,
    Sidebar,
    BelowName,
    TeamBlack,
    TeamDarkBlue,
    TeamDarkGreen,
    TeamDarkAqua,
    TeamDarkRed,
    TeamDarkPurple,
    TeamGold,
    TeamGray,
    TeamDarkGray,
    TeamBlue,
    TeamGreen,
    TeamAqua,
    TeamRed,
    TeamLightPurple,
    TeamYellow,
    TeamWhite,
}

impl DisplaySlot {
    pub const fn name(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Sidebar => "sidebar",
            Self::BelowName => "below_name",
            Self::TeamBlack => "sidebar.team.black",
            Self::TeamDarkBlue => "sidebar.team.dark_blue",
            Self::TeamDarkGreen => "sidebar.team.dark_green",
            Self::TeamDarkAqua => "sidebar.team.dark_aqua",
            Self::TeamDarkRed => "sidebar.team.dark_red",
            Self::TeamDarkPurple => "sidebar.team.dark_purple",
            Self::TeamGold => "sidebar.team.gold",
            Self::TeamGray => "sidebar.team.gray",
            Self::TeamDarkGray => "sidebar.team.dark_gray",
            Self::TeamBlue => "sidebar.team.blue",
            Self::TeamGreen => "sidebar.team.green",
            Self::TeamAqua => "sidebar.team.aqua",
            Self::TeamRed => "sidebar.team.red",
            Self::TeamLightPurple => "sidebar.team.light_purple",
            Self::TeamYellow => "sidebar.team.yellow",
            Self::TeamWhite => "sidebar.team.white",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum ObjectiveCriteriaRenderType {
    Integer,
    Hearts,
}

impl ObjectiveCriteriaRenderType {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Integer => "integer",
            Self::Hearts => "hearts",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum TeamVisibility {
    Always,
    Never,
    HideForOtherTeams,
    HideForOwnTeam,
}

impl TeamVisibility {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
            Self::HideForOtherTeams => "hideForOtherTeams",
            Self::HideForOwnTeam => "hideForOwnTeam",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum TeamCollisionRule {
    Always,
    Never,
    PushOtherTeams,
    PushOwnTeam,
}

impl TeamCollisionRule {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
            Self::PushOtherTeams => "pushOtherTeams",
            Self::PushOwnTeam => "pushOwnTeam",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[mser(varint)]
#[repr(u8)]
pub enum TeamColor {
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
    Red,
    LightPurple,
    Yellow,
    White,
}

impl TeamColor {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Black => "black",
            Self::DarkBlue => "dark_blue",
            Self::DarkGreen => "dark_green",
            Self::DarkAqua => "dark_aqua",
            Self::DarkRed => "dark_red",
            Self::DarkPurple => "dark_purple",
            Self::Gold => "gold",
            Self::Gray => "gray",
            Self::DarkGray => "dark_gray",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Aqua => "aqua",
            Self::Red => "red",
            Self::LightPurple => "light_purple",
            Self::Yellow => "yellow",
            Self::White => "white",
        }
    }
}
