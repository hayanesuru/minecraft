use crate::{biome, block_state, UnsafeWriter, Write, V21, V32};
use core::alloc::{Allocator, Layout};
use core::mem::{align_of, size_of};
use core::ptr::NonNull;
use core::slice::from_raw_parts;
use std::alloc::{handle_alloc_error, Global};

/// `BPE`, `LEN` > 0
///
/// `PAL` 2..=16
///
/// `BPE` > 0
pub struct PalettedContainer<
    T: Copy + Eq,
    const PAL: usize,
    const BPE: u8,
    const LEN: usize,
    A: Allocator = Global,
> {
    palette: [T; PAL],
    len: usize,
    ptr: NonNull<u8>,
    alloc: A,
}

impl<T: Copy + Eq, A: Allocator, const P: usize, const B: u8, const L: usize>
    PalettedContainer<T, P, B, L, A>
{
    const HALF: usize = {
        if L & 1 == 0 {
            L / 2
        } else {
            L / 2 + 1
        }
    };

    #[inline]
    const fn half(&self) -> usize {
        Self::HALF
    }

    #[inline]
    pub fn palette(&self) -> &[T] {
        unsafe { from_raw_parts(self.palette.as_ptr().cast(), self.len) }
    }

    #[inline]
    pub const fn palette_len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn direct(&self) -> &[T; L] {
        debug_assert!(self.len == 0);
        unsafe { &*self.ptr.as_ptr().cast() }
    }

    #[inline]
    pub fn indirect(&self) -> *const u8 {
        debug_assert!(self.len > 1);
        self.ptr.as_ptr()
    }

    #[inline]
    pub fn indirect_mut(&mut self) -> *mut u8 {
        debug_assert!(self.len > 1);
        self.ptr.as_ptr()
    }
}

impl<T: Copy + Eq, A: Allocator + Clone, const P: usize, const B: u8, const L: usize> Clone
    for PalettedContainer<T, P, B, L, A>
{
    fn clone(&self) -> Self {
        if self.len == 1 {
            return Self {
                palette: self.palette,
                len: self.len,
                ptr: NonNull::dangling(),
                alloc: self.alloc.clone(),
            };
        }
        unsafe {
            let size = if self.len != 0 {
                size_of::<u8>() * Self::HALF
            } else {
                size_of::<T>() * L
            };
            let layout = if self.len != 0 {
                let align = align_of::<u8>();
                Layout::from_size_align_unchecked(size, align)
            } else {
                let align = align_of::<T>();
                Layout::from_size_align_unchecked(size, align)
            };

            let ptr = match self.alloc.allocate(layout) {
                Ok(x) => x,
                Err(_) => handle_alloc_error(layout),
            };
            let mut ptr = NonNull::new_unchecked(ptr.as_ptr().cast::<u8>());
            core::ptr::copy_nonoverlapping(self.ptr.as_ref(), ptr.as_mut(), size);
            Self {
                palette: self.palette,
                len: self.len,
                ptr,
                alloc: self.alloc.clone(),
            }
        }
    }
}

impl<T: Copy + Eq, A: Allocator, const P: usize, const B: u8, const L: usize> Drop
    for PalettedContainer<T, P, B, L, A>
{
    fn drop(&mut self) {
        if self.len == 1 {
            return;
        }
        unsafe {
            let layout = if self.len != 0 {
                let size = size_of::<u8>() * Self::HALF;
                let align = align_of::<u8>();
                Layout::from_size_align_unchecked(size, align)
            } else {
                let size = size_of::<T>() * L;
                let align = align_of::<T>();
                Layout::from_size_align_unchecked(size, align)
            };

            self.alloc.deallocate(self.ptr, layout);
        }
    }
}

