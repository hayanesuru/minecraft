use crate::{CompoundTag, StringTag, Tag, read_tag};
use alloc::vec::Vec;
use mser::{Error, Read, Reader, Write, Writer};

impl AsRef<[(StringTag, Tag)]> for CompoundTag {
    #[inline]
    fn as_ref(&self) -> &[(StringTag, Tag)] {
        self.0.as_slice()
    }
}

impl AsMut<[(StringTag, Tag)]> for CompoundTag {
    #[inline]
    fn as_mut(&mut self) -> &mut [(StringTag, Tag)] {
        self.0.as_mut_slice()
    }
}

impl Write for CompoundTag {
    unsafe fn write(&self, w: &mut Writer) {
        crate::write_tag(w, crate::WriteEntry::Compound(&self.0));
    }

    fn len_s(&self) -> usize {
        crate::len_tag(crate::WriteEntry::Compound(&self.0))
    }
}

impl<K: Into<StringTag>, V> FromIterator<(K, V)> for CompoundTag
where
    Tag: From<V>,
{
    #[inline]
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|(x, y)| (x.into(), Tag::from(y)))
                .collect(),
        )
    }
}

impl Default for CompoundTag {
    fn default() -> Self {
        Self::new()
    }
}

impl CompoundTag {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, (StringTag, Tag)> {
        self.0.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, (StringTag, Tag)> {
        self.0.iter_mut()
    }

    #[inline]
    pub fn sort(&mut self) {
        self.0.sort_unstable_by(|x, y| x.0.cmp(&y.0));
    }

    #[inline]
    #[must_use]
    pub fn get(&self, index: usize) -> Option<(&str, &Tag)> {
        #[allow(clippy::manual_map)]
        match self.0.get(index) {
            Some((x, y)) => Some((x, y)),
            None => None,
        }
    }

    #[inline]
    #[must_use]
    /// # Safety
    ///
    /// `index` < [`len`]
    ///
    /// [`len`]: Self::len
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> (&str, &mut Tag) {
        let (x, y) = unsafe { self.0.get_unchecked_mut(index) };
        (&**x, y)
    }

    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn push(&mut self, k: StringTag, v: Tag) {
        self.0.push((k, v));
    }

    #[inline]
    pub fn find(&self, name: &str) -> Option<&Tag> {
        for (x, y) in &self.0 {
            let x = &**x;
            if x == name {
                return Some(y);
            }
        }
        None
    }

    #[inline]
    pub fn find_remove(&mut self, name: &str) -> Option<Tag> {
        for (i, (x, _)) in self.0.iter_mut().enumerate() {
            let x = &**x;
            if x == name {
                return Some(self.0.swap_remove(i).1);
            }
        }
        None
    }

    #[inline]
    pub fn find_mut(&mut self, name: &str) -> Option<&mut Tag> {
        for (x, y) in &mut self.0 {
            let x = &**x;
            if x == name {
                return Some(y);
            }
        }
        None
    }

    #[inline]
    pub fn into_inner(self) -> Vec<(StringTag, Tag)> {
        self.0
    }
}

impl Read<'_> for CompoundTag {
    #[inline]
    fn read(buf: &mut Reader) -> Result<Self, Error> {
        match read_tag(buf, crate::ReadEntry::Compound(Self::new()), 512) {
            Ok(Tag::Compound(x)) => Ok(x),
            Ok(_) => Err(Error),
            Err(e) => Err(e),
        }
    }
}

impl From<Vec<(StringTag, Tag)>> for CompoundTag {
    #[inline]
    fn from(value: Vec<(StringTag, Tag)>) -> Self {
        Self(value)
    }
}

impl IntoIterator for CompoundTag {
    type IntoIter = alloc::vec::IntoIter<(StringTag, Tag)>;
    type Item = (StringTag, Tag);

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
