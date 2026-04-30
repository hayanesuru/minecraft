#![no_std]

use alloc::vec::Vec;
use haya_collection::{List, capacity_fix};
use haya_ident::{Ident, ResourceKey};
use haya_math::BlockPosPacked;
use haya_nbt::Tag;
use mser::{Either, Error, Read, Reader, Utf8, V21, V32, Write, Writer};

pub mod advancement;
pub mod attribute;
pub mod block;
pub mod chat;
pub mod clientbound;
pub mod command;
pub mod debug;
pub mod effect;
pub mod entity;
pub mod entity_data;
pub mod food;
pub mod game_event;
pub mod item;
pub mod level_event;
pub mod map;
pub mod minecart;
pub mod particle;
pub mod path;
pub mod profile;
pub mod recipe;
pub mod redstone;
pub mod registry;
pub mod score;
pub mod serverbound;
pub mod sound;
pub mod stat;
pub mod structure;
pub mod trading;
pub mod trim;
pub mod types;

#[macro_use]
extern crate mser_macro;
extern crate alloc;

#[derive(Clone, Copy, Debug)]
pub struct Translatable<'a>(pub &'a str);

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum ClientIntent {
    Status = 1,
    Login = 2,
    Transfer = 3,
}

impl Write for ClientIntent {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            w.write_byte(*self as u8);
        }
    }

    fn len_s(&self) -> usize {
        1
    }
}

impl<'a> Read<'a> for ClientIntent {
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        match V21::read(buf)?.0 {
            1 => Ok(Self::Status),
            2 => Ok(Self::Login),
            3 => Ok(Self::Transfer),
            _ => Err(Error),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ComponentJson<'a>(pub Utf8<'a, 262144>);

#[derive(Clone, Serialize, Deserialize)]
pub struct Component(pub Tag);

#[derive(Clone, Serialize, Deserialize)]
pub struct Style(pub Tag);

#[derive(Clone, Serialize, Deserialize)]
pub struct Dialog(pub Tag);

#[derive(Clone, Serialize, Deserialize)]
pub struct KnownPack<'a> {
    pub namespace: Utf8<'a>,
    pub id: Utf8<'a>,
    pub version: Utf8<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerLinkUntrustedEntry<'a> {
    pub ty: Either<KnownLinkType, Component>,
    pub url: Utf8<'a>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum KnownLinkType {
    ReportBug,
    CommunityGuidelines,
    Support,
    Status,
    Feedback,
    Community,
    Website,
    Forums,
    News,
    Announcements,
}

