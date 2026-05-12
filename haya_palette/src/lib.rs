#![no_std]

mod chunk;

extern crate alloc;

pub use self::chunk::{ChunkCache, Direct, Indirect2, Indirect4};
use alloc::boxed::Box;
use core::array::from_fn;
use core::slice::from_raw_parts;
use mser::{Error, Read, Reader, V21, V32, Write, Writer};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Biome(pub u16);

impl<'a> Read<'a> for Biome {
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        Ok(Self(V21::read(buf)?.0 as u16))
    }
}

impl Write for Biome {
    unsafe fn write(&self, w: &mut Writer) {
        unsafe { V21(self.0 as u32).write(w) }
    }

    fn len_s(&self) -> usize {
        V21(self.0 as u32).len_s()
    }
}

#[derive(Clone)]
pub struct PalettedContainer<T: Copy, const P: usize, const L: usize, const H: usize> {
    palette: [T; P],
    len: u8,
    half: Box<[u8; H]>,
    full: Box<[T; L]>,
}

impl<T: Copy, const P: usize, const L: usize, const H: usize> PalettedContainer<T, P, L, H> {
    #[inline]
    pub fn palette(&self) -> &[T] {
        unsafe { from_raw_parts(self.palette.as_ptr(), self.len as usize) }
    }

    #[inline]
    pub const fn palette_len(&self) -> usize {
        self.len as usize
    }
}

impl<T: Copy + Default, const P: usize, const L: usize, const H: usize> Default
    for PalettedContainer<T, P, L, H>
{
    fn default() -> Self {
        Self {
            palette: from_fn(|_| T::default()),
            len: 1,
            half: into_array(alloc::vec![0u8; H].into_boxed_slice()),
            full: into_array(alloc::vec![T::default(); L].into_boxed_slice()),
        }
    }
}

fn into_array<T, const N: usize>(s: Box<[T]>) -> Box<[T; N]> {
    assert_eq!(s.len(), N);

    let ptr = Box::into_raw(s);
    let ptr = ptr as *mut [T; N];

    // SAFETY: The underlying array of a slice has the exact same layout as an
    // actual array `[T; N]` if `N` is equal to the slice's length.
    unsafe { Box::from_raw(ptr) }
}

impl<T: Copy + Default + Eq, const P: usize, const L: usize, const H: usize>
    PalettedContainer<T, P, L, H>
{
    pub fn new(n: T) -> Self {
        let mut p = Self::default();
        p.palette[0] = n;
        p
    }

    /// # Safety
    ///
    /// `index` must be less than `L`.
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        debug_assert!(index < L);
        unsafe {
            if self.len == 0 {
                self.full.get_unchecked(index)
            } else if self.len == 1 {
                self.palette.get_unchecked(0)
            } else {
                self.palette.get_unchecked(
                    ((*self.half.get_unchecked(index / 2) >> ((index & 0b1) << 2)) & 0b1111)
                        as usize,
                )
            }
        }
    }

    unsafe fn grow_zeroed(&mut self, index: usize, val: T) {
        unsafe {
            self.len = 2;
            self.half.fill(0);
            *self.palette.get_unchecked_mut(1) = val;
            let n = 1 << (index % 2 * 4);
            *self.half.get_unchecked_mut(index / 2) = n;
        }
    }

    unsafe fn grow_half_full(&mut self, index: usize, val: T) {
        unsafe {
            self.len = 0;
            for i in 0..L {
                let old = *self.palette.get_unchecked(
                    ((*self.half.get_unchecked(i / 2) >> (i % 2 * 4)) & 0b1111) as usize,
                );
                *self.full.get_unchecked_mut(i) = old;
            }
            *self.full.get_unchecked_mut(index) = val;
        }
    }

    pub fn set(&mut self, index: usize, val: T) -> T {
        debug_assert!(index < L);
        if self.len == 0 {
            unsafe {
                return core::ptr::replace(self.full.get_unchecked_mut(index), val);
            }
        }
        if self.len == 1 {
            unsafe {
                let &first = self.palette().get_unchecked(0);
                if first == val {
                    return first;
                }
                self.grow_zeroed(index, val);
                return first;
            }
        }

        let mut palette_idx = None;
        for (i, x) in self.palette().iter().enumerate() {
            if *x == val {
                palette_idx = Some(i);
                break;
            }
        }

        unsafe {
            let s = index % 2 * 4;
            let n = self.half.get_unchecked_mut(index / 2);
            let old = *self.palette.get_unchecked(((*n >> s) & 0b1111) as usize);

            if let Some(p) = palette_idx {
                *n = (*n & !(0b1111 << s)) | ((p as u8) << s);
            } else if self.len != P as u8 {
                *(self.palette.get_unchecked_mut(self.len as usize)) = val;
                self.len += 1;
                let p = self.len - 1;
                *n = (*n & !(0b1111 << s)) | (p << s);
            } else {
                self.grow_half_full(index, val);
            }
            old
        }
    }

    pub fn reset(&mut self, n: T) {
        self.len = 1;
        self.palette[0] = n;
    }
}

