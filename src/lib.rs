#![feature(allocator_api, is_sorted)]

#[macro_use]
extern crate mser_macro;

pub mod cfb8;
pub mod chunk;
pub mod command;
pub mod ecs;
pub mod entity;
pub mod inventory;
pub mod math;
pub mod net;
pub mod noise;
pub mod oneshot;
pub mod recipe;
pub mod rng;
pub mod text;

use core::alloc::Allocator;
use core::mem::transmute;
use glam::DVec3;
use math::{BlockPos, Direction, Position};
pub use minecraft_data::*;
pub use mser::*;
use uuid::Uuid;

/// Safety
///
/// `boxed_slice.len() == N`
#[inline]
unsafe fn boxed_slice_as_array_unchecked<T, A: Allocator, const N: usize>(
    boxed_slice: Box<[T], A>,
) -> Box<[T; N], A> {
    debug_assert_eq!(boxed_slice.len(), N);

    let (ptr, alloc) = Box::into_raw_with_allocator(boxed_slice);
    // SAFETY: Pointer and allocator came from an existing box,
    // and our safety condition requires that the length is exactly `N`
    unsafe { Box::from_raw_in(ptr.cast::<[T; N]>(), alloc) }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Identifier<'a>(pub &'a str);

impl Write for Identifier<'_> {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        V21(self.0.len() as u32 + 10).write(w);
        w.write(b"minecraft:");
        w.write(self.0.as_bytes());
    }

    #[inline]
    fn len(&self) -> usize {
        V21(self.0.len() as u32 + 10).len() + 10 + self.0.len()
    }
}

#[derive(Writable, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Hand {
    MainHand,
    OffHand,
}

impl From<u8> for Hand {
    #[inline]
    fn from(value: u8) -> Self {
        if value == 0 {
            Self::MainHand
        } else {
            Self::OffHand
        }
    }
}
#[derive(Writable, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl From<u8> for Difficulty {
    #[inline]
    fn from(value: u8) -> Self {
        if value > 3 {
            unsafe { transmute(0_u8) }
        } else {
            unsafe { transmute(value) }
        }
    }
}

#[derive(Writable, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Arm {
    Left,
    Right,
}

impl From<u8> for Arm {
    #[inline]
    fn from(value: u8) -> Self {
        if value == 0 {
            Self::Left
        } else {
            Self::Right
        }
    }
}

#[derive(Writable, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandBlockType {
    Sequence,
    Auto,
    Redstone,
}

impl From<u8> for CommandBlockType {
    #[inline]
    fn from(n: u8) -> Self {
        if n > 2 {
            Self::Sequence
        } else {
            unsafe { transmute(n) }
        }
    }
}

#[derive(Writable, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum JigsawBlockJoint {
    Rollable,
    Aligned,
}

impl JigsawBlockJoint {
    #[inline]
    pub const fn parse(n: &str) -> Self {
        match n.as_bytes() {
            b"aligned" => Self::Aligned,
            _ => Self::Rollable,
        }
    }
}
#[derive(Writable, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum RecipeBookCategory {
    Crafting,
    Furnace,
    BlastFurnace,
    Smoker,
}

impl RecipeBookCategory {
    #[inline]
    pub const fn new(n: u8) -> Self {
        if n > 3 {
            Self::Crafting
        } else {
            unsafe { transmute(n) }
        }
    }
}

impl From<u8> for RecipeBookCategory {
    #[inline]
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

#[derive(Writable, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl GameMode {
    #[inline]
    pub const fn try_from_u8(n: u8) -> Option<Self> {
        if n == 0xff {
            None
        } else if n > 3 {
            unsafe { Some(transmute(0_u8)) }
        } else {
            unsafe { Some(transmute(n)) }
        }
    }

    pub const fn parse(n: &[u8]) -> Self {
        match n {
            b"survival" => Self::Survival,
            b"creative" => Self::Creative,
            b"adventure" => Self::Adventure,
            _ => Self::Spectator,
        }
    }
}

impl From<u8> for GameMode {
    #[inline]
    fn from(value: u8) -> Self {
        if value > 3 {
            unsafe { transmute(0_u8) }
        } else {
            unsafe { transmute(value) }
        }
    }
}

#[derive(Clone)]
pub struct GameProfile {
    pub name: Box<str>,
    pub uuid: Uuid,
    pub textures: Box<str>,
    pub signature: Box<str>,
}

