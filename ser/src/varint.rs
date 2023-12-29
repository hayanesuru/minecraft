use super::writable::Write;
use super::writer::UnsafeWriter;
use crate::Read;

pub const V21MAX: usize = 0x1FFFFF;
pub const V7MAX: usize = 0x7F;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct V21(pub u32);

impl V21 {
    #[inline]
    pub const fn to_array(self) -> [u8; 3] {
        let n = self.0;
        [n as u8 | 0x80, (n >> 7) as u8 | 0x80, (n >> 14) as u8]
    }
}

impl Read for V21 {
    fn read(buf: &mut &[u8]) -> Option<Self> {
        match **buf {
            [a, ref b @ ..] if (a & 0x80) == 0 => {
                *buf = b;
                Some(Self(a as u32))
            }
            [a, b, ref c @ ..] if (b & 0x80) == 0 => {
                *buf = c;
                Some(Self((a & 0x7F) as u32 | (b as u32) << 7))
            }
            [a, b, c, ref d @ ..] if (c & 0x80) == 0 => {
                *buf = d;
                Some(Self(
                    (a & 0x7F) as u32 | ((b & 0x7F) as u32) << 7 | (c as u32) << 14,
                ))
            }
            [a, b, c, 0x00, ref d @ ..] => {
                *buf = d;
                Some(Self(
                    (a & 0x7F) as u32 | ((b & 0x7F) as u32) << 7 | (c as u32) << 14,
                ))
            }
            [a, b, c, 0x80, 0x00, ref d @ ..] => {
                *buf = d;
                Some(Self(
                    (a & 0x7F) as u32 | ((b & 0x7F) as u32) << 7 | (c as u32) << 14,
                ))
            }
            _ => None,
        }
    }
}

impl Write for V21 {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        let n = self.0;
        if n & 0xFFFFFF80 == 0 {
            w.write_byte(n as u8);
        } else if n & 0xFFFFC000 == 0 {
            w.write(&[n as u8 | 0x80, (n >> 7) as u8]);
        } else {
            w.write(&[n as u8 | 0x80, (n >> 7) as u8 | 0x80, (n >> 14) as u8]);
        }
    }

    #[inline]
    fn len(&self) -> usize {
        let n = self.0;
        if n & 0xFFFFFF80 == 0 {
            1
        } else if n & 0xFFFFC000 == 0 {
            2
        } else {
            3
        }
    }
}

