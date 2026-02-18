use crate::{UnsafeWriter, Write};
use core::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroU8, NonZeroU16, NonZeroU32,
    NonZeroU64, NonZeroU128,
};
use core::ptr::NonNull;

macro_rules! primitive {
    ($type:ty) => {
        impl Write for $type {
            #[inline(always)]
            unsafe fn write(&self, w: &mut UnsafeWriter) {
                unsafe {
                    w.write(&self.to_be_bytes());
                }
            }

            #[inline(always)]
            fn len_s(&self) -> usize {
                ::core::mem::size_of::<Self>()
            }
        }
    };
}

macro_rules! non_zero {
    ($type:ty) => {
        impl Write for $type {
            #[inline(always)]
            unsafe fn write(&self, w: &mut UnsafeWriter) {
                unsafe {
                    w.write(&self.get().to_be_bytes());
                }
            }

            #[inline(always)]
            fn len_s(&self) -> usize {
                ::core::mem::size_of::<Self>()
            }
        }
    };
}

impl Write for bool {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            w.write_byte(*self as u8);
        }
    }

    #[inline(always)]
    fn len_s(&self) -> usize {
        1
    }
}

impl Write for NonZeroI8 {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            w.write_byte(self.get() as u8);
        }
    }

    #[inline(always)]
    fn len_s(&self) -> usize {
        1
    }
}

impl Write for NonZeroU8 {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            w.write_byte(self.get());
        }
    }

    #[inline(always)]
    fn len_s(&self) -> usize {
        1
    }
}

impl Write for u8 {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            w.write_byte(*self);
        }
    }

    #[inline(always)]
    fn len_s(&self) -> usize {
        1
    }
}

impl Write for i8 {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            w.write_byte(*self as u8);
        }
    }

    #[inline(always)]
    fn len_s(&self) -> usize {
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

impl Write for str {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            w.write(self.as_bytes());
        }
    }

    #[inline(always)]
    fn len_s(&self) -> usize {
        str::len(self)
    }
}

impl Write for [u8] {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            w.write(self);
        }
    }

    #[inline(always)]
    fn len_s(&self) -> usize {
        <[u8]>::len(self)
    }
}

impl Write for &str {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            w.write(self.as_bytes());
        }
    }

    #[inline(always)]
    fn len_s(&self) -> usize {
        str::len(self)
    }
}

impl Write for uuid::Uuid {
    #[inline(always)]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe { w.write(self.as_bytes()) }
    }

    #[inline(always)]
    fn len_s(&self) -> usize {
        16
    }
}

impl Write for &[u8] {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            w.write(self);
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
        <[u8]>::len(self)
    }
}

impl<T: Write + ?Sized> Write for NonNull<T> {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
            self.as_ref().write(w);
        }
    }

    #[inline]
    fn len_s(&self) -> usize {
        unsafe { self.as_ref().len_s() }
    }
}

impl<T: Write> Write for Option<T> {
    #[inline]
    unsafe fn write(&self, w: &mut UnsafeWriter) {
        unsafe {
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
    }

    #[inline]
    fn len_s(&self) -> usize {
        match self.as_ref() {
            Some(x) => 1 + x.len_s(),
            None => 1,
        }
    }
}