impl Write for GameProfile {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        self.uuid.write(w);
        V21(self.name.len() as u32).write(w);
        w.write(self.name.as_bytes());
        if !self.textures.is_empty() && !self.signature.is_empty() {
            w.write_byte(1);
            w.write_byte(8);
            w.write(b"textures");
            V21(self.textures.len() as u32).write(w);
            w.write(self.textures.as_bytes());
            true.write(w);
            V21(self.signature.len() as u32).write(w);
            w.write(self.signature.as_bytes());
        } else {
            w.write_byte(0);
        }
    }

    #[inline]
    fn len(&self) -> usize {
        let mut w = 0;
        w += self.uuid.len();
        w += V21(self.name.len() as u32).len();
        w += self.name.len();
        if !self.textures.is_empty() && !self.signature.is_empty() {
            w += 10;
            w += V21(self.textures.len() as u32).len();
            w += self.textures.len();
            w += 1;
            w += V21(self.signature.len() as u32).len();
            w += self.signature.len();
        } else {
            w += 1;
        }
        w
    }
}

#[derive(Clone, Copy)]
pub struct Abilities {
    pub invulnerable: bool,
    pub flying: bool,
    pub allow_fly: bool,
    pub creative_mode: bool,
    pub allow_modify_world: bool,
    pub fly_speed: f32,
    pub walk_speed: f32,
}

impl From<GameMode> for Abilities {
    fn from(value: GameMode) -> Self {
        match value {
            GameMode::Survival => Self {
                invulnerable: false,
                flying: false,
                allow_fly: false,
                creative_mode: false,
                allow_modify_world: true,
                fly_speed: 0.1,
                walk_speed: 0.1,
            },
            GameMode::Creative => Self {
                invulnerable: true,
                flying: false,
                allow_fly: true,
                creative_mode: true,
                allow_modify_world: true,
                fly_speed: 0.1,
                walk_speed: 0.1,
            },
            GameMode::Adventure => Self {
                invulnerable: false,
                flying: false,
                allow_fly: false,
                creative_mode: false,
                allow_modify_world: false,
                fly_speed: 0.1,
                walk_speed: 0.1,
            },
            GameMode::Spectator => Self {
                invulnerable: true,
                flying: true,
                allow_fly: true,
                creative_mode: false,
                allow_modify_world: false,
                fly_speed: 0.1,
                walk_speed: 0.1,
            },
        }
    }
}

#[derive(Copy, Clone)]
pub struct BlockHitResult {
    pub pos: Position,
    pub missed: bool,
    pub side: Direction,
    pub block_pos: BlockPos,
    pub inside_block: bool,
}

impl Read for BlockHitResult {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        let block_pos = BlockPos::from(buf.i64()?);
        let side = Direction::from(buf.u8()?);
        let x = buf.f32()? as f64;
        let y = buf.f32()? as f64;
        let z = buf.f32()? as f64;
        let pos = DVec3 { x, y, z } + block_pos.0.as_dvec3();
        let inside_block = buf.u8()? == 1;
        Some(Self {
            pos: Position(pos),
            missed: false,
            side,
            block_pos,
            inside_block,
        })
    }
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum StructureBlockAction {
    UpdateData,
    SaveArea,
    LoadArea,
    ScanArea,
}

impl StructureBlockAction {
    #[inline]
    pub const fn new(n: u8) -> Self {
        if n > 3 {
            Self::UpdateData
        } else {
            unsafe { transmute(n) }
        }
    }
}

impl From<u8> for StructureBlockAction {
    #[inline]
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum StructureBlockMode {
    Save,
    Load,
    Corner,
    Data,
}

impl StructureBlockMode {
    #[inline]
    pub const fn new(n: u8) -> Self {
        if n > 3 {
            Self::Save
        } else {
            unsafe { transmute(n) }
        }
    }
}

impl From<u8> for StructureBlockMode {
    #[inline]
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum BlockMirror {
    None,
    LeftRight,
    FrontBack,
}

impl BlockMirror {
    #[inline]
    pub const fn new(n: u8) -> Self {
        if n > 2 {
            Self::None
        } else {
            unsafe { transmute(n) }
        }
    }
}

impl From<u8> for BlockMirror {
    #[inline]
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum BlockRotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

impl BlockRotation {
    #[inline]
    pub const fn new(n: u8) -> Self {
        if n > 3 {
            Self::Clockwise90
        } else {
            unsafe { transmute(n) }
        }
    }
}

impl From<u8> for BlockRotation {
    #[inline]
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}
