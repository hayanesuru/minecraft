use alloc::alloc::{Allocator, Global};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::convert::Infallible;
use core::mem::MaybeUninit;
use core::str::FromStr;
use core::{fmt, hash, iter, mem, ops};

#[repr(transparent)]
pub struct SmolStr<A: Allocator = Global>(Repr<A>);

impl SmolStr {
    pub const EMPTY: Self = Self::new_inline("");

    /// Constructs an inline variant of `SmolStr`.
    ///
    /// This never allocates.
    ///
    /// # Panics
    ///
    /// Panics if `text.len() > 23`.
    #[inline]
    pub const fn new_inline(text: &str) -> Self {
        assert!(text.len() <= INLINE_CAP); // avoids bounds checks in loop

        let text = text.as_bytes();
        let mut buf = [0; INLINE_CAP];
        let mut i = 0;
        while i < text.len() {
            buf[i] = text[i];
            i += 1
        }
        Self(Repr::Inline {
            // SAFETY: We know that `len` is less than or equal to the maximum value of `InlineSize`
            // as we asserted it.
            len: unsafe { InlineSize::transmute_from_u8(text.len() as u8) },
            buf,
        })
    }

    /// # Safety
    pub const unsafe fn new_inline_unchecked(buf: [u8; INLINE_CAP], len: usize) -> Self {
        Self(Repr::Inline {
            len: unsafe { InlineSize::transmute_from_u8(len as u8) },
            buf,
        })
    }

    /// # Safety
    pub const unsafe fn new_heap_unchecked(buf: Box<[u8]>) -> Self {
        Self(Repr::Heap(buf))
    }

    /// Constructs a `SmolStr` from a statically allocated string.
    ///
    /// This never allocates.
    #[inline(always)]
    pub const fn new_static(text: &'static str) -> SmolStr {
        // NOTE: this never uses the inline storage; if a canonical
        // representation is needed, we could check for `len() < INLINE_CAP`
        // and call `new_inline`, but this would mean an extra branch.
        SmolStr(Repr::Static(text))
    }

    /// Constructs a `SmolStr` from a `str`, heap-allocating if necessary.
    #[inline(always)]
    pub fn new(text: impl AsRef<str>) -> SmolStr {
        SmolStr(Repr::new(text.as_ref()))
    }

    /// Returns the length of `self` in bytes.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if `self` has a length of zero bytes.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<A: Allocator> SmolStr<A> {
    /// Returns `true` if `self` is heap-allocated.
    #[inline(always)]
    pub const fn is_heap_allocated(&self) -> bool {
        matches!(self.0, Repr::Heap(..))
    }

    /// Returns a `&str` slice of this `SmolStr`.
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<A: Allocator> Clone for SmolStr<A>
where
    A: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        if !self.is_heap_allocated() {
            // SAFETY: We verified that the payload of `Repr` is a POD
            return unsafe { core::ptr::read(self as *const SmolStr<A>) };
        }
        Self(self.0.clone())
    }
}

impl Default for SmolStr {
    #[inline(always)]
    fn default() -> SmolStr {
        SmolStr(Repr::Inline {
            len: InlineSize::_V0,
            buf: [0; INLINE_CAP],
        })
    }
}

