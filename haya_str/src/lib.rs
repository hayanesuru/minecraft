#![no_std]

use core::mem::transmute;
use core::ptr::copy_nonoverlapping;

const MAX: usize = 31;

#[derive(Clone, Copy)]
pub struct HayaStr {
    len: Len,
    data: [u8; MAX],
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Len {
    N0,
    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
    N10,
    N11,
    N12,
    N13,
    N14,
    N15,
    N16,
    N17,
    N18,
    N19,
    N20,
    N21,
    N22,
    N23,
    N24,
    N25,
    N26,
    N27,
    N28,
    N29,
    N30,
    N31,
}

impl AsRef<str> for HayaStr {
    #[inline]
    fn as_ref(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.data.get_unchecked(0..self.len as usize)) }
    }
}

impl AsMut<str> for HayaStr {
    /// Provide a mutable `&str` view of the currently initialized bytes.
    ///
    /// This returns a mutable string slice that borrows the portion of the internal
    /// buffer indicated by the stored length. The bytes are interpreted as UTF‑8
    /// without additional validation (code assumes the stored bytes are valid UTF‑8).
    ///
    /// # Examples
    ///
    /// ```
    /// let mut s = HayaStr::new("hi").unwrap();
    /// let slice = s.as_mut();
    /// slice.make_ascii_uppercase();
    /// assert_eq!(&*s, "HI");
    /// ```
    #[inline]
    fn as_mut(&mut self) -> &mut str {
        unsafe {
            core::str::from_utf8_unchecked_mut(self.data.get_unchecked_mut(0..self.len as usize))
        }
    }
}

impl core::ops::Deref for HayaStr {
    type Target = str;

    /// Dereferences this `HayaStr` to a string slice view of its contents.
    ///
    /// Returns a `&str` that borrows the stored UTF-8 bytes for the current length.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut s = HayaStr::new("hi").unwrap();
    /// let slice: &str = s.deref();
    /// assert_eq!(slice, "hi");
    /// ```
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl core::ops::DerefMut for HayaStr {
    /// Mutably dereferences the HayaStr to its underlying `str`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut s = HayaStr::new("ab").unwrap();
    /// let slice: &mut str = s.deref_mut();
    /// slice.make_ascii_uppercase();
    /// assert_eq!(&*s, "AB");
    /// ```
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl core::fmt::Debug for HayaStr {
    /// Formats the value using the underlying string's debug representation.
    ///
    /// # Examples
    ///
    /// ```
    /// let s = HayaStr::new("hi").unwrap();
    /// assert_eq!(format!("{:?}", s), "\"hi\"");
    /// ```
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.as_ref())
    }
}

impl core::fmt::Display for HayaStr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl PartialEq for HayaStr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }
}

impl PartialOrd for HayaStr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HayaStr {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl Eq for HayaStr {}

impl core::hash::Hash for HayaStr {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl Default for HayaStr {
    fn default() -> Self {
        Self {
            len: Len::N0,
            data: [0; MAX],
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OutOfBoundsError;

impl HayaStr {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        matches!(self.len, Len::N0)
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    pub const fn try_push(&mut self, ch: char) -> Result<(), OutOfBoundsError> {
        let len = self.len as usize;
        let ch_len = ch.len_utf8();
        let new_len = ch_len + len;
        if new_len <= MAX {
            unsafe {
                let ptr = self.data.as_mut_ptr().add(len);
                ch.encode_utf8(core::slice::from_raw_parts_mut(ptr, ch_len));
                self.len = transmute::<u8, Len>(new_len as u8);
            }
            Ok(())
        } else {
            Err(OutOfBoundsError)
        }
    }

    pub const fn try_extend(&mut self, s: &str) -> Result<(), OutOfBoundsError> {
        let len = self.len as usize;
        let new_len = len + s.len();
        if new_len <= MAX {
            unsafe {
                let ptr = self.data.as_mut_ptr().add(len);
                copy_nonoverlapping(s.as_ptr(), ptr, s.len());
                self.len = transmute::<u8, Len>(new_len as u8);
                Ok(())
            }
        } else {
            Err(OutOfBoundsError)
        }
    }

    pub const fn clear(&mut self) {
        self.len = Len::N0;
    }

    pub const fn new(s: &str) -> Result<Self, OutOfBoundsError> {
        if s.len() > MAX {
            Err(OutOfBoundsError)
        } else {
            unsafe {
                let mut data = [0; MAX];
                copy_nonoverlapping(s.as_ptr(), data.as_mut_ptr(), s.len());
                Ok(Self {
                    len: transmute::<u8, Len>(s.len() as u8),
                    data,
                })
            }
        }
    }
}