impl KnownLinkType {
    pub const fn translation_key(self) -> Translatable<'static> {
        Translatable(match self {
            Self::ReportBug => "known_server_link.report_bug",
            Self::CommunityGuidelines => "known_server_link.community_guidelines",
            Self::Support => "known_server_link.support",
            Self::Status => "known_server_link.status",
            Self::Feedback => "known_server_link.feedback",
            Self::Community => "known_server_link.community",
            Self::Website => "known_server_link.website",
            Self::Forums => "known_server_link.forums",
            Self::News => "known_server_link.news",
            Self::Announcements => "known_server_link.announcements",
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientInformation<'a> {
    pub language: Utf8<'a, 16>,
    pub view_distance: u8,
    pub chat_visibility: ChatVisibility,
    pub chat_colors: bool,
    pub model_customisation: u8,
    pub main_hand: HumanoidArm,
    pub text_filtering_enabled: bool,
    pub allows_listing: bool,
    pub particle_status: ParticleStatus,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

impl ChatVisibility {
    pub const fn translation_key(self) -> Translatable<'static> {
        Translatable(match self {
            Self::Full => "options.chat.visibility.full",
            Self::System => "options.chat.visibility.system",
            Self::Hidden => "options.chat.visibility.hidden",
        })
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum HumanoidArm {
    Left,
    Right,
}

impl HumanoidArm {
    pub const fn translation_key(self) -> Translatable<'static> {
        Translatable(match self {
            Self::Left => "options.mainHand.left",
            Self::Right => "options.mainHand.right",
        })
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum ParticleStatus {
    All,
    Decreased,
    Minimal,
}

impl ParticleStatus {
    pub const fn translation_key(self) -> Translatable<'static> {
        Translatable(match self {
            Self::All => "options.particles.all",
            Self::Decreased => "options.particles.decreased",
            Self::Minimal => "options.particles.minimal",
        })
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Peaceful => "peaceful",
            Self::Easy => "easy",
            Self::Normal => "normal",
            Self::Hard => "hard",
        }
    }

    pub const fn translation_key(self) -> Translatable<'static> {
        Translatable(match self {
            Self::Peaceful => "options.difficulty.peaceful",
            Self::Easy => "options.difficulty.easy",
            Self::Normal => "options.difficulty.normal",
            Self::Hard => "options.difficulty.hard",
        })
    }

    pub const fn parse(n: &[u8]) -> Option<Self> {
        match n {
            b"peaceful" => Some(Self::Peaceful),
            b"easy" => Some(Self::Easy),
            b"normal" => Some(Self::Normal),
            b"hard" => Some(Self::Hard),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ContainerId(#[mser(varint)] pub u32);

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

impl Rarity {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Common => "common",
            Self::Uncommon => "uncommon",
            Self::Rare => "rare",
            Self::Epic => "epic",
        }
    }
}

#[derive(Clone)]
pub enum HolderSet<'a, T> {
    Named(Ident<'a>),
    Direct(List<'a, T>),
}

impl<'a, T: Read<'a>> Read<'a> for HolderSet<'a, T> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let len = V32::read(buf)?.0;
        if len == 0 {
            let name = Ident::read(buf)?;
            Ok(Self::Named(name))
        } else {
            let len = (len - 1) as usize;
            let mut vec = Vec::with_capacity(capacity_fix(len));
            for _ in 0..len {
                vec.push(T::read(buf)?);
            }
            Ok(Self::Direct(List::Owned(vec)))
        }
    }
}

impl<T: Write> Write for HolderSet<'_, T> {
    unsafe fn write(&self, w: &mut Writer) {
        match self {
            Self::Named(name) => unsafe {
                V32(0).write(w);
                name.write(w);
            },
            Self::Direct(direct) => unsafe {
                V32((direct.len() + 1) as u32).write(w);
                for holder in direct.as_slice() {
                    holder.write(w);
                }
            },
        }
    }

    fn len_s(&self) -> usize {
        match self {
            Self::Named(name) => V32(0).len_s() + name.len_s(),
            Self::Direct(direct) => {
                let mut len = V32((direct.len() + 1) as u32).len_s();
                for x in direct.as_slice() {
                    len += x.len_s();
                }
                len
            }
        }
    }
}

#[derive(Clone)]
pub enum Holder<T, R> {
    Reference(R),
    Direct(T),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum EquipmentSlotGroup {
    Any,
    Mainhand,
    Offhand,
    Hand,
    Feet,
    Legs,
    Chest,
    Head,
    Armor,
    Body,
    Saddle,
}

impl EquipmentSlotGroup {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Mainhand => "mainhand",
            Self::Offhand => "offhand",
            Self::Hand => "hand",
            Self::Feet => "feet",
            Self::Legs => "legs",
            Self::Chest => "chest",
            Self::Head => "head",
            Self::Armor => "armor",
            Self::Body => "body",
            Self::Saddle => "saddle",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum EquipmentSlot {
    Mainhand,
    Feet,
    Legs,
    Chest,
    Head,
    Offhand,
    Body,
    Saddle,
}

impl EquipmentSlot {
    pub const fn new(n: u8) -> Self {
        if n > Self::Saddle as u8 {
            Self::Mainhand
        } else {
            unsafe { core::mem::transmute::<u8, Self>(n) }
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Mainhand => "mainhand",
            Self::Feet => "feet",
            Self::Legs => "legs",
            Self::Chest => "chest",
            Self::Head => "head",
            Self::Offhand => "offhand",
            Self::Body => "body",
            Self::Saddle => "saddle",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum DyeColor {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

impl DyeColor {
    pub const fn name(self) -> &'static str {
        match self {
            Self::White => "white",
            Self::Orange => "orange",
            Self::Magenta => "magenta",
            Self::LightBlue => "light_blue",
            Self::Yellow => "yellow",
            Self::Lime => "lime",
            Self::Pink => "pink",
            Self::Gray => "gray",
            Self::LightGray => "light_gray",
            Self::Cyan => "cyan",
            Self::Purple => "purple",
            Self::Blue => "blue",
            Self::Brown => "brown",
            Self::Green => "green",
            Self::Red => "red",
            Self::Black => "black",
        }
    }
}

#[derive(Clone)]
pub struct Filterable<T> {
    pub raw: T,
    pub filtered: Option<T>,
}

impl<T: Write> Write for Filterable<T> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            self.raw.write(w);
            self.filtered.write(w);
        }
    }

    fn len_s(&self) -> usize {
        self.raw.len_s() + self.filtered.len_s()
    }
}

impl<'a, T: Read<'a>> Read<'a> for Filterable<T> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(Self {
            raw: T::read(buf)?,
            filtered: Option::<T>::read(buf)?,
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LockCode(pub Tag);

#[derive(Clone)]
pub struct WeightedList<'a, T, const MAX: usize = { usize::MAX }>(pub List<'a, Weighted<T>, MAX>);

impl<'a, T: Read<'a>, const MAX: usize> Read<'a> for WeightedList<'a, T, MAX> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(Self(List::read(buf)?))
    }
}

impl<'a, T: Write, const MAX: usize> Write for WeightedList<'a, T, MAX> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe { self.0.write(w) }
    }

    fn len_s(&self) -> usize {
        self.0.len_s()
    }
}

