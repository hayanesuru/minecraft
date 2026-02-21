#![no_std]

use mser::{Error, Read, UnsafeWriter, V32, Write};

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Write for Vec3 {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            self.x.write(w);
            self.y.write(w);
            self.z.write(w);
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
        self.x.len_s() + self.y.len_s() + self.z.len_s()
    }
}

impl<'a> Read<'a> for Vec3 {
    #[inline]
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        Ok(Self {
            x: f64::read(buf)?,
            y: f64::read(buf)?,
            z: f64::read(buf)?,
        })
    }
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.,
        y: 0.,
        z: 0.,
    };
}

#[derive(Default, Debug, Clone, Copy)]
pub enum LpVec3 {
    #[default]
    Zero,
    Normal {
        a: u8,
        b: u8,
        c: u32,
    },
    Extended {
        a: u8,
        b: u8,
        c: u32,
        d: u32,
    },
}

impl<'a> Read<'a> for LpVec3 {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let a = u8::read(buf)?;
        if a == 0 {
            Ok(Self::Zero)
        } else {
            let b = u8::read(buf)?;
            let c = u32::read(buf)?;
            if a & 4 == 4 {
                let d = V32::read(buf)?.0;
                Ok(Self::Extended { a, b, c, d })
            } else {
                Ok(Self::Normal { a, b, c })
            }
        }
    }
}

impl Write for LpVec3 {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        match *self {
            Self::Zero => unsafe {
                w.write_byte(0);
            },
            Self::Normal { a, b, c } => unsafe {
                w.write_byte(a);
                w.write_byte(b);
                c.write(w);
            },
            Self::Extended { a, b, c, d } => unsafe {
                w.write_byte(a);
                w.write_byte(b);
                c.write(w);
                V32(d).write(w);
            },
        }
    }

    fn len_s(&self) -> usize {
        match *self {
            Self::Zero => 1,
            Self::Normal { a: _, b: _, c } => 2 + c.len_s(),
            Self::Extended { a: _, b: _, c, d } => 2 + c.len_s() + V32(d).len_s(),
        }
    }
}

impl LpVec3 {
    pub fn new(vec3: Vec3) -> Self {
        let x = Self::sanitize(vec3.x);
        let y = Self::sanitize(vec3.y);
        let z = Self::sanitize(vec3.z);
        let max = x.abs().max(y.abs()).max(z.abs());
        if max < 3.051944088384301E-5 {
            return Self::Zero;
        }

        let divisor = libm::ceil(max) as u64;
        let is_extended = divisor & 3 != divisor;
        let packed_divisor = if is_extended {
            (divisor as u64 & 3) | 4
        } else {
            divisor as u64
        };
        let packed_x = Self::pack_coord(x / (divisor as f64)) << 3;
        let packed_y = Self::pack_coord(y / (divisor as f64)) << 18;
        let packed_z = Self::pack_coord(z / (divisor as f64)) << 33;
        let packed = packed_divisor | packed_x | packed_y | packed_z;

        let a = packed as u8;
        let b = (packed >> 8) as u8;
        let c = (packed >> 16) as u32;

        if is_extended {
            let d = ((divisor as u64) >> 2) as u32;
            Self::Extended { a, b, c, d }
        } else {
            Self::Normal { a, b, c }
        }
    }

    pub fn to_vec3(self) -> Vec3 {
        match self {
            Self::Zero => Vec3::ZERO,
            Self::Normal { a, b, c } => {
                let packed: u64 = (c as u64) << 16 | (b as u64) << 8 | (a as u64);
                let multiplier = (a & 3) as u64 as f64;

                Vec3 {
                    x: Self::unpack_coord(packed >> 3) * multiplier,
                    y: Self::unpack_coord(packed >> 18) * multiplier,
                    z: Self::unpack_coord(packed >> 33) * multiplier,
                }
            }
            Self::Extended { a, b, c, d } => {
                let packed: u64 = (c as u64) << 16 | (b as u64) << 8 | (a as u64);
                let multiplier = (a & 3) as u64;
                let multiplier = multiplier | ((d as u64) << 2);
                let multiplier = multiplier as f64;

                Vec3 {
                    x: Self::unpack_coord(packed >> 3) * multiplier,
                    y: Self::unpack_coord(packed >> 18) * multiplier,
                    z: Self::unpack_coord(packed >> 33) * multiplier,
                }
            }
        }
    }

    fn unpack_coord(value: u64) -> f64 {
        f64::min((value & 32767) as f64, 32766.) * 2. / 32766. - 1.
    }

    fn pack_coord(value: f64) -> u64 {
        libm::round((value * 0.5 + 0.5) * 32766.) as u64
    }

    fn sanitize(value: f64) -> f64 {
        if value.is_nan() {
            0.
        } else {
            f64::clamp(value, -1.7179869183E10, 1.7179869183E10)
        }
    }
}

impl From<LpVec3> for Vec3 {
    fn from(value: LpVec3) -> Self {
        value.to_vec3()
    }
}

impl From<Vec3> for LpVec3 {
    fn from(value: Vec3) -> Self {
        LpVec3::new(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ByteAngle(pub u8);

impl ByteAngle {
    pub fn new(f: f32) -> ByteAngle {
        ByteAngle(libm::floorf(f * 256.0 / 360.0) as u8)
    }

    pub fn to_degrees(self) -> f32 {
        let angle = self.0 as u32 * 360;
        angle as f32 / 256.0
    }
}

impl<'a> Read<'a> for ByteAngle {
    #[inline]
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        Ok(Self(u8::read(buf)?))
    }
}

impl Write for ByteAngle {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe { w.write_byte(self.0) }
    }

    #[inline]
    fn len_s(&self) -> usize {
        1
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct BlockPosPacked(pub i64);

impl Write for BlockPosPacked {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe { self.0.write(w) }
    }

    fn len_s(&self) -> usize {
        self.0.len_s()
    }
}

impl<'a> Read<'a> for BlockPosPacked {
    fn read(buf: &mut &'a [u8]) -> Result<Self, Error> {
        Ok(Self(i64::read(buf)?))
    }
}

impl BlockPosPacked {
    #[must_use]
    pub const fn to_pos(self) -> BlockPos {
        let v = self.0;
        BlockPos {
            x: (v >> 38) as i32,
            y: ((v << 52) >> 52) as i32,
            z: ((v << 26) >> 38) as i32,
        }
    }
}

impl BlockPos {
    #[must_use]
    pub const fn to_i64(self) -> BlockPosPacked {
        let x = (self.x & 0x3FF_FFFF) as i64;
        let y = (self.y & 0xFFF) as i64;
        let z = (self.z & 0x3FF_FFFF) as i64;
        BlockPosPacked((x << 38) | (z << 12) | y)
    }
}
