use super::{Read, UnsafeWriter, Write};
use crate::{Error, likely, unlikely};
use core::slice::from_raw_parts;

pub const V21MAX: usize = 0x1FFFFF;
pub const V7MAX: usize = 0x7F;

#[repr(transparent)]
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct V21(pub u32);

impl V21 {
    #[inline]
    pub const fn to_array(self) -> [u8; 3] {
        let n = self.0;
        [n as u8 | 0x80, (n >> 7) as u8 | 0x80, (n >> 14) as u8]
    }
}

impl<'a> Read<'a> for V21 {
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        unsafe {
            let mut ptr = buf.as_ptr();
            let len = buf.len();

            if unlikely(len == 0) {
                return Err(Error);
            }
            let a = *ptr;
            ptr = ptr.add(1);
            if likely((a & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 1);
                return Ok(Self(a as u32));
            }

            if unlikely(len == 1) {
                return Err(Error);
            }
            let b = *ptr;
            ptr = ptr.add(1);
            if likely((b & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 2);
                return Ok(Self((a & 0x7F) as u32 | ((b as u32) << 7)));
            }

            if unlikely(len == 2) {
                return Err(Error);
            }
            let c = *ptr;
            ptr = ptr.add(1);
            let p = (a & 0x7F) as u32 | (((b & 0x7F) as u32) << 7) | ((c as u32) << 14);
            if likely((c & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 3);
                return Ok(Self(p));
            }

            if unlikely(len == 3) {
                return Err(Error);
            }
            let d = *ptr;
            ptr = ptr.add(1);
            if unlikely(d == 0x00) {
                *buf = from_raw_parts(ptr, len - 4);
                return Ok(Self(p));
            }
            if unlikely(len == 4) {
                return Err(Error);
            }
            let e = *ptr;
            ptr = ptr.add(1);
            if likely(d == 0x80 && e == 0x00) {
                *buf = from_raw_parts(ptr, len - 5);
                return Ok(Self(p));
            }
            Err(Error)
        }
    }
}

impl Write for V21 {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            let n = self.0;
            if n & 0xFFFFFF80 == 0 {
                w.write_byte(n as u8);
            } else if n & 0xFFFFC000 == 0 {
                w.write(&[n as u8 | 0x80, (n >> 7) as u8]);
            } else {
                w.write(&[n as u8 | 0x80, (n >> 7) as u8 | 0x80, (n >> 14) as u8]);
            }
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
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

#[repr(transparent)]
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
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
    }

    #[inline]
    fn len_s(&self) -> usize {
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

impl<'a> Read<'a> for V32 {
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        unsafe {
            let mut ptr = buf.as_ptr();
            let len = buf.len();

            if unlikely(len == 0) {
                return Err(Error);
            }
            let a = *ptr;
            ptr = ptr.add(1);
            if likely((a & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 1);
                return Ok(Self(a as u32));
            }

            if unlikely(len == 1) {
                return Err(Error);
            }
            let b = *ptr;
            ptr = ptr.add(1);
            if likely((b & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 2);
                return Ok(Self((a & 0x7F) as u32 | ((b as u32) << 7)));
            }

            if unlikely(len == 2) {
                return Err(Error);
            }
            let c = *ptr;
            ptr = ptr.add(1);
            if unlikely((c & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 3);
                return Ok(Self(
                    (a & 0x7F) as u32 | (((b & 0x7F) as u32) << 7) | ((c as u32) << 14),
                ));
            }

            if unlikely(len == 3) {
                return Err(Error);
            }
            let d = *ptr;
            ptr = ptr.add(1);
            if unlikely((d & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 4);
                return Ok(Self(
                    (a & 0x7F) as u32
                        | (((b & 0x7F) as u32) << 7)
                        | (((c & 0x7F) as u32) << 14)
                        | ((d as u32) << 21),
                ));
            }

            if unlikely(len == 4) {
                return Err(Error);
            }
            let e = *ptr;
            ptr = ptr.add(1);
            if (e & 0xF0) == 0 {
                *buf = core::slice::from_raw_parts(ptr, len - 5);
                return Ok(Self(
                    (a & 0x7F) as u32
                        | (((b & 0x7F) as u32) << 7)
                        | (((c & 0x7F) as u32) << 14)
                        | (((d & 0x7F) as u32) << 21)
                        | ((e as u32) << 28),
                ));
            }
            Err(Error)
        }
    }
}

#[repr(transparent)]
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct V64(pub u64);

