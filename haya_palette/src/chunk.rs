use crate::Palette;
use alloc::vec::Vec;
use core::iter::FusedIterator;

const BLOCK_PER_CHUNK: usize = 4 * 4 * 4;
const INDIRECT4_PER_CHUNK: usize = BLOCK_PER_CHUNK / 2;
const INDIRECT2_PER_CHUNK: usize = BLOCK_PER_CHUNK / 4;
const INDEX_MASK: u64 = 0x3FFF_FFFF_FFFF_FFFF;

#[derive(Clone)]
pub struct Direct<T: Palette> {
    data: [T; BLOCK_PER_CHUNK],
}

#[derive(Clone)]
pub struct Indirect4<T: Palette> {
    palette: [T; 16],
    data: [u8; INDIRECT4_PER_CHUNK],
}

#[derive(Clone)]
pub struct Indirect2<T: Palette> {
    palette: [T; 4],
    data: [u8; INDIRECT2_PER_CHUNK],
}

#[derive(Clone)]
pub struct ChunkCache<T: Palette> {
    pub direct: Vec<Direct<T>>,
    pub indirect2: Vec<Indirect2<T>>,
    pub indirect4: Vec<Indirect4<T>>,
    pub chunks: Int64Map,
    pub direct_key: Vec<u32>,
    pub indirect4_key: Vec<u32>,
    pub indirect2_key: Vec<u32>,
    pub single_key: Vec<u32>,
}

impl<T: Palette> ChunkCache<T> {
    pub fn get(&self, x: i32, y: i32, z: i32) -> Option<T> {
        let j = ((x & 3) | ((y & 3) << 2) | ((z & 3) << 4)) as usize;
        let sx = ((x >> 2) & 0x3FF_FFFF) as i64;
        let sy = ((y >> 2) & 0xFFF) as i64;
        let sz = ((z >> 2) & 0x3FF_FFFF) as i64;
        let i = ((sx << 38) | (sz << 12) | sy) as u64;
        let t = match self.chunks.get(i) {
            Some(t) => t,
            None => {
                mser::cold_path();
                return None;
            }
        };
        let n = (t & INDEX_MASK) as usize;
        let ty = t >> 62;
        unsafe {
            Some(match ty {
                3 => T::from_id(n as u32),
                2 => self.indirect2.get_unchecked(n).get(j),
                1 => self.indirect4.get_unchecked(n).get(j),
                _ => *self.direct.get_unchecked(n).data.get_unchecked(j),
            })
        }
    }
}

impl<T: Palette> Indirect2<T> {
    unsafe fn get(&self, index: usize) -> T {
        unsafe {
            let b = *self.data.get_unchecked(index >> 2);
            let i = (b >> ((index & 3) << 1)) & 0x3;
            *self.palette.get_unchecked(i as usize)
        }
    }
}

impl<T: Palette> Indirect4<T> {
    unsafe fn get(&self, index: usize) -> T {
        unsafe {
            let b = *self.data.get_unchecked(index >> 1);
            let i = (b >> ((index & 1) << 2)) & 0xF;
            *self.palette.get_unchecked(i as usize)
        }
    }
}

fn mix(x: u64, mask: usize) -> usize {
    (U64_PRIME_MAX.wrapping_mul(x) as usize) & mask
}

const U64_PRIME_MAX: u64 = u64::MAX - 58;

#[derive(Clone, Copy)]
struct Slot(u64, u64);

impl Slot {
    const EMPTY: Self = Self(u64::MAX, u64::MAX);

    fn is_empty(self) -> bool {
        self.0 == u64::MAX
    }
}

#[derive(Clone)]
pub struct Int64Map {
    slots: Vec<Slot>,
    mask: usize,
    len: usize,
}