impl<A: Allocator> ops::Deref for SmolStr<A> {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

// region: PartialEq implementations

impl<A: Allocator> Eq for SmolStr<A> {}

impl<A: Allocator> PartialEq<SmolStr<A>> for SmolStr<A> {
    fn eq(&self, other: &SmolStr<A>) -> bool {
        self.0.ptr_eq(&other.0) || self.as_str() == other.as_str()
    }
}

impl<A: Allocator> PartialEq<str> for SmolStr<A> {
    #[inline(always)]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<A: Allocator> PartialEq<SmolStr<A>> for str {
    #[inline(always)]
    fn eq(&self, other: &SmolStr<A>) -> bool {
        other == self
    }
}

impl<'a, A: Allocator> PartialEq<&'a str> for SmolStr<A> {
    #[inline(always)]
    fn eq(&self, other: &&'a str) -> bool {
        self == *other
    }
}

impl<A: Allocator> PartialEq<SmolStr<A>> for &str {
    #[inline(always)]
    fn eq(&self, other: &SmolStr<A>) -> bool {
        *self == other
    }
}

impl<A: Allocator> Ord for SmolStr<A> {
    fn cmp(&self, other: &SmolStr<A>) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<A: Allocator> PartialOrd for SmolStr<A> {
    fn partial_cmp(&self, other: &SmolStr<A>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: Allocator> hash::Hash for SmolStr<A> {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.as_str().hash(hasher);
    }
}

impl<A: Allocator> fmt::Debug for SmolStr<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<A: Allocator> fmt::Display for SmolStr<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl iter::FromIterator<char> for SmolStr {
    fn from_iter<I: iter::IntoIterator<Item = char>>(iter: I) -> SmolStr {
        from_char_iter(iter.into_iter())
    }
}

fn from_char_iter(mut iter: impl Iterator<Item = char>) -> SmolStr {
    let (min_size, _) = iter.size_hint();
    if min_size > INLINE_CAP {
        let mut heap = Vec::with_capacity(min_size);
        for ch in iter {
            let len = ch.len_utf8();
            if len == 1 {
                heap.push(ch as u8);
            } else {
                heap.reserve(len);
                unsafe {
                    ch.encode_utf8(
                        &mut *(heap.spare_capacity_mut() as *mut [MaybeUninit<u8>] as *mut [u8]),
                    );
                    heap.set_len(heap.len() + len);
                }
            }
        }
        if heap.len() <= INLINE_CAP {
            // size hint lied
            return SmolStr::new_inline(unsafe { core::str::from_utf8_unchecked(&heap) });
        }
        return SmolStr(Repr::Heap(heap.into_boxed_slice()));
    }
    let mut len = 0;
    let mut buf = [0u8; INLINE_CAP];
    while let Some(ch) = iter.next() {
        let size = ch.len_utf8();
        if size + len > INLINE_CAP {
            let (min_remaining, _) = iter.size_hint();
            let mut heap = Vec::with_capacity(size + len + min_remaining);
            heap.extend(unsafe { buf.get_unchecked(..len) });
            {
                let len = ch.len_utf8();
                if len == 1 {
                    heap.push(ch as u8);
                } else {
                    heap.reserve(len);
                    unsafe {
                        ch.encode_utf8(
                            &mut *(heap.spare_capacity_mut() as *mut [MaybeUninit<u8>]
                                as *mut [u8]),
                        );
                        heap.set_len(heap.len() + len);
                    }
                }
            }
            for ch in iter {
                let len = ch.len_utf8();
                if len == 1 {
                    heap.push(ch as u8);
                } else {
                    heap.reserve(len);
                    unsafe {
                        ch.encode_utf8(
                            &mut *(heap.spare_capacity_mut() as *mut [MaybeUninit<u8>]
                                as *mut [u8]),
                        );
                        heap.set_len(heap.len() + len);
                    }
                }
            }
            return SmolStr(Repr::Heap(heap.into_boxed_slice()));
        }
        ch.encode_utf8(unsafe { buf.get_unchecked_mut(len..) });
        len += size;
    }
    SmolStr(Repr::Inline {
        // SAFETY: We know that `len` is less than or equal to the maximum value of `InlineSize`
        // as we otherwise return early.
        len: unsafe { InlineSize::transmute_from_u8(len as u8) },
        buf,
    })
}

impl<A: Allocator> AsRef<str> for SmolStr<A> {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<A: Allocator> AsRef<[u8]> for SmolStr<A> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl FromStr for SmolStr {
    type Err = Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<SmolStr, Self::Err> {
        Ok(SmolStr::new(s))
    }
}

pub const INLINE_CAP: usize = InlineSize::_V23 as usize;

/// A [`u8`] with a bunch of niches.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
enum InlineSize {
    _V0 = 0,
    _V1,
    _V2,
    _V3,
    _V4,
    _V5,
    _V6,
    _V7,
    _V8,
    _V9,
    _V10,
    _V11,
    _V12,
    _V13,
    _V14,
    _V15,
    _V16,
    _V17,
    _V18,
    _V19,
    _V20,
    _V21,
    _V22,
    _V23,
}

impl InlineSize {
    /// SAFETY: `value` must be less than or equal to [`INLINE_CAP`]
    #[inline(always)]
    const unsafe fn transmute_from_u8(value: u8) -> Self {
        debug_assert!(value <= InlineSize::_V23 as u8);
        // SAFETY: The caller is responsible to uphold this invariant
        unsafe { mem::transmute::<u8, Self>(value) }
    }
}

#[derive(Clone, Debug)]
enum Repr<A: Allocator = Global> {
    Inline {
        len: InlineSize,
        buf: [u8; INLINE_CAP],
    },
    Static(&'static str),
    Heap(Box<[u8], A>),
}

impl Repr {
    /// This function tries to create a new Repr::Inline or Repr::Static
    /// If it isn't possible, this function returns None
    fn new_on_stack<T>(text: T) -> Option<Self>
    where
        T: AsRef<str>,
    {
        fn inner(text: &str) -> Option<Repr> {
            let len = text.len();
            if len <= INLINE_CAP {
                let mut buf = [0; INLINE_CAP];
                unsafe {
                    buf.get_unchecked_mut(0..len)
                        .copy_from_slice(text.as_bytes());
                }
                return Some(Repr::Inline {
                    // SAFETY: We know that `len` is less than or equal to the maximum value of `InlineSize`
                    len: unsafe { InlineSize::transmute_from_u8(len as u8) },
                    buf,
                });
            }
            None
        }
        inner(text.as_ref())
    }

    fn new(text: &str) -> Self {
        match Self::new_on_stack(text) {
            Some(x) => x,
            None => Self::Heap(Box::from(text.as_bytes())),
        }
    }

    #[inline(always)]
    fn len(&self) -> usize {
        match self {
            Repr::Heap(data) => data.len(),
            Repr::Static(data) => data.len(),
            Repr::Inline { len, .. } => *len as usize,
        }
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        match self {
            Repr::Heap(data) => data.is_empty(),
            Repr::Static(data) => data.is_empty(),
            &Repr::Inline { len, .. } => len as u8 == 0,
        }
    }
}

impl<A: Allocator> Repr<A> {
    fn ptr_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Heap(l0), Self::Heap(r0)) => core::ptr::eq(l0.as_ptr(), r0.as_ptr()),
            (Self::Static(l0), Self::Static(r0)) => core::ptr::eq(l0, r0),
            (
                Self::Inline {
                    len: l_len,
                    buf: l_buf,
                },
                Self::Inline {
                    len: r_len,
                    buf: r_buf,
                },
            ) => l_len == r_len && l_buf == r_buf,
            _ => false,
        }
    }
    #[inline]
    fn as_str(&self) -> &str {
        match self {
            Repr::Heap(data) => unsafe { core::str::from_utf8_unchecked(data) },
            Repr::Static(data) => data,
            Repr::Inline { len, buf } => {
                let len = *len as usize;
                // SAFETY: len is guaranteed to be <= INLINE_CAP
                let buf = unsafe { buf.get_unchecked(..len) };
                // SAFETY: buf is guaranteed to be valid utf8 for ..len bytes
                unsafe { ::core::str::from_utf8_unchecked(buf) }
            }
        }
    }
}
/// Convert value to [`SmolStr`] using [`fmt::Display`], potentially without allocating.
///
/// Almost identical to [`ToString`], but converts to `SmolStr` instead.
pub trait ToString1 {
    fn to_string1(&self) -> SmolStr;
}

impl<T> ToString1 for T
where
    T: fmt::Display + ?Sized,
{
    fn to_string1(&self) -> SmolStr {
        let mut w = StringBuilder::new();
        fmt::Write::write_fmt(&mut w, format_args!("{}", self))
            .expect("a formatting trait implementation returned an error");
        w.finish()
    }
}

/// Formats arguments to a [`SmolStr`], potentially without allocating.
///
/// See [`alloc::format!`] or [`format_args!`] for syntax documentation.
#[macro_export]
macro_rules! format {
    ($($tt:tt)*) => {{
        let mut w = $crate::str::StringBuilder::new();
        ::core::fmt::Write::write_fmt(&mut w, format_args!($($tt)*)).expect("a formatting trait implementation returned an error");
        w.finish()
    }};
}

/// A builder that can be used to efficiently build a [`SmolStr`].
///
/// This won't allocate if the final string fits into the inline buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringBuilder<A: Allocator = Global>(StringBuilderRepr<A>);

#[derive(Clone, Debug, PartialEq, Eq)]
enum StringBuilderRepr<A: Allocator = Global> {
    Inline { len: usize, buf: [u8; INLINE_CAP] },
    Heap(Vec<u8, A>),
}

impl Default for StringBuilderRepr {
    #[inline]
    fn default() -> Self {
        StringBuilderRepr::Inline {
            buf: [0; INLINE_CAP],
            len: 0,
        }
    }
}

impl Default for StringBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl StringBuilder {
    /// Creates a new empty [`StringBuilder`].
    #[must_use]
    pub const fn new() -> Self {
        Self(StringBuilderRepr::Inline {
            buf: [0; INLINE_CAP],
            len: 0,
        })
    }

