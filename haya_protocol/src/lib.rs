#![no_std]
#![warn(clippy::shadow_reuse, clippy::use_self)]

use crate::inventory::HumanoidArm;
use alloc::vec::Vec;
use core::str::FromStr;
use haya_collection::{List, Map, capacity_fix};
use haya_ident::{Ident, ResourceKey};
use haya_math::{BlockPosPacked, Direction, FVec3, IVec3};
use haya_nbt::Tag;
use minecraft_data::data_component_type;
use mser::{Either, Error, Read, Reader, Utf8, V21, V32, Write, Writer};

pub mod advancement;
pub mod attribute;
pub mod block;
pub mod chat;
pub mod clientbound;
pub mod command;
pub mod crafting;
pub mod debug;
pub mod effect;
pub mod entity;
pub mod entity_data;
pub mod food;
pub mod game_event;
pub mod inventory;
pub mod item_stack;
pub mod level_event;
pub mod map;
pub mod minecart;
pub mod particle;
pub mod path;
pub mod profile;
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
pub mod waypoint;

#[macro_use]
extern crate mser_macro;
extern crate alloc;

#[derive(Clone, Copy, Debug)]
pub struct Translatable<'a>(pub &'a str, pub &'a str);

#[derive(Clone, Serialize, Deserialize)]
pub struct ComponentJson<'a>(pub Utf8<'a, 262144>);

#[derive(Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ComponentRaw(pub Tag);

#[derive(Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct StyleRaw(pub Tag);

#[derive(Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct DialogRaw(pub Tag);

#[derive(Clone, Serialize, Deserialize)]
pub struct KnownPack<'a> {
    pub namespace: Utf8<'a>,
    pub id: Utf8<'a>,
    pub version: Utf8<'a>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerLinkUntrustedEntry<'a> {
    pub ty: Either<KnownLinkType, ComponentRaw>,
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
    pub const fn name(self) -> &'static str {
        match self {
            Self::ReportBug => "report_bug",
            Self::CommunityGuidelines => "community_guidelines",
            Self::Support => "support",
            Self::Status => "status",
            Self::Feedback => "feedback",
            Self::Community => "community",
            Self::Website => "website",
            Self::Forums => "forums",
            Self::News => "news",
            Self::Announcements => "announcements",
        }
    }

    pub const fn translation_key(self) -> Translatable<'static> {
        Translatable("known_server_link.", self.name())
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
    pub const fn name(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::System => "system",
            Self::Hidden => "hidden",
        }
    }

    pub const fn translation_key(self) -> Translatable<'static> {
        Translatable("options.chat.visibility.", self.name())
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
    pub const fn name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Decreased => "decreased",
            Self::Minimal => "minimal",
        }
    }

    pub const fn translation_key(self) -> Translatable<'static> {
        Translatable("options.particles.", self.name())
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
        Translatable("options.difficulty.", self.name())
    }
}

