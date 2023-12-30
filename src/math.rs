use crate::{Bytes, Read, UnsafeWriter, Write, V21};
use core::mem::transmute;
use glam::{DVec3, IVec3, Vec2};
use minecraft_data::{
    prop_axis__x_y_z, prop_facing__north_east_south_west_up_down,
    prop_facing__north_south_west_east,
};

pub const FRAC_PI_180: f32 = 0.017453292;

#[derive(Writable, Clone, Copy)]
pub struct GlobalPos<'a> {
    pub world: &'a str,
    pub position: BlockPos,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct ChunkSectionPos(pub IVec3);

impl From<BlockPos> for ChunkSectionPos {
    #[inline]
    fn from(value: BlockPos) -> Self {
        Self(value.0 >> 4)
    }
}

impl From<i64> for ChunkSectionPos {
    #[inline]
    fn from(value: i64) -> Self {
        Self(IVec3 {
            x: (value >> 42) as i32,
            y: ((value << 44) >> 44) as i32,
            z: ((value << 22) >> 42) as i32,
        })
    }
}

impl From<ChunkSectionPos> for i64 {
    #[inline]
    fn from(value: ChunkSectionPos) -> Self {
        (((value.0.x & 0x3FFFFF) as i64) << 42)
            | ((value.0.y & 0xFFFFF) as i64)
            | (((value.0.z & 0x3FFFFF) as i64) << 20)
    }
}

impl Write for ChunkSectionPos {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write(&i64::from(*self).to_be_bytes());
    }

    #[inline]
    fn len(&self) -> usize {
        8
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(transparent)]
pub struct BlockPos(pub IVec3);

impl BlockPos {
    #[inline]
    pub const fn to_i64(self) -> i64 {
        let x = (self.0.x & 0x3FF_FFFF) as i64;
        let y = (self.0.y & 0xFFF) as i64;
        let z = (self.0.z & 0x3FF_FFFF) as i64;
        x << 38 | z << 12 | y
    }
}

impl From<i64> for BlockPos {
    #[inline]
    fn from(value: i64) -> Self {
        Self(IVec3 {
            x: (value >> 38) as i32,
            y: ((value << 52) >> 52) as i32,
            z: ((value << 26) >> 38) as i32,
        })
    }
}

impl From<BlockPos> for i64 {
    #[inline]
    fn from(value: BlockPos) -> Self {
        value.to_i64()
    }
}

impl Write for BlockPos {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write(&self.to_i64().to_be_bytes());
    }

    #[inline]
    fn len(&self) -> usize {
        8
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Position(pub DVec3);

impl Write for Position {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write(&self.0.x.to_be_bytes());
        w.write(&self.0.y.to_be_bytes());
        w.write(&self.0.z.to_be_bytes());
    }

    #[inline]
    fn len(&self) -> usize {
        24
    }
}

impl Read for Position {
    #[inline]
    fn read(buf: &mut &[u8]) -> Option<Self> {
        Some(Self(DVec3 {
            x: buf.f64()?,
            y: buf.f64()?,
            z: buf.f64()?,
        }))
    }
}
/// pitch, yaw
#[derive(Clone, Copy, Default)]
#[repr(transparent)]
pub struct Rotation(pub Vec2);

impl Rotation {
    #[inline]
    pub const fn yaw(self) -> f32 {
        self.0.y
    }
    #[inline]
    pub const fn pitch(self) -> f32 {
        self.0.x
    }
    #[inline]
    pub fn direction(self) -> Direction {
        match (self.0.y / 90.0 + 0.5).floor() as i32 & 3 {
            0 => Direction::South,
            1 => Direction::West,
            2 => Direction::North,
            _ => Direction::East,
        }
    }

    #[inline]
    pub fn segment(self) -> minecraft_data::prop_rotation {
        unsafe { core::mem::transmute((self.0.y * 0.044444446).round() as i8 & 15) }
    }