impl Int64Map {
    pub fn with_capacity(capacity: usize) -> Self {
        let size = capacity.next_power_of_two();
        let mask = size - 1;
        let slots = alloc::vec![Slot::EMPTY; size];
        Self {
            slots,
            len: 0,
            mask,
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let capacity = self.len + additional;
        while self.mask < capacity {
            self.rehash();
        }
    }

    pub fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        debug_assert_ne!(key, u64::MAX);
        while self.len * 4 > self.slots.len() * 3 {
            self.rehash();
        }
        let mut i = mix(key, self.mask);
        loop {
            let slot = unsafe { self.slots.get_unchecked_mut(i) };
            if slot.is_empty() {
                *slot = Slot(key, value);
                self.len += 1;
                return None;
            }
            if slot.0 == key {
                let old = slot.1;
                *slot = Slot(key, value);
                return Some(old);
            }
            i = (i + 1) & self.mask;
        }
    }

    pub fn get(&self, key: u64) -> Option<u64> {
        let mut i = mix(key, self.mask);
        loop {
            let slot = unsafe { self.slots.get_unchecked(i) };
            if slot.0 == key {
                return Some(slot.1);
            }
            if slot.is_empty() {
                return None;
            }
            i = (i + 1) & self.mask;
        }
    }

    pub fn get_mut(&mut self, key: u64) -> Option<&mut u64> {
        let mut i = mix(key, self.mask);
        loop {
            let slot = unsafe { self.slots.get_unchecked_mut(i) };
            if slot.0 == key {
                return Some(unsafe { &mut self.slots.get_unchecked_mut(i).1 });
            }
            if slot.is_empty() {
                return None;
            }
            i = (i + 1) & self.mask;
        }
    }

    pub fn remove(&mut self, key: u64) -> Option<u64> {
        let mut i = mix(key, self.mask);
        let mut slot = unsafe { self.slots.get_unchecked_mut(i) };
        loop {
            if slot.0 == key {
                break;
            }
            if slot.is_empty() {
                return None;
            }
            i = (i + 1) & self.mask;
            slot = unsafe { self.slots.get_unchecked_mut(i) };
        }
        self.len -= 1;
        let v = slot.1;
        *slot = Slot::EMPTY;
        let mut j = (i + 1) & self.mask;
        loop {
            let v = unsafe { *self.slots.get_unchecked(j) };
            if v.is_empty() {
                break;
            }
            let natural = mix(v.0, self.mask);
            let remove = if natural <= i {
                i < j || j < natural
            } else {
                j < natural && i < j
            };
            if remove {
                unsafe {
                    *self.slots.get_unchecked_mut(i) = v;
                    *self.slots.get_unchecked_mut(j) = Slot::EMPTY;
                }
                i = j;
            }
            j = (j + 1) & self.mask;
        }
        Some(v)
    }

    pub fn contains_key(&self, key: u64) -> bool {
        self.get(key).is_some()
    }

    pub fn clear(&mut self) {
        self.slots.fill(Slot::EMPTY);
        self.len = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter::new(&self.slots)
    }

    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut::new(&mut self.slots)
    }

    pub fn keys(&self) -> Keys<'_> {
        Keys { inner: self.iter() }
    }

    pub fn values(&self) -> Values<'_> {
        Values { inner: self.iter() }
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_> {
        ValuesMut {
            inner: self.iter_mut(),
        }
    }

    #[cold]
    fn rehash(&mut self) {
        let new_size = (self.mask + 1) << 1;
        self.mask = new_size - 1;
        let old = core::mem::replace(&mut self.slots, alloc::vec![Slot::EMPTY; new_size]);
        for slot in old {
            if slot.is_empty() {
                continue;
            }
            let mut i = mix(slot.0, self.mask);
            loop {
                let j = unsafe { self.slots.get_unchecked_mut(i) };
                if j.is_empty() {
                    *j = slot;
                    break;
                }
                i = (i + 1) & self.mask;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
}

impl Default for Int64Map {
    fn default() -> Self {
        Self::with_capacity(8)
    }
}

impl PartialEq for Int64Map {
    fn eq(&self, other: &Int64Map) -> bool {
        self.len == other.len
            && self
                .slots
                .iter()
                .filter(|x| !x.is_empty())
                .all(|slot| other.get(slot.0) == Some(slot.1))
    }
}

impl Eq for Int64Map {}

impl core::fmt::Debug for Int64Map {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.debug_map().entries(self.iter()).finish()
    }
}

#[derive(Clone)]
pub struct Iter<'a> {
    inner: core::slice::Iter<'a, Slot>,
}

