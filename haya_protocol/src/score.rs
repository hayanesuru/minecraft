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