impl V21 {
    pub fn decode(n: &[u8]) -> (usize, u32) {
        match n {
            [a, ..] if (a & 0x80) == 0 => (1, u32::from(*a)),
            [a, b, ..] if (b & 0x80) == 0 => (2, u32::from(a & 0x7F) | u32::from(*b) << 7),
            [a, b, c, ..] if (c & 0x80) == 0 => (
                3,
                u32::from(a & 0x7F) | u32::from(b & 0x7F) << 7 | u32::from(*c) << 14,
            ),
            _ => (0, 0),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct V32(pub u32);

impl V32 {
    #[inline]
    pub const fn to_array(self) -> [u8; 5] {
        let n = self.0;
        [
            n as u8 | 0x80,
            (n >> 7) as u8 | 0x80,
            (n >> 14) as u8 | 0x80,
            (n >> 21) as u8 | 0x80,
            (n >> 28) as u8,
        ]
    }
}

impl Write for V32 {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        let n = self.0;
        if n & 0xFFFFFF80 == 0 {
            w.write_byte(n as u8);
        } else if n & 0xFFFFC000 == 0 {
            w.write(&[n as u8 | 0x80, (n >> 7) as u8]);
        } else if n & 0xFFE00000 == 0 {
            w.write(&[n as u8 | 0x80, (n >> 7) as u8 | 0x80, (n >> 14) as u8]);
        } else if n & 0xF0000000 == 0 {
            w.write(&[
                n as u8 | 0x80,
                (n >> 7) as u8 | 0x80,
                (n >> 14) as u8 | 0x80,
                (n >> 21) as u8,
            ]);
        } else {
            w.write(&[
                n as u8 | 0x80,
                (n >> 7) as u8 | 0x80,
                (n >> 14) as u8 | 0x80,
                (n >> 21) as u8 | 0x80,
                (n >> 28) as u8,
            ]);
        }
    }

    #[inline]
    fn len(&self) -> usize {
        let n = self.0;
        if n & 0xFFFFFF80 == 0 {
            1
        } else if n & 0xFFFFC000 == 0 {
            2
        } else if n & 0xFFE00000 == 0 {
            3
        } else if n & 0xF0000000 == 0 {
            4
        } else {
            5
        }
    }
}

impl Read for V32 {
    fn read(buf: &mut &[u8]) -> Option<Self> {
        match **buf {
            [a, ref b @ ..] if (a & 0x80) == 0 => {
                *buf = b;
                Some(Self(a as u32))
            }
            [a, b, ref c @ ..] if (b & 0x80) == 0 => {
                *buf = c;
                Some(Self((a & 0x7F) as u32 | (b as u32) << 7))
            }
            [a, b, c, ref d @ ..] if (c & 0x80) == 0 => {
                *buf = d;
                Some(Self(
                    (a & 0x7F) as u32 | ((b & 0x7F) as u32) << 7 | (c as u32) << 14,
                ))
            }
            [a, b, c, d, ref e @ ..] if (d & 0x80) == 0 => {
                *buf = e;
                Some(Self(
                    (a & 0x7F) as u32
                        | ((b & 0x7F) as u32) << 7
                        | ((c & 0x7F) as u32) << 14
                        | (d as u32) << 21,
                ))
            }
            [a, b, c, d, e, ref g @ ..] if (e & 0xF0) == 0 => {
                *buf = g;
                Some(Self(
                    (a & 0x7F) as u32
                        | ((b & 0x7F) as u32) << 7
                        | ((c & 0x7F) as u32) << 14
                        | ((d & 0x7F) as u32) << 21
                        | (e as u32) << 28,
                ))
            }
            _ => None,
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct V64(pub u64);

impl V64 {
    pub const fn to_array(self) -> [u8; 10] {
        let n = self.0;
        [
            n as u8 | 0x80,
            (n >> 7) as u8 | 0x80,
            (n >> 14) as u8 | 0x80,
            (n >> 21) as u8 | 0x80,
            (n >> 28) as u8 | 0x80,
            (n >> 35) as u8 | 0x80,
            (n >> 42) as u8 | 0x80,
            (n >> 49) as u8 | 0x80,
            (n >> 56) as u8 | 0x80,
            (n >> 63) as u8,
        ]
    }
}

impl Read for V64 {
    fn read(buf: &mut &[u8]) -> Option<Self> {
        match **buf {
            [a, ref b @ ..] if (a & 0x80) == 0 => {
                *buf = b;
                Some(Self(a as u64))
            }
            [a, b, ref c @ ..] if (b & 0x80) == 0 => {
                *buf = c;
                Some(Self((a & 0x7F) as u64 | (b as u64) << 7))
            }
            [a, b, c, ref d @ ..] if (c & 0x80) == 0 => {
                *buf = d;
                Some(Self(
                    (a & 0x7F) as u64 | ((b & 0x7F) as u64) << 7 | (c as u64) << 14,
                ))
            }
            [a, b, c, d, ref e @ ..] if (d & 0x80) == 0 => {
                *buf = e;
                Some(Self(
                    (a & 0x7F) as u64
                        | ((b & 0x7F) as u64) << 7
                        | ((c & 0x7F) as u64) << 14
                        | (d as u64) << 21,
                ))
            }
            [a, b, c, d, e, ref f @ ..] if (e & 0x80) == 0 => {
                *buf = f;
                Some(Self(
                    (a & 0x7F) as u64
                        | ((b & 0x7F) as u64) << 7
                        | ((c & 0x7F) as u64) << 14
                        | ((d & 0x7F) as u64) << 21
                        | (e as u64) << 28,
                ))
            }
            [a, b, c, d, e, f, ref g @ ..] if (f & 0x80) == 0 => {
                *buf = g;
                Some(Self(
                    (a & 0x7F) as u64
                        | ((b & 0x7F) as u64) << 7
                        | ((c & 0x7F) as u64) << 14
                        | ((d & 0x7F) as u64) << 21
                        | ((e & 0x7F) as u64) << 28
                        | (f as u64) << 35,
                ))
            }
            [a, b, c, d, e, f, g, ref h @ ..] if (g & 0x80) == 0 => {
                *buf = h;
                Some(Self(
                    (a & 0x7F) as u64
                        | ((b & 0x7F) as u64) << 7
                        | ((c & 0x7F) as u64) << 14
                        | ((d & 0x7F) as u64) << 21
                        | ((e & 0x7F) as u64) << 28
                        | ((f & 0x7F) as u64) << 35
                        | (g as u64) << 42,
                ))
            }
            [a, b, c, d, e, f, g, h, ref i @ ..] if (h & 0x80) == 0 => {
                *buf = i;
                Some(Self(
                    (a & 0x7F) as u64
                        | ((b & 0x7F) as u64) << 7
                        | ((c & 0x7F) as u64) << 14
                        | ((d & 0x7F) as u64) << 21
                        | ((e & 0x7F) as u64) << 28
                        | ((f & 0x7F) as u64) << 35
                        | ((g & 0x7F) as u64) << 42
                        | (h as u64) << 49,
                ))
            }
            [a, b, c, d, e, f, g, h, i, ref j @ ..] if (i & 0x80) == 0 => {
                *buf = j;
                Some(Self(
                    (a & 0x7F) as u64
                        | ((b & 0x7F) as u64) << 7
                        | ((c & 0x7F) as u64) << 14
                        | ((d & 0x7F) as u64) << 21
                        | ((e & 0x7F) as u64) << 28
                        | ((f & 0x7F) as u64) << 35
                        | ((g & 0x7F) as u64) << 42
                        | ((h & 0x7F) as u64) << 49
                        | (i as u64) << 56,
                ))
            }
            [a, b, c, d, e, f, g, h, i, j, ref k @ ..] if (j & 0xFE) == 0 => {
                *buf = k;
                Some(Self(
                    (a & 0x7F) as u64
                        | ((b & 0x7F) as u64) << 7
                        | ((c & 0x7F) as u64) << 14
                        | ((d & 0x7F) as u64) << 21
                        | ((e & 0x7F) as u64) << 28
                        | ((f & 0x7F) as u64) << 35
                        | ((g & 0x7F) as u64) << 42
                        | ((h & 0x7F) as u64) << 49
                        | ((i & 0x7F) as u64) << 56
                        | (j as u64) << 63,
                ))
            }
            _ => None,
        }
    }
}
