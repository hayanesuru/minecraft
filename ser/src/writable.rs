use super::writer::UnsafeWriter;
use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
    NonZeroU32, NonZeroU64, NonZeroU8,
};

#[allow(clippy::len_without_is_empty)]
pub trait Write {
    fn write(&self, w: &mut UnsafeWriter);

    fn len(&self) -> usize;
}

macro_rules! primitive {
    ($type:ty) => {
        impl Write for $type {
            #[inline(always)]
            fn write(&self, w: &mut UnsafeWriter) {
                w.write(&self.to_be_bytes());
            }

            #[inline(always)]
            fn len(&self) -> usize {
                ::core::mem::size_of::<Self>()
            }
        }
    };
}

macro_rules! non_zero {
    ($type:ty) => {
        impl Write for $type {
            #[inline(always)]
            fn write(&self, w: &mut UnsafeWriter) {
                w.write(&self.get().to_be_bytes());
            }

            #[inline(always)]
            fn len(&self) -> usize {
                ::core::mem::size_of::<Self>()
            }
        }
    };
}

impl Write for bool {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(*self as u8)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        1
    }
}

impl Write for NonZeroI8 {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(self.get() as u8)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        1
    }
}

impl Write for NonZeroU8 {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(self.get())
    }

    #[inline(always)]
    fn len(&self) -> usize {
        1
    }
}

impl Write for u8 {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(*self)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        1
    }
}

impl Write for i8 {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(*self as u8)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        1
    }
}

primitive!(i16);
primitive!(i32);
primitive!(i64);
primitive!(i128);
primitive!(u16);
primitive!(u32);
primitive!(u64);
primitive!(u128);
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
    fn write(&self, w: &mut UnsafeWriter) {
        w.write(self.as_bytes());
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }
}

impl Write for [u8] {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write(self);
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }
}

impl Write for uuid::Uuid {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write(self.as_bytes())
    }

    #[inline(always)]
    fn len(&self) -> usize {
        16
    }
}

impl Write for [&str] {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        for &x in self {
            crate::V21(x.len() as u32).write(w);
            w.write(x.as_bytes());
        }
    }

    #[inline(always)]
    fn len(&self) -> usize {
        let mut l = 0;
        for &x in self {
            l += crate::V21(x.len() as u32).len();
            l += x.len();
        }
        l
    }
}

impl Write for &[&str] {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        Write::write(*self, w);
    }

    #[inline]
    fn len(&self) -> usize {
        Write::len(*self)
    }
}