    /// Builds a [`SmolStr`] from `self`.
    #[must_use]
    pub fn finish(&self) -> SmolStr {
        SmolStr(match &self.0 {
            &StringBuilderRepr::Inline { len, buf } => {
                debug_assert!(len <= INLINE_CAP);
                Repr::Inline {
                    // SAFETY: We know that `value.len` is less than or equal to the maximum value of `InlineSize`
                    len: unsafe { InlineSize::transmute_from_u8(len as u8) },
                    buf,
                }
            }
            StringBuilderRepr::Heap(heap) => unsafe {
                Repr::new(core::str::from_utf8_unchecked(heap))
            },
        })
    }

    /// Appends the given [`char`] to the end of `self`'s buffer.
    pub fn push_char(&mut self, c: char) {
        match &mut self.0 {
            StringBuilderRepr::Inline { len, buf } => {
                let char_len = c.len_utf8();
                let new_len = *len + char_len;
                if new_len <= INLINE_CAP {
                    unsafe { c.encode_utf8(buf.get_unchecked_mut(*len..)) };
                    *len += char_len;
                } else {
                    let mut heap = Vec::with_capacity(new_len);
                    // copy existing inline bytes over to the heap
                    // SAFETY: inline data is guaranteed to be valid utf8 for `old_len` bytes
                    unsafe { heap.extend(buf.get_unchecked(..*len)) };
                    unsafe {
                        c.encode_utf8(
                            &mut *(heap.spare_capacity_mut() as *mut [MaybeUninit<u8>]
                                as *mut [u8]),
                        );
                        heap.set_len(new_len);
                    }
                    self.0 = StringBuilderRepr::Heap(heap);
                }
            }
            StringBuilderRepr::Heap(h) => {
                let mut dst = [0; 4];
                let s = c.encode_utf8(&mut dst);
                h.extend(s.as_bytes());
            }
        }
    }