impl<const L: usize, const H: usize> PalettedContainer<u16, 16, L, H> {
    /// # Safety
    ///
    /// [write](Writer::write)
    pub unsafe fn write(&self, bits: u8, w: &mut Writer) {
        unsafe {
            if self.len == 0 {
                // Bits per entry
                w.write_byte(bits);

                // Number of longs in data array
                V32(data_len(L, bits as usize) as u32).write(w);
                // Data array
                let bits_per_u64 = 64 / bits * bits;
                let mut n = 0_u64;
                let mut m = 0;
                for &x in &*self.full {
                    let x = x as u64;
                    n |= x << m;
                    m += bits;
                    if m == bits_per_u64 {
                        m = 0;
                        n.write(w);
                        n = 0;
                    }
                }
                if m > 0 {
                    n.write(w);
                }
            } else if self.len == 1 {
                // Bits per entry
                w.write_byte(0);
                // Palette
                let val = *self.palette().get_unchecked(0);
                V21(val as u32).write(w);
                // Number of longs
                w.write_byte(0);
            } else {
                let bits_per_entry = u8::BITS - (self.len - 1).leading_zeros();
                // Bits per entry
                w.write_byte(bits_per_entry as u8);
                // Palette len
                w.write_byte(self.len);
                // Palette
                for val in self.palette() {
                    val.write(w);
                }
                // Number of longs in data array
                V32(data_len(L, bits_per_entry as usize) as u32).write(w);
                // Data array
                debug_assert!(H.is_multiple_of(8));
                if bits_per_entry == 4 {
                    let ptr = self.half.as_ptr().cast::<[u8; 8]>();
                    for x in 0..H / 8 {
                        let x = *ptr.add(x);
                        w.write(&u64::from_le_bytes(x).to_be_bytes());
                    }
                    return;
                }
                let bits_per_u64 = 64 / bits_per_entry * bits_per_entry;
                let mut n = 0_u64;
                let mut m = 0;
                for &x in &*self.half {
                    n |= ((x & 0b1111) as u64) << m;
                    m += bits_per_entry;
                    if m == bits_per_u64 {
                        m = 0;
                        n.write(w);
                        n = 0;
                    }
                    n |= ((x >> 4) as u64) << m;
                    m += bits_per_entry;
                    if m == bits_per_u64 {
                        m = 0;
                        n.write(w);
                        n = 0;
                    }
                }
                if m > 0 {
                    n.write(w);
                }
            }
        }
    }

    pub fn len_s(&self, bits: u8) -> usize {
        if self.len == 0 {
            let all = data_len(L, bits as usize);

            let mut len = 1;
            len += V32(all as u32).len_s();
            len += all * 8;
            len
        } else if self.len == 1 {
            let val = unsafe { *self.palette().get_unchecked(0) };
            2 + V21(val as u32).len_s()
        } else {
            let bits_per_entry = (u8::BITS - (self.len - 1).leading_zeros()) as usize;
            let mut len = 1;

            let all = data_len(L, bits_per_entry);

            len += 1;
            for pal in self.palette() {
                len += pal.len_s();
            }
            len += V32(all as u32).len_s();
            len += all * 8;
            len
        }
    }
}

impl<const L: usize, const H: usize> PalettedContainer<u16, 4, L, H> {
    /// # Safety
    ///
    /// [write](Writer::write)
    pub unsafe fn write(&self, bits: u8, w: &mut Writer) {
        unsafe {
            if self.len == 0 {
                // Bits per entry
                w.write_byte(bits);

                // Number of longs in data array
                V32(data_len(L, bits as usize) as u32).write(w);
                // Data array
                let bits_per_u64 = 64 / bits * bits;
                let mut n = 0_u64;
                let mut m = 0;
                for &x in &*self.full {
                    let x = x as u64;
                    n |= x << m;
                    m += bits;
                    if m == bits_per_u64 {
                        m = 0;
                        n.write(w);
                        n = 0;
                    }
                }
                if m > 0 {
                    n.write(w);
                }
            } else if self.len == 1 {
                // Bits per entry
                w.write_byte(0);
                // Palette
                let val = *self.palette().get_unchecked(0);
                V21(val as u32).write(w);
                // Number of longs
                w.write_byte(0);
            } else {
                let bits_per_entry = u8::BITS - (self.len - 1).leading_zeros();

                // Bits per entry
                w.write_byte(bits_per_entry as u8);

                // Palette len
                w.write_byte(self.len);
                // Palette
                for &val in self.palette() {
                    val.write(w);
                }

                // Number of longs in data array
                V32(data_len(L, bits_per_entry as usize) as u32).write(w);
                // Data array
                let bits_per_u64 = 64 / bits_per_entry * bits_per_entry;
                let mut n = 0_u64;
                let mut m = 0;
                for &x in &*self.half {
                    n |= ((x & 0b1111) as u64) << m;
                    m += bits_per_entry;
                    if m == bits_per_u64 {
                        m = 0;
                        n.write(w);
                        n = 0;
                    }
                    n |= ((x >> 4) as u64) << m;
                    m += bits_per_entry;
                    if m == bits_per_u64 {
                        m = 0;
                        n.write(w);
                        n = 0;
                    }
                }
                if m > 0 {
                    n.write(w);
                }
            }
        }
    }

    pub fn len_s(&self, bits: u8) -> usize {
        if self.len == 0 {
            let all = data_len(L, bits as usize);

            let mut len = 1;
            len += V32(all as u32).len_s();
            len += all * 8;
            len
        } else if self.len == 1 {
            let val = unsafe { *self.palette().get_unchecked(0) };
            2 + V21(val as u32).len_s()
        } else {
            let bits_per_entry = (u8::BITS - (self.len - 1).leading_zeros()) as usize;
            let mut len = 1;

            let all = data_len(L, bits_per_entry);

            len += 1;
            for pal in self.palette() {
                len += pal.len_s();
            }
            len += V32(all as u32).len_s();
            len += all * 8;
            len
        }
    }
}

#[inline]
const fn data_len(vals_count: usize, bits_per_val: usize) -> usize {
    let div = 64 / bits_per_val;
    if vals_count.is_multiple_of(div) {
        vals_count / div
    } else {
        vals_count / div + 1
    }
}