impl<T: Copy + Eq, A: Allocator, const P: usize, const B: u8, const L: usize>
    PalettedContainer<T, P, B, L, A>
{
    pub const fn new(n: T, alloc: A) -> Self {
        let mut palette = unsafe { core::mem::zeroed::<[T; P]>() };
        palette[0] = n;
        Self {
            palette,
            len: 1,
            ptr: NonNull::dangling(),
            alloc,
        }
    }

    pub fn get(&self, index: usize) -> T {
        debug_assert!(index < L);
        unsafe {
            if self.len == 0 {
                *self.ptr.as_ptr().cast::<T>().add(index)
            } else if self.len == 1 {
                *self.palette().get_unchecked(0)
            } else {
                *self.palette.get_unchecked(
                    ((*self.ptr.as_ptr().add(index / 2) >> ((index & 0b1) << 2)) & 0b1111) as usize,
                )
            }
        }
    }

    #[cold]
    #[inline(never)]
    unsafe fn grow_zeroed(&mut self, index: usize, val: T) {
        self.len = 2;

        let size = size_of::<u8>() * Self::HALF;
        let align = align_of::<u8>();
        let layout = Layout::from_size_align_unchecked(size, align);
        let data = self.alloc.allocate_zeroed(layout);

        match data {
            Ok(x) => {
                self.ptr = NonNull::new_unchecked(x.cast().as_ptr());
            }
            Err(_) => handle_alloc_error(layout),
        }

        *self.palette.get_unchecked_mut(1) = val;
        let n = 1 << (index % 2 * 4);
        *self.ptr.as_ptr().add(index / 2) = n;
    }

    #[cold]
    #[inline(never)]
    unsafe fn grow(&mut self) {
        let size = size_of::<u8>() * Self::HALF;
        let align = align_of::<u8>();
        let layout = Layout::from_size_align_unchecked(size, align);
        let data = self.alloc.allocate(layout);

        match data {
            Ok(x) => {
                self.ptr = NonNull::new_unchecked(x.cast().as_ptr());
            }
            Err(_) => handle_alloc_error(layout),
        }
    }

    #[cold]
    #[inline(never)]
    unsafe fn grow_full(&mut self, index: usize, val: T) {
        self.len = 0;

        let size = size_of::<T>() * L;
        let align = align_of::<T>();
        let layout = Layout::from_size_align_unchecked(size, align);
        let data = self.alloc.allocate(layout);

        let indirect = match data {
            Ok(x) => core::mem::replace(&mut self.ptr, x.cast()),
            Err(_) => handle_alloc_error(layout),
        };
        if indirect.as_ptr() == NonNull::dangling().as_ptr() {
            return;
        }
        for index in 0..L {
            let val = *self.palette.get_unchecked(
                (*indirect.as_ptr().add(index / 2) >> (index % 2 * 4) & 0b1111) as usize,
            );
            self.ptr.as_ptr().cast::<T>().add(index).write(val);
        }
        self.ptr.as_ptr().cast::<T>().add(index).write(val);

        let size = size_of::<u8>() * Self::HALF;
        let align = align_of::<u8>();
        let layout = Layout::from_size_align_unchecked(size, align);
        self.alloc.deallocate(indirect, layout);
    }

    pub fn set(&mut self, index: usize, val: T) -> T {
        debug_assert!(index < L);
        if self.len == 0 {
            unsafe {
                return core::ptr::replace(self.ptr.as_ptr().cast::<T>().add(index), val);
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
            }
        }

        unsafe {
            let s = index % 2 * 4;
            let n = self.ptr.as_ptr().add(index / 2);
            let old = *self.palette().get_unchecked((*n >> s & 0b1111) as usize);

            if let Some(p) = palette_idx {
                *n = (*n & !(0b1111 << s)) | ((p as u8) << s);
            } else if self.len != P {
                *(self.palette.get_unchecked_mut(self.len)) = val;
                self.len += 1;
                let p = self.len - 1;
                *n = (*n & !(0b1111 << s)) | ((p as u8) << s);
            } else {
                self.grow_full(index, val);
            }
            old
        }
    }
}

impl<A: Allocator, const B: u8, const L: usize> Write
    for PalettedContainer<block_state, 16, B, L, A>
{
    fn write(&self, w: &mut UnsafeWriter) {
        if self.len == 0 {
            // Bits per entry
            w.write_byte(B);

            // Number of longs in data array.
            V32(data_len(L, B as usize) as u32).write(w);
            // Data array
            let vals_per_u64 = 64 / B * B;
            let mut n = 0_u64;
            let mut m = 0;
            for &x in self.direct() {
                let x = x.id() as u64;
                n |= x << m;
                m += B;
                if m == vals_per_u64 {
                    m = 0;
                    n.write(w);
                    n = 0;
                }
            }
            if m > 0 {
                n.write(w);
            }
            return;
        }
        if self.len == 1 {
            // Bits per entry
            w.write_byte(0);
            // Palette
            let val = unsafe { *self.palette().get_unchecked(0) };
            val.write(w);
            // Number of longs
            w.write_byte(0);
            return;
        }

        let bits_per_entry = 4_u8;

        // Bits per entry
        w.write_byte(bits_per_entry);

        // Palette len
        w.write_byte(self.len as u8);
        // Palette
        for val in self.palette() {
            (*val).write(w);
        }

        // Number of longs in data array.
        V32(data_len(L, bits_per_entry as usize) as u32).write(w);
        // Data array
        for x in 0..self.half() / 8 {
            let x = unsafe { *self.ptr.as_ptr().add(x * 8).cast::<[u8; 8]>() };
            w.write(&u64::from_le_bytes(x).to_be_bytes());
        }
    }

    fn len(&self) -> usize {
        if self.len == 0 {
            let all = data_len(L, B as usize);

            let mut len = 1;
            len += V32(all as u32).len();
            len += all * 8;
            len
        } else if self.len == 1 {
            let val = unsafe { *self.palette().get_unchecked(0) };
            2 + val.len()
        } else {
            debug_assert!(u8::BITS - (self.len as u8 - 1).leading_zeros() <= 4);
            let bits_per_entry = 4_usize;
            let mut len = 1;

            let all = data_len(L, bits_per_entry);

            len += 1;
            for val in self.palette() {
                len += (*val).len();
            }
            len += V32(all as u32).len();
            len += all * 8;
            len
        }
    }
}

