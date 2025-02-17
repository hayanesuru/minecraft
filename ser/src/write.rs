use crate::{UnsafeWriter, Write};
use alloc::borrow::{Cow, ToOwned};
use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
    NonZeroU32, NonZeroU64, NonZeroU8,
};
use core::ptr::NonNull;

macro_rules! primitive {
    ($type:ty) => {
        unsafe impl Write for $type {
            #[inline(always)]
            unsafe fn write(&self, w: &mut UnsafeWriter) {
                w.write(&self.to_be_bytes());
            }

            #[inline(always)]
            unsafe fn sz(&self) -> usize {
                ::core::mem::size_of::<Self>()
            }
        }
    };
}

macro_rules! non_zero {
    ($type:ty) => {
        unsafe impl Write for $type {
            #[inline(always)]
            unsafe fn write(&self, w: &mut UnsafeWriter) {
                w.write(&self.get().to_be_bytes());
            }

            #[inline(always)]
            unsafe fn sz(&self) -> usize {
                ::core::mem::size_of::<Self>()
            }
        }
    };
}

pub struct Write2<'a, A: ?Sized, B: ?Sized> {
    pub a: &'a A,
    pub b: &'a B,
}

unsafe impl<A: Write + ?Sized, B: Write + ?Sized> Write for Write2<'_, A, B> {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        self.a.write(w);
        self.b.write(w);
    }

    #[inline]
    unsafe fn sz(&self) -> usize {
        self.a.sz() + self.b.sz()
    }
}

pub struct Write3<'a, A: ?Sized, B: ?Sized, C: ?Sized> {
    pub a: &'a A,
    pub b: &'a B,
    pub c: &'a C,
}

unsafe impl<A: Write + ?Sized, B: Write + ?Sized, C: Write + ?Sized> Write for Write3<'_, A, B, C> {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        self.a.write(w);
        self.b.write(w);
        self.c.write(w);
    }

    #[inline]
    unsafe fn sz(&self) -> usize {
        self.a.sz() + self.b.sz() + self.c.sz()
    }
}

unsafe impl<T: Write> Write for alloc::slice::Iter<'_, T> {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        self.clone().for_each(|x| x.write(w));
    }

    #[inline(always)]
    unsafe fn sz(&self) -> usize {
        self.clone().map(|x| x.sz()).sum()
    }
}

unsafe impl<T: Write> Write for alloc::slice::IterMut<'_, T> {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        self.as_slice().iter().for_each(|x| x.write(w));
    }

    #[inline(always)]
    unsafe fn sz(&self) -> usize {
        self.as_slice().iter().map(|x| x.sz()).sum()
    }
}

unsafe impl Write for bool {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(*self as u8)
    }

    #[inline(always)]
    unsafe fn sz(&self) -> usize {
        1
    }
}

unsafe impl Write for NonZeroI8 {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(self.get() as u8)
    }

    #[inline(always)]
    unsafe fn sz(&self) -> usize {
        1
    }
}

unsafe impl Write for NonZeroU8 {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(self.get())
    }

    #[inline(always)]
    unsafe fn sz(&self) -> usize {
        1
    }
}

unsafe impl Write for u8 {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(*self)
    }

    #[inline(always)]
    unsafe fn sz(&self) -> usize {
        1
    }
}

unsafe impl Write for i8 {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(*self as u8)
    }

    #[inline(always)]
    unsafe fn sz(&self) -> usize {
        1
    }
}

primitive!(i16);
primitive!(i32);
primitive!(i64);
primitive!(u16);
primitive!(u32);
primitive!(u64);
primitive!(f32);
primitive!(f64);
non_zero!(NonZeroI16);
non_zero!(NonZeroI32);
non_zero!(NonZeroI64);
non_zero!(NonZeroI128);
non_zero!(NonZeroU16);
non_zero!(NonZeroU32);
non_zero!(NonZeroU64);
non_zero!(NonZeroU128);

unsafe impl Write for str {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        w.write(self.as_bytes());
    }

    #[inline(always)]
    unsafe fn sz(&self) -> usize {
        str::len(self)
    }
}

unsafe impl Write for [u8] {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        w.write(self);
    }

    #[inline(always)]
    unsafe fn sz(&self) -> usize {
        <[u8]>::len(self)
    }
}

unsafe impl Write for &str {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        w.write(self.as_bytes());
    }

    #[inline(always)]
    unsafe fn sz(&self) -> usize {
        str::len(self)
    }
}

unsafe impl Write for uuid::Uuid {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        w.write(self.as_bytes())
    }

    #[inline(always)]
    unsafe fn sz(&self) -> usize {
        16
    }
}

unsafe impl Write for &[u8] {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        w.write(self);
    }

    #[inline]
    unsafe fn sz(&self) -> usize {
        <[u8]>::len(self)
    }
}

unsafe impl<T: Write + ?Sized> Write for NonNull<T> {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        self.as_ref().write(w);
    }

    #[inline]
    unsafe fn sz(&self) -> usize {
        self.as_ref().sz()
    }
}

unsafe impl<T: ?Sized + Write + ToOwned> Write for Cow<'_, T> {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        <Self as AsRef<T>>::as_ref(self).write(w);
    }

    #[inline]
    unsafe fn sz(&self) -> usize {
        <Self as AsRef<T>>::as_ref(self).sz()
    }
}

unsafe impl<T: Write> Write for Option<T> {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        match self.as_ref() {
            Some(x) => {
                w.write_byte(1);
                x.write(w);
            }
            None => {
                w.write_byte(0);
            }
        }
    }

    #[inline]
    unsafe fn sz(&self) -> usize {
        match self.as_ref() {
            Some(x) => 1 + x.sz(),
            None => 1,
        }
    }
}