impl FromStr for Difficulty {
    type Err = Error;
    fn from_str(n: &str) -> Result<Self, Self::Err> {
        Ok(match n {
            "peaceful" => Self::Peaceful,
            "easy" => Self::Easy,
            "normal" => Self::Normal,
            "hard" => Self::Hard,
            _ => return Err(Error),
        })
    }
}

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
            let len2 = (len - 1) as usize;
            let mut vec = Vec::with_capacity(capacity_fix(len2));
            for _ in 0..len2 {
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

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct MilliSeconds(pub u64);

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum ClickType {
    Pickup,
    QuickMove,
    Swap,
    Clone,
    Throw,
    QuickCraft,
    PickupAll,
}

// CRC32C
#[derive(Clone, Serialize, Deserialize)]
pub struct HashedPatchMap<'a> {
    pub added_components: Map<'a, data_component_type, u32, 256>,
    pub removed_components: List<'a, data_component_type, 256>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HashedStack<'a> {
    pub item: minecraft_data::item,
    #[mser(varint)]
    pub count: u32,
    pub components: HashedPatchMap<'a>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Input(pub u8);

impl Input {
    pub const FORWARD: u8 = 1;
    pub const BACKWARD: u8 = 2;
    pub const LEFT: u8 = 4;
    pub const RIGHT: u8 = 8;
    pub const JUMP: u8 = 16;
    pub const SHIFT: u8 = 32;
    pub const SPRINT: u8 = 64;
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum CommandBlockEntityMode {
    Sequence,
    Auto,
    Redstone,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[repr(u8)]
#[mser(varint)]
pub enum JointType {
    Rollable,
    Aligned,
}

#[derive(Clone, Copy)]
pub struct JointTypeName(pub JointType);

impl<'a> Read<'a> for JointTypeName {
    fn read(buf: &mut Reader<'a>) -> Result<Self, Error> {
        let a: Utf8 = Utf8::read(buf)?;
        Ok(match a.0 {
            "rollable" => Self(JointType::Rollable),
            _ => Self(JointType::Aligned),
        })
    }
}

impl Write for JointTypeName {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe {
            let a: Utf8 = Utf8(self.0.name());
            a.write(w);
        }
    }

    fn len_s(&self) -> usize {
        let a: Utf8 = Utf8(self.0.name());
        a.len_s()
    }
}

impl JointType {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Rollable => "rollable",
            Self::Aligned => "aligned",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum StructureUpdateType {
    UpdateData,
    SaveArea,
    LoadArea,
    ScanArea,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum StructureMode {
    Save,
    Load,
    Corner,
    Data,
}

impl StructureMode {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Save => "save",
            Self::Load => "load",
            Self::Corner => "corner",
            Self::Data => "data",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum Mirror {
    None,
    LeftRight,
    FrontBack,
}

impl Mirror {
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::LeftRight => "left_right",
            Self::FrontBack => "front_back",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

impl Rotation {
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Clockwise90 => "clockwise_90",
            Self::Clockwise180 => "180",
            Self::Counterclockwise90 => "counterclockwise_90",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum TestBlockMode {
    Start,
    Log,
    Fail,
    Accept,
}

impl TestBlockMode {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Log => "log",
            Self::Fail => "fail",
            Self::Accept => "accept",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TestInstanceData<'a> {
    pub test: Option<ResourceKey<'a>>,
    pub size: IVec3,
    pub rotation: Rotation,
    pub ignore_entities: bool,
    pub status: TestInstanceStatus,
    pub error_message: Option<ComponentRaw>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
#[mser(varint)]
pub enum TestInstanceStatus {
    Cleared,
    Running,
    Finished,
}

impl TestInstanceStatus {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Cleared => "cleared",
            Self::Running => "running",
            Self::Finished => "finished",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockHitResult {
    pub block_pos: BlockPosPacked,
    pub face: Direction,
    pub click: FVec3,
    pub inside: bool,
    pub world_border_hit: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clientbound::login::LoginFinished;
    use crate::profile::{GameProfileRef, PropertyMapRef};
    use crate::types::Id as _;
    use haya_collection::List;
    use minecraft_data::clientbound_login;
    use uuid::Uuid;

    #[test]
    fn test_write() {
        let packet: LoginFinished = LoginFinished {
            game_profile: GameProfileRef {
                id: Uuid::nil(),
                name: Utf8("abc"),
                properties: PropertyMapRef(List::Borrowed(&[])),
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
        let mut reader = Reader::new(&data);
        let id = V32::read(&mut reader).unwrap().0;
        assert_eq!(clientbound_login::new(id as _).unwrap(), LoginFinished::ID);
        assert_eq!(Uuid::read(&mut reader).unwrap(), Uuid::nil());
        assert_eq!(Utf8::<16>::read(&mut reader).unwrap().0, "abc");
        assert_eq!(V32::read(&mut reader).unwrap().0, 0);
        assert!(reader.is_empty());
    }
}