impl<A: Allocator, const P: usize, const B: u8, const L: usize> Write
    for PalettedContainer<biome, P, B, L, A>
{
    fn write(&self, w: &mut UnsafeWriter) {
        if self.len == 0 {
            // Bits per entry
            w.write_byte(B);

            // Number of longs in data array.
            V32(data_len(L, B as usize) as u32).write(w);
            // Data array
            let vals_per_u64 = 64 / B * B;
            let mut n = 0_u64;
            let mut m = 0;
            for &x in self.direct() {
                let x = x as u64;
                n |= x << m;
                m += B;
                if m == vals_per_u64 {
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
            let val = unsafe { *self.palette().get_unchecked(0) };
            val.write(w);
            // Number of longs
            w.write_byte(0);
        } else {
            let bits_per_entry = u8::BITS - (self.len as u8 - 1).leading_zeros();

            // Bits per entry
            V32(bits_per_entry).write(w);

            // Palette len
            w.write_byte(self.len as u8);
            // Palette
            for &val in self.palette() {
                val.write(w);
            }

            // Number of longs in data array.
            V32(data_len(L, bits_per_entry as usize) as u32).write(w);
            // Data array
            let vals_per_u64 = 64 / bits_per_entry * bits_per_entry;
            let mut n = 0_u64;
            let mut m = 0;
            for &x in unsafe { from_raw_parts(self.indirect(), self.half()) } {
                n |= ((x & 0b1111) as u64) << m;
                m += bits_per_entry;
                if m == vals_per_u64 {
                    m = 0;
                    n.write(w);
                    n = 0;
                }
                n |= ((x >> 4) as u64) << m;
                m += bits_per_entry;
                if m == vals_per_u64 {
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

    fn len(&self) -> usize {
        if self.len == 0 {
            let all = data_len(L, B as usize);

            let mut len = 1;
            len += V32(all as u32).len();
            len += all * 8;
            len
        } else if self.len == 1 {
            let val = unsafe { *self.palette().get_unchecked(0) };
            2 + V21(val as u32).len()
        } else {
            let bits_per_entry = (u8::BITS - (self.len as u8 - 1).leading_zeros()) as usize;
            let mut len = 1;

            let all = data_len(L, bits_per_entry);

            len += 1;
            len += self.len;
            len += V32(all as u32).len();
            len += all * 8;
            len
        }
    }
}

#[inline]
const fn data_len(vals_count: usize, bits_per_val: usize) -> usize {
    let vals_per_u64 = 64 / bits_per_val;
    if vals_count % vals_per_u64 == 0 {
        vals_count / vals_per_u64
    } else {
        vals_count / vals_per_u64 + 1
    }
}

impl<T: Copy + Eq, A: Allocator + Clone, const P: usize, const B: u8, const L: usize>
    PalettedContainer<T, P, B, L, A>
{
    pub fn shrink_to_fit(&mut self, mut f: impl FnMut(T, usize)) {
        if self.len == 1 {
            let val = unsafe { *self.palette().get_unchecked(0) };
            f(val, L);
            return;
        }

        if self.len == 0 {
            let mut pal = unsafe { core::mem::zeroed::<[T; P]>() };
            let mut count = [0_usize; P];
            let mut len = 0;
            for &x in self.direct() {
                let pos = unsafe {
                    pal.get_unchecked(0..if len == P + 1 { P } else { len })
                        .iter()
                        .position(|p| *p == x)
                };
                match pos {
                    Some(x) => unsafe { *count.get_unchecked_mut(x) += 1 },
                    None => unsafe {
                        if len == P + 1 {
                            f(x, 1);
                        } else if len == P {
                            len += 1;
                            f(x, 1);
                        } else {
                            *count.get_unchecked_mut(len) = 1;
                            *pal.get_unchecked_mut(len) = x;
                            len += 1;
                        }
                    },
                }
            }
            if len == P + 1 {
                for index in 0..P {
                    unsafe {
                        let pal = *pal.get_unchecked(index);
                        let count = *count.get_unchecked(index);
                        f(pal, count);
                    }
                }
                return;
            }
            let palette = unsafe {
                let mut arr =
                    core::array::from_fn::<_, P, _>(|i| (*count.get_unchecked(i), i as u8));
                arr.sort_unstable_by_key(|(x, _)| *x);
                core::array::from_fn::<_, P, _>(|i| {
                    *pal.get_unchecked(arr.get_unchecked(i).1 as usize)
                })
            };
            let prev = core::mem::replace(
                self,
                PalettedContainer {
                    palette,
                    len,
                    ptr: NonNull::dangling(),
                    alloc: self.alloc.clone(),
                },
            );

            if len == 1 {
                return;
            }

            unsafe {
                self.grow();
                let ptr = prev.ptr.as_ptr().cast::<T>();

                for index in 0..self.half() - 1 {
                    let val1 = ptr.add(index * 2);
                    let val2 = ptr.add(index * 2 + 1);

                    let mut p1 = 0;
                    let mut p2 = 0;
                    for (i, x) in self.palette().iter().enumerate() {
                        if *x == *val1 {
                            p1 = i;
                            break;
                        }
                    }
                    for (i, x) in self.palette().iter().enumerate() {
                        if *x == *val2 {
                            p2 = i;
                            break;
                        }
                    }

                    let n = self.ptr.as_ptr().add(index);
                    *n = ((p2 as u8) << 4) | (p1 as u8);
                }
                let index = self.half() - 1;
                let mut p1 = 0;
                let mut p2 = 0;

                let val1 = ptr.add(index * 2);
                for (i, x) in self.palette().iter().enumerate() {
                    if *x == *val1 {
                        p1 = i;
                        break;
                    }
                }

                if L - 1 == index * 2 + 1 {
                    let val2 = ptr.add(index * 2 + 1);
                    for (i, x) in self.palette().iter().enumerate() {
                        if *x == *val2 {
                            p2 = i;
                            break;
                        }
                    }
                }

                let n = self.ptr.as_ptr().add(index);
                *n = ((p2 as u8) << 4) | (p1 as u8);
            }
            return;
        }

        let mut arr = [(0_usize, 0_u8); P];
        for (idx, ele) in arr.iter_mut().enumerate() {
            ele.1 = idx as u8;
        }
        for x in 0..Self::HALF {
            unsafe {
                let y = *self.ptr.as_ptr().add(x);
                let a = y & 0b1111;
                let b = y >> 4;
                arr.get_unchecked_mut(a as usize).0 += 1;
                arr.get_unchecked_mut(b as usize).0 += 1;
            }
        }
        let changed = !arr.is_sorted_by_key(|x| x.0);
        if changed {
            arr.sort_unstable_by_key(|x| usize::MAX - x.0);
        }
        let mut arr2 = [0_u8; P];
        let mut len = 0;
        for (count, ele) in arr {
            if count == 0 {
                break;
            }
            unsafe {
                *arr2.get_unchecked_mut(ele as usize) = len as u8;
                let x = *self.palette.get_unchecked(ele as usize);
                f(x, count);
            }
            len += 1;
        }

        debug_assert_ne!(len, 0);

        if len == 1 {
            *self = Self::new(self.get(0), self.alloc.clone());
            return;
        }
        self.len = len;
        if changed {
            let mut palette = unsafe { core::mem::zeroed::<[T; P]>() };
            for (index, (_, ele)) in arr.into_iter().enumerate() {
                unsafe {
                    *palette.get_unchecked_mut(index) = *self.palette.get_unchecked(ele as usize);
                }
            }
            self.palette = palette;
            for x in 0..Self::HALF {
                unsafe {
                    let x = self.ptr.as_ptr().add(x);
                    *x = *arr2.get_unchecked((*x & 0b1111) as usize)
                        | ((*arr2.get_unchecked((*x >> 4) as usize)) << 4);
                }
            }
        }
    }
}