impl<'a> Iter<'a> {
    fn new(vec: &'a [Slot]) -> Self {
        Iter { inner: vec.iter() }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (u64, u64);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find(|&x| !x.is_empty()).map(|r| (r.0, r.1))
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.rfind(|&x| !x.is_empty()).map(|r| (r.0, r.1))
    }
}

impl<'a> FusedIterator for Iter<'a> {}

impl<'a> IntoIterator for &'a mut Int64Map {
    type Item = (&'a mut u64, &'a mut u64);
    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut::new(&mut self.slots)
    }
}

pub struct IterMut<'a> {
    inner: core::slice::IterMut<'a, Slot>,
}

impl<'a> IterMut<'a> {
    fn new(vec: &'a mut [Slot]) -> Self {
        IterMut {
            inner: vec.iter_mut(),
        }
    }
}

impl<'a> Iterator for IterMut<'a> {
    type Item = (&'a mut u64, &'a mut u64);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .find(|x| !x.is_empty())
            .map(|x| (&mut x.0, &mut x.1))
    }
}

impl<'a> DoubleEndedIterator for IterMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .rfind(|x| !x.is_empty())
            .map(|x| (&mut x.0, &mut x.1))
    }
}

impl<'a> FusedIterator for IterMut<'a> {}

pub struct Keys<'a> {
    inner: Iter<'a>,
}

impl<'a> Iterator for Keys<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        self.inner.next().map(|kv| kv.0)
    }
}

impl<'a> DoubleEndedIterator for Keys<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(k, _)| k)
    }
}

pub struct Values<'a> {
    inner: Iter<'a>,
}

impl<'a> Iterator for Values<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, v)| v)
    }
}

pub struct ValuesMut<'a> {
    pub(crate) inner: IterMut<'a>,
}

impl<'a> Iterator for ValuesMut<'a> {
    type Item = &'a mut u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| x.1)
    }
}

impl IntoIterator for Int64Map {
    type Item = (u64, u64);
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.slots)
    }
}

pub struct IntoIter {
    inner: alloc::vec::IntoIter<Slot>,
}

impl IntoIter {
    fn new(vec: Vec<Slot>) -> Self {
        IntoIter {
            inner: vec.into_iter(),
        }
    }
}

impl Iterator for IntoIter {
    type Item = (u64, u64);

    fn next(&mut self) -> Option<(u64, u64)> {
        self.inner.find(|x| !x.is_empty()).map(|kv| (kv.0, kv.1))
    }
}

impl<'a> FusedIterator for Keys<'a> {}

impl<'a> FusedIterator for Values<'a> {}

impl<'a> FusedIterator for ValuesMut<'a> {}

impl FusedIterator for IntoIter {}

impl Extend<(u64, u64)> for Int64Map {
    #[inline]
    fn extend<T: IntoIterator<Item = (u64, u64)>>(&mut self, iter: T) {
        for elem in iter {
            self.insert(elem.0, elem.1);
        }
    }
}

impl core::iter::FromIterator<(u64, u64)> for Int64Map {
    fn from_iter<T: IntoIterator<Item = (u64, u64)>>(iter: T) -> Self {
        let iterator = iter.into_iter();
        let (lower_bound, _) = iterator.size_hint();
        let mut map = Int64Map::with_capacity(lower_bound);
        for elem in iterator {
            map.insert(elem.0, elem.1);
        }
        map
    }
}