impl V64 {
    #[inline]
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

impl Write for V64 {
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            let n = self.0;
            if n & 0xFFFFFFFFFFFFFF80 == 0 {
                w.write_byte(n as u8);
            } else if n & 0x8000000000000000 != 0 {
                w.write(&[
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
                ]);
            } else if n & 0xFFFFFFFFFFFFC000 == 0 {
                w.write(&[n as u8 | 0x80, (n >> 7) as u8]);
            } else if n & 0xFFFFFFFFFFE00000 == 0 {
                w.write(&[n as u8 | 0x80, (n >> 7) as u8 | 0x80, (n >> 14) as u8]);
            } else if n & 0xFFFFFFFFF0000000 == 0 {
                w.write(&[
                    n as u8 | 0x80,
                    (n >> 7) as u8 | 0x80,
                    (n >> 14) as u8 | 0x80,
                    (n >> 21) as u8,
                ]);
            } else if n & 0xFFFFFFF800000000 == 0 {
                w.write(&[
                    n as u8 | 0x80,
                    (n >> 7) as u8 | 0x80,
                    (n >> 14) as u8 | 0x80,
                    (n >> 21) as u8 | 0x80,
                    (n >> 28) as u8,
                ]);
            } else if n & 0xFFFFFC0000000000 == 0 {
                w.write(&[
                    n as u8 | 0x80,
                    (n >> 7) as u8 | 0x80,
                    (n >> 14) as u8 | 0x80,
                    (n >> 21) as u8 | 0x80,
                    (n >> 28) as u8 | 0x80,
                    (n >> 35) as u8,
                ]);
            } else if n & 0xFFFE000000000000 == 0 {
                w.write(&[
                    n as u8 | 0x80,
                    (n >> 7) as u8 | 0x80,
                    (n >> 14) as u8 | 0x80,
                    (n >> 21) as u8 | 0x80,
                    (n >> 28) as u8 | 0x80,
                    (n >> 35) as u8 | 0x80,
                    (n >> 42) as u8,
                ]);
            } else if n & 0xFF00000000000000 == 0 {
                w.write(&[
                    n as u8 | 0x80,
                    (n >> 7) as u8 | 0x80,
                    (n >> 14) as u8 | 0x80,
                    (n >> 21) as u8 | 0x80,
                    (n >> 28) as u8 | 0x80,
                    (n >> 35) as u8 | 0x80,
                    (n >> 42) as u8 | 0x80,
                    (n >> 49) as u8,
                ]);
            } else {
                w.write(&[
                    n as u8 | 0x80,
                    (n >> 7) as u8 | 0x80,
                    (n >> 14) as u8 | 0x80,
                    (n >> 21) as u8 | 0x80,
                    (n >> 28) as u8 | 0x80,
                    (n >> 35) as u8 | 0x80,
                    (n >> 42) as u8 | 0x80,
                    (n >> 49) as u8 | 0x80,
                    (n >> 56) as u8,
                ]);
            }
        }
    }

    fn len_s(&self) -> usize {
        let n = self.0;
        if n & 0xFFFFFFFFFFFFFF80 == 0 {
            1
        } else if n & 0x8000000000000000 != 0 {
            10
        } else if n & 0xFFFFFFFFFFFFC000 == 0 {
            2
        } else if n & 0xFFFFFFFFFFE00000 == 0 {
            3
        } else if n & 0xFFFFFFFFF0000000 == 0 {
            4
        } else if n & 0xFFFFFFF800000000 == 0 {
            5
        } else if n & 0xFFFFFC0000000000 == 0 {
            6
        } else if n & 0xFFFE000000000000 == 0 {
            7
        } else if n & 0xFF00000000000000 == 0 {
            8
        } else {
            9
        }
    }
}