#[derive(Clone)]
pub struct Weighted<T> {
    pub weight: u32,
    pub value: T,
}

impl<T: Write> Write for Weighted<T> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            V32(self.weight).write(w);
            self.value.write(w);
        }
    }
    fn len_s(&self) -> usize {
        V32(self.weight).len_s() + self.value.len_s()
    }
}

impl<'a, T: Read<'a>> Read<'a> for Weighted<T> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(Self {
            weight: V32::read(buf)?.0,
            value: T::read(buf)?,
        })
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum HeightmapType {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

impl HeightmapType {
    pub const fn name(self) -> &'static str {
        match self {
            Self::WorldSurfaceWg => "WORLD_SURFACE_WG",
            Self::WorldSurface => "WORLD_SURFACE",
            Self::OceanFloorWg => "OCEAN_FLOOR_WG",
            Self::OceanFloor => "OCEAN_FLOOR",
            Self::MotionBlocking => "MOTION_BLOCKING",
            Self::MotionBlockingNoLeaves => "MOTION_BLOCKING_NO_LEAVES",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BitSet<'a>(pub List<'a, u64>);

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum GameType {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl GameType {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Survival => "survival",
            Self::Creative => "creative",
            Self::Adventure => "adventure",
            Self::Spectator => "spectator",
        }
    }
}

#[derive(Clone, Copy)]
pub struct OptionalGameType(pub Option<GameType>);

impl<'a> Read<'a> for OptionalGameType {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(Self(match u8::read(buf)? {
            0xff => None,
            1 => Some(GameType::Creative),
            2 => Some(GameType::Adventure),
            3 => Some(GameType::Spectator),
            _ => Some(GameType::Survival),
        }))
    }
}

