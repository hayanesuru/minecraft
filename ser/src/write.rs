use crate::{UnsafeWriter, Write};
use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
    NonZeroU32, NonZeroU64, NonZeroU8,
};

macro_rules! primitive {
    ($type:ty) => {
        impl Write for $type {
            #[inline(always)]
            fn write(&self, w: &mut UnsafeWriter) {
                w.write(&self.to_be_bytes());
            }

            #[inline(always)]
            fn sz(&self) -> usize {
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
            fn sz(&self) -> usize {
                ::core::mem::size_of::<Self>()
            }
        }
    };
}

pub struct Write2<'a, A: ?Sized, B: ?Sized> {
    pub a: &'a A,
    pub b: &'a B,
}

impl<A: Write + ?Sized, B: Write + ?Sized> Write for Write2<'_, A, B> {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        self.a.write(w);
        self.b.write(w);
    }

    #[inline]
    fn sz(&self) -> usize {
        self.a.sz() + self.b.sz()
    }
}

pub struct Write3<'a, A: ?Sized, B: ?Sized, C: ?Sized> {
    pub a: &'a A,
    pub b: &'a B,
    pub c: &'a C,
}

impl<A: Write + ?Sized, B: Write + ?Sized, C: Write + ?Sized> Write for Write3<'_, A, B, C> {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        self.a.write(w);
        self.b.write(w);
        self.c.write(w);
    }

    #[inline]
    fn sz(&self) -> usize {
        self.a.sz() + self.b.sz() + self.c.sz()
    }
}

impl<T: Write> Write for alloc::slice::Iter<'_, T> {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        self.clone().for_each(|x| x.write(w));
    }

    #[inline(always)]
    fn sz(&self) -> usize {
        self.clone().map(|x| x.sz()).sum()
    }
}

impl<T: Write> Write for alloc::slice::IterMut<'_, T> {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        self.as_slice().iter().for_each(|x| x.write(w));
    }

    #[inline(always)]
    fn sz(&self) -> usize {
        self.as_slice().iter().map(|x| x.sz()).sum()
    }
}

impl Write for bool {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(*self as u8)
    }

    #[inline(always)]
    fn sz(&self) -> usize {
        1
    }
}

impl Write for NonZeroI8 {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(self.get() as u8)
    }

    #[inline(always)]
    fn sz(&self) -> usize {
        1
    }
}

impl Write for NonZeroU8 {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(self.get())
    }

    #[inline(always)]
    fn sz(&self) -> usize {
        1
    }
}

impl Write for u8 {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(*self)
    }

    #[inline(always)]
    fn sz(&self) -> usize {
        1
    }
}

impl Write for i8 {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write_byte(*self as u8)
    }

    #[inline(always)]
    fn sz(&self) -> usize {
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
    fn write(&self, w: &mut UnsafeWriter) {
        w.write(self.as_bytes());
    }

    #[inline(always)]
    fn sz(&self) -> usize {
        str::len(self)
    }
}

impl Write for [u8] {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write(self);
    }

    #[inline(always)]
    fn sz(&self) -> usize {
        <[u8]>::len(self)
    }
}

impl Write for &str {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write(self.as_bytes());
    }

    #[inline(always)]
    fn sz(&self) -> usize {
        str::len(self)
    }
}

impl Write for uuid::Uuid {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write(self.as_bytes())
    }

    #[inline(always)]
    fn sz(&self) -> usize {
        16
    }
}

impl Write for [&str] {
    #[inline(always)]
    fn write(&self, w: &mut UnsafeWriter) {
        for &x in self {
            crate::V21(str::len(x) as u32).write(w);
            w.write(x.as_bytes());
        }
    }

    #[inline(always)]
    fn sz(&self) -> usize {
        let mut l = 0;
        for &x in self {
            l += crate::V21(str::len(x) as u32).sz();
            l += str::len(x);
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
    fn sz(&self) -> usize {
        Write::sz(*self)
    }
}

impl Write for &[u8] {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        w.write(self);
    }

    #[inline]
    fn sz(&self) -> usize {
        <[u8]>::len(self)
    }
}