    #[inline]
    pub fn segment_opposite(self) -> minecraft_data::prop_rotation {
        match self.segment() {
            minecraft_data::prop_rotation::d_0 => minecraft_data::prop_rotation::d_8,
            minecraft_data::prop_rotation::d_1 => minecraft_data::prop_rotation::d_9,
            minecraft_data::prop_rotation::d_2 => minecraft_data::prop_rotation::d_10,
            minecraft_data::prop_rotation::d_3 => minecraft_data::prop_rotation::d_11,
            minecraft_data::prop_rotation::d_4 => minecraft_data::prop_rotation::d_12,
            minecraft_data::prop_rotation::d_5 => minecraft_data::prop_rotation::d_13,
            minecraft_data::prop_rotation::d_6 => minecraft_data::prop_rotation::d_14,
            minecraft_data::prop_rotation::d_7 => minecraft_data::prop_rotation::d_15,
            minecraft_data::prop_rotation::d_8 => minecraft_data::prop_rotation::d_0,
            minecraft_data::prop_rotation::d_9 => minecraft_data::prop_rotation::d_1,
            minecraft_data::prop_rotation::d_10 => minecraft_data::prop_rotation::d_2,
            minecraft_data::prop_rotation::d_11 => minecraft_data::prop_rotation::d_3,
            minecraft_data::prop_rotation::d_12 => minecraft_data::prop_rotation::d_4,
            minecraft_data::prop_rotation::d_13 => minecraft_data::prop_rotation::d_5,
            minecraft_data::prop_rotation::d_14 => minecraft_data::prop_rotation::d_6,
            minecraft_data::prop_rotation::d_15 => minecraft_data::prop_rotation::d_7,
        }
    }

    pub fn nearest(self) -> Direction {
        let pitch = self.pitch() * FRAC_PI_180;
        let yaw = self.yaw() * FRAC_PI_180;
        let p1 = pitch.sin();
        let p2 = pitch.cos();
        let y1 = yaw.sin();
        let y2 = yaw.cos();
        let b1 = y1 > 0.0;
        let b2 = p1 < 0.0;
        let b3 = y2 > 0.0;
        let f1 = if b1 { y1 } else { -y1 };
        let f2 = if b2 { -p1 } else { p1 };
        let f3 = if b3 { y2 } else { -y2 };
        if f1 > f3 {
            if f2 > f1 * p2 {
                if b2 {
                    Direction::Up
                } else {
                    Direction::Down
                }
            } else if b1 {
                Direction::East
            } else {
                Direction::West
            }
        } else if f2 > f3 * p2 {
            if b2 {
                Direction::Up
            } else {
                Direction::Down
            }
        } else if b3 {
            Direction::South
        } else {
            Direction::North
        }
    }

    pub fn nearest_order(self) -> [Direction; 6] {
        let pitch = self.pitch() * FRAC_PI_180;
        let yaw = self.yaw() * FRAC_PI_180;
        let p1 = pitch.sin();
        let p2 = pitch.cos();
        let y1 = yaw.sin();
        let y2 = yaw.cos();
        let b1 = y1 > 0.0;
        let b2 = p1 < 0.0;
        let b3 = y2 > 0.0;
        let f1 = if b1 { y1 } else { -y1 };
        let f2 = if b2 { -p1 } else { p1 };
        let f3 = if b3 { y2 } else { -y2 };
        let f4 = f1 * p2;
        let f5 = f3 * p2;
        let we = if b1 { Direction::East } else { Direction::West };
        let du = if b2 { Direction::Up } else { Direction::Down };
        let ns = if b3 {
            Direction::South
        } else {
            Direction::North
        };
        if f1 > f3 {
            if f2 > f4 {
                dirs6(du, we, ns)
            } else if f5 > f2 {
                dirs6(we, ns, du)
            } else {
                dirs6(we, du, ns)
            }
        } else if f2 > f5 {
            dirs6(du, ns, we)
        } else if f4 > f2 {
            dirs6(ns, we, du)
        } else {
            dirs6(ns, du, we)
        }
    }

    #[inline]
    pub fn points_to(self, dir: Direction) -> bool {
        let f = self.yaw() * FRAC_PI_180;
        match dir {
            Direction::North => -f.cos() > 0.0,
            Direction::South => f.cos() > 0.0,
            Direction::West => f.sin() > 0.0,
            Direction::East => -f.sin() > 0.0,
            Direction::Down => self.pitch() > 0.0,
            Direction::Up => -self.pitch() > 0.0,
        }
    }

    pub fn points_to_x(self) -> Direction {
        if (self.yaw() * FRAC_PI_180).sin() >= 0.0 {
            Direction::West
        } else {
            Direction::East
        }
    }

    pub fn points_to_z(self) -> Direction {
        if (self.yaw() * FRAC_PI_180).cos() <= 0.0 {
            Direction::North
        } else {
            Direction::South
        }
    }