impl Write for OptionalGameType {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            match self.0 {
                Some(x) => x.write(w),
                None => w.write_byte(0xff),
            }
        }
    }

    fn len_s(&self) -> usize {
        match self.0 {
            Some(x) => x.len_s(),
            None => 1,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GlobalPos<'a> {
    pub dimension: ResourceKey<'a>,
    pub pos: BlockPosPacked,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum InteractionHand {
    MainHand,
    OffHand,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum EntityAnchor {
    Feet,
    Eyes,
}

impl EntityAnchor {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Feet => "feet",
            Self::Eyes => "eyes",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OptionalV32(#[mser(varint)] u32);

impl Default for OptionalV32 {
    fn default() -> Self {
        Self::none()
    }
}

impl OptionalV32 {
    pub const fn some(value: u32) -> Self {
        Self(value + 1)
    }
    pub const fn none() -> Self {
        Self(0)
    }
    pub const fn is_some(self) -> bool {
        self.0 != 0
    }
    pub const fn is_none(self) -> bool {
        self.0 == 0
    }
    pub const fn get(self) -> Option<u32> {
        if self.0 == 0 { None } else { Some(self.0 - 1) }
    }
    pub const fn unwrap(self) -> u32 {
        self.0 - 1
    }
}

#[derive(Clone)]
pub struct V32List<'a>(pub List<'a, u32>);

impl<'a> Read<'a> for V32List<'a> {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let len = V21::read(buf)?.0 as usize;
        let mut vec = Vec::with_capacity(capacity_fix(len).min(buf.len()));
        for _ in 0..len {
            vec.push(V32::read(buf)?.0);
        }
        Ok(Self(List::Owned(vec)))
    }
}

impl<'a> Write for V32List<'a> {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            let x = self.0.as_slice();
            V21(x.len() as u32).write(w);
            for y in x {
                V32(*y).write(w);
            }
        }
    }

    fn len_s(&self) -> usize {
        let x = self.0.as_slice();
        let mut len = V21(x.len() as u32).len_s();
        for y in x {
            len += V32(*y).len_s();
        }
        len
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RespawnData<'a> {
    pub global_pos: GlobalPos<'a>,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Rotations {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum WeatheringCopperState {
    Unaffected,
    Exposed,
    Weathered,
    Oxidized,
}

impl WeatheringCopperState {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unaffected => "unaffected",
            Self::Exposed => "exposed",
            Self::Weathered => "weathered",
            Self::Oxidized => "oxidized",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum ChatFormatting {
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
    Obfuscated,
    Bold,
    Strikethrough,
    Underline,
    Italic,
    Reset,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Relatives(pub u32);

impl Relatives {
    pub const X: u32 = 1;
    pub const Y: u32 = 2;
    pub const Z: u32 = 4;
    pub const Y_ROT: u32 = 8;
    pub const X_ROT: u32 = 16;
    pub const DELTA_X: u32 = 32;
    pub const DELTA_Y: u32 = 64;
    pub const DELTA_Z: u32 = 128;
    pub const ROTATE_DELTA: u32 = 256;

    pub const ALL: u32 = Self::X
        | Self::Y
        | Self::Z
        | Self::Y_ROT
        | Self::X_ROT
        | Self::DELTA_X
        | Self::DELTA_Y
        | Self::DELTA_Z
        | Self::ROTATE_DELTA;
    pub const ROTATION: u32 = Self::Y_ROT | Self::X_ROT;
    pub const DELTA: u32 = Self::DELTA_X | Self::DELTA_Y | Self::DELTA_Z | Self::ROTATE_DELTA;

    pub const fn x(self) -> bool {
        self.0 & Self::X != 0
    }

    pub const fn y(self) -> bool {
        self.0 & Self::Y != 0
    }

    pub const fn z(self) -> bool {
        self.0 & Self::Z != 0
    }

    pub const fn y_rot(self) -> bool {
        self.0 & Self::Y_ROT != 0
    }

    pub const fn x_rot(self) -> bool {
        self.0 & Self::X_ROT != 0
    }

    pub const fn delta_x(self) -> bool {
        self.0 & Self::DELTA_X != 0
    }

    pub const fn delta_y(self) -> bool {
        self.0 & Self::DELTA_Y != 0
    }

    pub const fn delta_z(self) -> bool {
        self.0 & Self::DELTA_Z != 0
    }

    pub const fn rotate_delta(self) -> bool {
        self.0 & Self::ROTATE_DELTA != 0
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResourceTexture<'a>(pub Ident<'a>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clientbound::login::LoginFinished;
    use crate::profile::{GameProfileRef, PropertyMap};
    use crate::types::Id as _;
    use haya_collection::{List, Map};
    use minecraft_data::clientbound__login;
    use uuid::Uuid;

    #[test]
    fn test_write() {
        let packet: LoginFinished = LoginFinished {
            game_profile: GameProfileRef {
                id: Uuid::nil(),
                name: Utf8("abc"),
                properties: PropertyMap(Map(List::Borrowed(&[]))),
            },
        };

        let id = LoginFinished::ID;
        let len1 = id.len_s();
        let len2 = packet.len_s() + len1;
        let data = unsafe {
            let mut data = alloc::vec::Vec::with_capacity(len2);
            mser::write_unchecked(data.as_mut_ptr(), &id);
            mser::write_unchecked(data.as_mut_ptr().add(len1), &packet);
            data.set_len(len2);
            data.into_boxed_slice()
        };
        let mut data = Reader::new(&data);
        let id = V32::read(&mut data).unwrap().0;
        assert_eq!(clientbound__login::new(id as _).unwrap(), LoginFinished::ID);
        assert_eq!(Uuid::read(&mut data).unwrap(), Uuid::nil());
        assert_eq!(Utf8::<16>::read(&mut data).unwrap().0, "abc");
        assert_eq!(V32::read(&mut data).unwrap().0, 0);
        assert!(data.is_empty());
    }
}