impl<'a> Read<'a> for V64 {
    fn read(buf: &mut &[u8]) -> Result<Self, Error> {
        unsafe {
            let mut ptr = buf.as_ptr();
            let len = buf.len();

            if unlikely(len == 0) {
                return Err(Error);
            }
            let a = *ptr;
            ptr = ptr.add(1);
            if likely((a & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 1);
                return Ok(Self(a as u64));
            }

            if unlikely(len == 1) {
                return Err(Error);
            }
            let b = *ptr;
            ptr = ptr.add(1);
            if likely((b & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 2);
                return Ok(Self((a & 0x7F) as u64 | ((b as u64) << 7)));
            }

            if likely(len >= 10) {
                let y = u64::from_le_bytes(*ptr.cast::<[u8; 8]>());
                if unlikely(y & 0xFE80_8080_8080_8080 == 0x0080_8080_8080_8080) {
                    *buf = from_raw_parts(ptr.add(8), len - 10);
                    return Ok(Self(
                        ((a & 0x7F) as u64)
                            | (((b & 0x7F) as u64) << 7)
                            | ((y & 0x0000_0000_0000_007F) << 14)
                            | ((y & 0x0000_0000_0000_7F00) << 13)
                            | ((y & 0x0000_0000_007F_0000) << 12)
                            | ((y & 0x0000_0000_7F00_0000) << 11)
                            | ((y & 0x0000_007F_0000_0000) << 10)
                            | ((y & 0x0000_7F00_0000_0000) << 9)
                            | ((y & 0x007F_0000_0000_0000) << 8)
                            | ((y & 0x0100_0000_0000_0000) << 7),
                    ));
                }
            }

            if unlikely(len == 2) {
                return Err(Error);
            }
            let c = *ptr;
            ptr = ptr.add(1);
            if unlikely((c & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 3);
                return Ok(Self(
                    (a & 0x7F) as u64 | (((b & 0x7F) as u64) << 7) | ((c as u64) << 14),
                ));
            }

            if unlikely(len == 3) {
                return Err(Error);
            }
            let d = *ptr;
            ptr = ptr.add(1);
            if unlikely((d & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 4);
                return Ok(Self(
                    (a & 0x7F) as u64
                        | (((b & 0x7F) as u64) << 7)
                        | (((c & 0x7F) as u64) << 14)
                        | ((d as u64) << 21),
                ));
            }
            if unlikely(len == 4) {
                return Err(Error);
            }
            let e = *ptr;
            ptr = ptr.add(1);
            if likely((e & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 5);
                return Ok(Self(
                    (a & 0x7F) as u64
                        | (((b & 0x7F) as u64) << 7)
                        | (((c & 0x7F) as u64) << 14)
                        | (((d & 0x7F) as u64) << 21)
                        | ((e as u64) << 28),
                ));
            }

            if unlikely(len == 5) {
                return Err(Error);
            }
            let f = *ptr;
            ptr = ptr.add(1);
            if likely((f & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 6);
                return Ok(Self(
                    (a & 0x7F) as u64
                        | (((b & 0x7F) as u64) << 7)
                        | (((c & 0x7F) as u64) << 14)
                        | (((d & 0x7F) as u64) << 21)
                        | (((e & 0x7F) as u64) << 28)
                        | ((f as u64) << 35),
                ));
            }

            if unlikely(len == 6) {
                return Err(Error);
            }
            let g = *ptr;
            ptr = ptr.add(1);
            if likely((g & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 7);
                return Ok(Self(
                    (a & 0x7F) as u64
                        | (((b & 0x7F) as u64) << 7)
                        | (((c & 0x7F) as u64) << 14)
                        | (((d & 0x7F) as u64) << 21)
                        | (((e & 0x7F) as u64) << 28)
                        | (((f & 0x7F) as u64) << 35)
                        | ((g as u64) << 42),
                ));
            }

            if unlikely(len == 7) {
                return Err(Error);
            }
            let h = *ptr;
            ptr = ptr.add(1);
            if likely((h & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 8);
                return Ok(Self(
                    (a & 0x7F) as u64
                        | (((b & 0x7F) as u64) << 7)
                        | (((c & 0x7F) as u64) << 14)
                        | (((d & 0x7F) as u64) << 21)
                        | (((e & 0x7F) as u64) << 28)
                        | (((f & 0x7F) as u64) << 35)
                        | (((g & 0x7F) as u64) << 42)
                        | ((h as u64) << 49),
                ));
            }

            if unlikely(len == 8) {
                return Err(Error);
            }
            let i = *ptr;
            ptr = ptr.add(1);
            if likely((i & 0x80) == 0) {
                *buf = from_raw_parts(ptr, len - 9);
                return Ok(Self(
                    (a & 0x7F) as u64
                        | (((b & 0x7F) as u64) << 7)
                        | (((c & 0x7F) as u64) << 14)
                        | (((d & 0x7F) as u64) << 21)
                        | (((e & 0x7F) as u64) << 28)
                        | (((f & 0x7F) as u64) << 35)
                        | (((g & 0x7F) as u64) << 42)
                        | (((h & 0x7F) as u64) << 49)
                        | ((i as u64) << 56),
                ));
            }
        }

        Err(Error)
    }
}

#[test]
fn test_varint() {
    let mut r = 0xE3D172B05F73CBC3u64;
    let mut buf = [0u8; 10];

    for _ in 0..100_000 {
        r = r.wrapping_add(0xa0761d6478bd642f);
        let x = (r ^ 0xe7037ed1a0b428db) as u128;
        let t = (r as u128).wrapping_mul(x);
        let x = (t.wrapping_shr(64) ^ t) as u64;
        unsafe {
            let mut w = crate::UnsafeWriter(core::ptr::NonNull::new_unchecked(buf.as_mut_ptr()));
            let y = V64(x);
            y.write(&mut w);
            let sz = y.len_s();
            assert_eq!(buf.as_ptr().add(sz), w.ptr().as_ptr());
            let mut sl = core::slice::from_raw_parts(buf.as_ptr(), sz);
            assert_eq!(V64::read(&mut sl).unwrap(), y);
            assert!(sl.is_empty());

            let mut w = crate::UnsafeWriter(core::ptr::NonNull::new_unchecked(buf.as_mut_ptr()));
            let y = V32(x as u32);
            y.write(&mut w);
            let sz = y.len_s();
            assert_eq!(buf.as_ptr().add(sz), w.ptr().as_ptr());
            let mut sl = core::slice::from_raw_parts(buf.as_ptr(), sz);
            assert_eq!(V32::read(&mut sl).unwrap(), y);
            assert!(sl.is_empty());

            let mut w = crate::UnsafeWriter(core::ptr::NonNull::new_unchecked(buf.as_mut_ptr()));
            let y = V21(x as u32 & 0x1FFFFF);
            y.write(&mut w);
            let sz = y.len_s();
            assert_eq!(buf.as_ptr().add(sz), w.ptr().as_ptr());
            let mut sl = core::slice::from_raw_parts(buf.as_ptr(), sz);
            assert_eq!(V21::read(&mut sl).unwrap(), y);
            assert!(sl.is_empty());
        }
    }
}