    pub fn push2(&mut self, c: u8) {
        match &mut self.0 {
            StringBuilderRepr::Inline { len, buf } => {
                let new_len = *len + 1;
                if new_len <= INLINE_CAP {
                    unsafe {
                        *buf.get_unchecked_mut(*len) = c;
                    }
                    *len += 1;
                } else {
                    let mut heap = Vec::with_capacity(new_len);
                    // copy existing inline bytes over to the heap
                    // SAFETY: inline data is guaranteed to be valid utf8 for `old_len` bytes
                    unsafe { heap.extend(buf.get_unchecked(..*len)) };
                    heap.push(c);
                    self.0 = StringBuilderRepr::Heap(heap);
                }
            }
            StringBuilderRepr::Heap(h) => {
                h.push(c);
            }
        }
    }

    /// Appends a given string slice onto the end of `self`'s buffer.
    pub fn extend(&mut self, s: &[u8]) {
        match &mut self.0 {
            StringBuilderRepr::Inline { len, buf } => {
                let old_len = *len;
                *len += s.len();

                // if the new length will fit on the stack (even if it fills it entirely)
                if *len <= INLINE_CAP {
                    unsafe { buf.get_unchecked_mut(old_len..*len).copy_from_slice(s) }
                    return; // skip the heap push below
                }

                let mut heap = Vec::with_capacity(*len);

                // copy existing inline bytes over to the heap
                // SAFETY: inline data is guaranteed to be valid utf8 for `old_len` bytes
                unsafe { heap.extend_from_slice(buf.get_unchecked(..old_len)) };
                heap.extend(s);
                self.0 = StringBuilderRepr::Heap(heap);
            }
            StringBuilderRepr::Heap(heap) => heap.extend(s),
        }
    }
}

impl fmt::Write for StringBuilder {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.extend(s.as_bytes());
        Ok(())
    }
}

impl From<StringBuilder> for SmolStr {
    fn from(value: StringBuilder) -> Self {
        value.finish()
    }
}

impl From<Box<str>> for SmolStr {
    #[inline]
    fn from(value: Box<str>) -> Self {
        Self(match Repr::new_on_stack(&*value) {
            Some(x) => x,
            None => Repr::Heap(unsafe { Box::from_raw(Box::into_raw(value) as *mut [u8]) }),
        })
    }
}
