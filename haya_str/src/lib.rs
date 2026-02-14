#![no_std]

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
    #[inline]
    fn as_mut(&mut self) -> &mut str {
        unsafe {
            core::str::from_utf8_unchecked_mut(self.data.get_unchecked_mut(0..self.len as usize))
        }
    }
}

impl core::fmt::Debug for HayaStr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_ref())
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
                ch.encode_utf8(core::slice::from_raw_parts_mut(
                    self.data.as_mut_ptr().add(len),
                    ch_len,
                ));
                self.len = core::mem::transmute::<u8, Len>(new_len as u8);
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
                core::ptr::copy_nonoverlapping(
                    s.as_ptr(),
                    self.data.as_mut_ptr().add(len),
                    s.len(),
                );
                self.len = core::mem::transmute::<u8, Len>(new_len as u8);
                Ok(())
            }
        } else {
            Err(OutOfBoundsError)
        }
    }

    pub const fn new(s: &str) -> Result<Self, OutOfBoundsError> {
        if s.len() > MAX {
            Err(OutOfBoundsError)
        } else {
            unsafe {
                let mut data = [0; MAX];
                core::ptr::copy_nonoverlapping(s.as_ptr(), data.as_mut_ptr(), s.len());
                Ok(Self {
                    len: core::mem::transmute::<u8, Len>(s.len() as u8),
                    data,
                })
            }
        }
    }
}
