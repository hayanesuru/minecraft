use crate::Component;
use haya_ident::Ident;
use minecraft_data::{villager_profession, villager_type};
use mser::{Read, V21, Write};
use uuid::Uuid;

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

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum EquineVariant {
    White,
    Creamy,
    Chestnut,
    Brown,
    Black,
    Gray,
    DarkBrown,
}

impl EquineVariant {
    pub const fn name(self) -> &'static str {
        match self {
            Self::White => "white",
            Self::Creamy => "creamy",
            Self::Chestnut => "chestnut",
            Self::Brown => "brown",
            Self::Black => "black",
            Self::Gray => "gray",
            Self::DarkBrown => "dark_brown",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PaintingVariant<'a> {
    #[mser(varint)]
    pub width: u32,
    #[mser(varint)]
    pub height: u32,
    pub asset_id: Ident<'a>,
    pub title: Option<Component>,
    pub author: Option<Component>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum LlamaVariant {
    Creamy,
    White,
    Brown,
    Gray,
}

impl LlamaVariant {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Creamy => "creamy",
            Self::White => "white",
            Self::Brown => "brown",
            Self::Gray => "gray",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum AxolotlVariant {
    Lucy,
    Wild,
    Gold,
    Cyan,
    Blue,
}

impl AxolotlVariant {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Lucy => "lucy",
            Self::Wild => "wild",
            Self::Gold => "gold",
            Self::Cyan => "cyan",
            Self::Blue => "blue",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EntityReference {
    pub entity: Uuid,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VillagerData {
    pub ty: villager_type,
    pub profession: villager_profession,
    #[mser(varint)]
    pub level: u32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum ArmadilloState {
    Idle,
    Rolling,
    Scared,
    Unrolling,
}

impl ArmadilloState {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Rolling => "rolling",
            Self::Scared => "scared",
            Self::Unrolling => "unrolling",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum SnifferState {
    Idling,
    FeelingHappy,
    Scenting,
    Sniffing,
    Searching,
    Digging,
    Rising,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum CopperGolemState {
    Idle,
    GettingItem,
    GettingNoItem,
    DroppingItem,
    DroppingNoItem,
}

impl CopperGolemState {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::GettingItem => "getting_item",
            Self::GettingNoItem => "getting_no_item",
            Self::DroppingItem => "dropping_item",
            Self::DroppingNoItem => "dropping_no_item",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[mser(varint)]
#[repr(u8)]
pub enum Pose {
    Standing,
    FallFlying,
    Sleeping,
    Swimming,
    SpinAttack,
    Crouching,
    LongJumping,
    Dying,
    Croaking,
    UsingTongue,
    Sitting,
    Roaring,
    Sniffing,
    Emerging,
    Digging,
    Sliding,
    Shooting,
    Inhaling,
}

impl Pose {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Standing => "standing",
            Self::FallFlying => "fall_flying",
            Self::Sleeping => "sleeping",
            Self::Swimming => "swimming",
            Self::SpinAttack => "spin_attack",
            Self::Crouching => "crouching",
            Self::LongJumping => "long_jumping",
            Self::Dying => "dying",
            Self::Croaking => "croaking",
            Self::UsingTongue => "using_tongue",
            Self::Sitting => "sitting",
            Self::Roaring => "roaring",
            Self::Sniffing => "sniffing",
            Self::Emerging => "emerging",
            Self::Digging => "digging",
            Self::Sliding => "sliding",
            Self::Shooting => "shooting",
            Self::Inhaling => "inhaling",
        }
    }
}