    pub fn points_to_y(self) -> Direction {
        if self.pitch() >= 0.0 {
            Direction::Down
        } else {
            Direction::Up
        }
    }
}

const fn dirs6(a: Direction, b: Direction, c: Direction) -> [Direction; 6] {
    [a, b, c, c.opposite(), b.opposite(), a.opposite()]
}

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    #[inline]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }

    #[inline]
    pub const fn clockwise(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::South => Self::West,
            Self::West => Self::North,
            Self::East => Self::South,
            Self::Down => Self::Down,
            Self::Up => Self::Up,
        }
    }

    const EXCEPT: [[Self; 5]; 6] = [
        [Self::North, Self::South, Self::West, Self::East, Self::Up],
        [Self::North, Self::South, Self::West, Self::East, Self::Down],
        [Self::South, Self::West, Self::East, Self::Down, Self::Up],
        [Self::North, Self::West, Self::East, Self::Down, Self::Up],
        [Self::North, Self::South, Self::East, Self::Down, Self::Up],
        [Self::North, Self::South, Self::West, Self::Down, Self::Up],
    ];
    const EXCEPT_CARDINAL: [[Self; 3]; 4] = [
        [Self::South, Self::West, Self::East],
        [Self::North, Self::West, Self::East],
        [Self::North, Self::South, Self::East],
        [Self::North, Self::South, Self::West],
    ];

    pub const fn except(self) -> [Self; 5] {
        unsafe { *Self::EXCEPT.as_ptr().add(self as usize) }
    }

    pub const fn except_cardinal(self) -> [Self; 3] {
        debug_assert!(self as u8 > 1);
        unsafe {
            *Self::EXCEPT_CARDINAL
                .as_ptr()
                .add((self as u8 - 2) as usize)
        }
    }
}

impl From<u8> for Direction {
    #[inline]
    fn from(value: u8) -> Self {
        if value > 5 {
            unsafe { transmute(0_u8) }
        } else {
            unsafe { transmute(value) }
        }
    }
}

impl From<Direction> for IVec3 {
    #[inline]
    fn from(value: Direction) -> Self {
        match value {
            Direction::Down => Self::NEG_Y,
            Direction::Up => Self::Y,
            Direction::North => Self::NEG_Z,
            Direction::South => Self::Z,
            Direction::West => Self::NEG_X,
            Direction::East => Self::X,
        }
    }
}

impl From<Direction> for prop_axis__x_y_z {
    #[inline]
    fn from(value: Direction) -> Self {
        match value {
            Direction::Down => Self::y,
            Direction::Up => Self::y,
            Direction::North => Self::z,
            Direction::South => Self::z,
            Direction::West => Self::x,
            Direction::East => Self::x,
        }
    }
}

impl From<Direction> for prop_facing__north_east_south_west_up_down {
    #[inline]
    fn from(value: Direction) -> Self {
        match value {
            Direction::Down => Self::down,
            Direction::Up => Self::up,
            Direction::North => Self::north,
            Direction::South => Self::south,
            Direction::West => Self::west,
            Direction::East => Self::east,
        }
    }
}

impl From<prop_facing__north_east_south_west_up_down> for Direction {
    #[inline]
    fn from(value: prop_facing__north_east_south_west_up_down) -> Self {
        match value {
            prop_facing__north_east_south_west_up_down::down => Self::Down,
            prop_facing__north_east_south_west_up_down::up => Self::Up,
            prop_facing__north_east_south_west_up_down::north => Self::North,
            prop_facing__north_east_south_west_up_down::south => Self::South,
            prop_facing__north_east_south_west_up_down::west => Self::West,
            prop_facing__north_east_south_west_up_down::east => Self::East,
        }
    }
}

impl From<prop_facing__north_south_west_east> for Direction {
    #[inline]
    fn from(value: prop_facing__north_south_west_east) -> Self {
        match value {
            prop_facing__north_south_west_east::north => Self::North,
            prop_facing__north_south_west_east::east => Self::East,
            prop_facing__north_south_west_east::south => Self::South,
            prop_facing__north_south_west_east::west => Self::West,
        }
    }
}

impl From<Direction> for prop_facing__north_south_west_east {
    #[inline]
    fn from(value: Direction) -> Self {
        match value {
            Direction::North => Self::north,
            Direction::South => Self::south,
            Direction::West => Self::west,
            Direction::East => Self::east,
            Direction::Down => Self::north,
            Direction::Up => Self::north,
        }
    }
}
