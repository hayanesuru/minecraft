use crate::Palette;
use alloc::vec::Vec;

const BLOCK_PER_CHUNK: usize = 4 * 4 * 4;
const INDIRECT_PER_CHUNK: usize = BLOCK_PER_CHUNK / 2;

const INDEX_MASK: u64 = 0x3FFF_FFFF_FFFF_FFFF;
const SINGLE_FLAG: u64 = 0x8000_0000_0000_0000;
const INDIRECT_FLAG: u64 = 0x4000_0000_0000_0000;

#[repr(align(64))]
#[derive(Clone)]
pub struct Direct<T: Palette> {
    pub data: [T; BLOCK_PER_CHUNK],
}

#[repr(align(64))]
#[derive(Clone)]
pub struct Indirect<T: Palette> {
    pub palette: [T; 16],
    pub data: [u8; INDIRECT_PER_CHUNK],
}

#[derive(Clone)]
pub struct ChunkCache<T: Palette> {
    pub direct: Vec<Direct<T>>,
    pub indirect: Vec<Indirect<T>>,
    pub single: Vec<T>,
    pub direct_key: Vec<u32>,
    pub indirect_key: Vec<u32>,
    pub single_key: Vec<u32>,
    pub chunks: Int64Map,
}

impl<T: Palette> ChunkCache<T> {
    pub fn get(&self, x: i32, y: i32, z: i32) -> Option<T> {
        let x = ((x >> 2) & 0x3FF_FFFF) as i64;
        let y = ((y >> 2) & 0xFFF) as i64;
        let z = ((z >> 2) & 0x3FF_FFFF) as i64;
        let i = ((x << 38) | (z << 12) | y) as u64;
        let j = ((x & 3) | ((y & 3) << 2) | ((z & 3) << 4)) as usize;
        match self.chunks.get(i) {
            Some(t) => {
                let n = (t & INDEX_MASK) as usize;
                unsafe {
                    Some(if (t & SINGLE_FLAG) != 0 {
                        T::from_id(n as u32)
                    } else if (t & INDIRECT_FLAG) == 0 {
                        *self.direct.get_unchecked(n).data.get_unchecked(j)
                    } else {
                        let indirect = self.indirect.get_unchecked(n);
                        let l = *indirect.data.get_unchecked(j >> 1);
                        let index = (l >> ((j & 1) << 2)) & 0xF;
                        *indirect.palette.get_unchecked(index as usize)
                    })
                }
            }
            None => None,
        }
    }
}

fn mix(x: u64, mod_mask: usize) -> usize {
    (U64_PRIME_MAX.wrapping_mul(x) as usize) & mod_mask
}

const U64_PRIME_MAX: u64 = u64::MAX - 58;

#[derive(Clone, Copy)]
struct Kv(u64, u64);

#[derive(Clone)]
enum Slot {
    Single(u64, u64),
    Multi(Vec<Kv>),
}

#[derive(Clone)]
pub struct Int64Map {
    cache: Vec<Slot>,
    size: u32,
    mod_mask: usize,
    count: usize,
    load_factor: usize,
}

impl Int64Map {
    pub const fn new() -> Self {
        Self {
            cache: Vec::new(),
            size: 0,
            count: 0,
            mod_mask: 0,
            load_factor: 800,
        }
    }
}

impl Int64Map {
    pub fn with_capacity(capacity: usize) -> Self {
        let mut map = Self::new();
        map.reserve(capacity);
        map
    }

    pub fn set_load_factor(&mut self, load_factor: usize) {
        self.load_factor = load_factor;
        self.increase_cache_if_needed();
    }

    pub fn get_load_factor(&self) -> usize {
        self.load_factor
    }

    pub fn reserve(&mut self, additional: usize) {
        let capacity = self.count + additional;
        while self.lim() < capacity {
            self.increase_cache();
        }
    }

    pub fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        self.increase_cache_if_needed();
        let ix = mix(key, self.mod_mask);
        let vals = unsafe { self.cache.get_unchecked_mut(ix) };
        match vals {
            Slot::Single(k1, v1) => {
                if *k1 == key {
                    *v1 = value;
                    Some(*v1)
                } else {
                    self.count += 1;
                    *vals = Slot::Multi(alloc::vec![Kv(*k1, *v1), Kv(key, value)]);
                    None
                }
            }
            Slot::Multi(kvs) => {
                let pos = kvs.iter().position(|kv| kv.0 == key);
                let old = if let Some(pos) = pos {
                    Some(kvs.swap_remove(pos).1)
                } else {
                    self.count += 1;
                    None
                };
                if kvs.is_empty() {
                    *vals = Slot::Single(key, value);
                } else {
                    kvs.push(Kv(key, value));
                }
                old
            }
        }
    }

    pub fn insert_checked(&mut self, key: u64, value: u64) -> bool {
        self.increase_cache_if_needed();
        let ix = mix(key, self.mod_mask);
        let vals = unsafe { self.cache.get_unchecked_mut(ix) };
        match vals {
            Slot::Single(k1, v1) => {
                if *k1 == key {
                    *v1 = value;
                    return false;
                }
                *vals = Slot::Multi(alloc::vec![Kv(*k1, *v1), Kv(key, value)]);
                self.count += 1;
                false
            }
            Slot::Multi(kvs) => {
                if kvs.iter().any(|kv| kv.0 == key) {
                    return false;
                }
                self.count += 1;
                kvs.push(Kv(key, value));
                true
            }
        }
    }

    pub fn get(&self, key: u64) -> Option<u64> {
        if self.is_empty() {
            return None;
        }
        let ix = mix(key, self.mod_mask);
        let vals = unsafe { self.cache.get_unchecked(ix) };
        match vals {
            Slot::Single(k1, v1) => {
                if *k1 == key {
                    Some(*v1)
                } else {
                    None
                }
            }
            Slot::Multi(kvs) => {
                for &kv in kvs.iter() {
                    if kv.0 == key {
                        return Some(kv.1);
                    }
                }
                None
            }
        }
    }

    pub fn get_mut(&mut self, key: u64) -> Option<&mut u64> {
        if self.is_empty() {
            return None;
        }
        let k = key;
        let ix = mix(k, self.mod_mask);
        let vals = unsafe { self.cache.get_unchecked_mut(ix) };
        match vals {
            Slot::Single(k1, v1) => {
                if *k1 == k {
                    Some(v1)
                } else {
                    None
                }
            }
            Slot::Multi(kvs) => {
                for kv in kvs.iter_mut() {
                    if kv.0 == k {
                        return Some(&mut kv.1);
                    }
                }
                None
            }
        }
    }

    pub fn remove(&mut self, key: u64) -> Option<u64> {
        if self.is_empty() {
            return None;
        }
        let k = key;
        let ix = mix(k, self.mod_mask);
        let vals = unsafe { self.cache.get_unchecked_mut(ix) };
        match vals {
            Slot::Single(k1, v1) => {
                if *k1 == k {
                    self.count -= 1;
                    let v1 = *v1;
                    *vals = Slot::Multi(Vec::new());
                    Some(v1)
                } else {
                    None
                }
            }
            Slot::Multi(kvs) => {
                let pos = kvs.iter().position(|kv| kv.0 == k);
                if let Some(pos) = pos {
                    self.count -= 1;
                    let kv = kvs.swap_remove(pos);
                    Some(kv.1)
                } else {
                    None
                }
            }
        }
    }

    pub fn contains_key(&self, key: u64) -> bool {
        self.get(key).is_some()
    }

    pub fn clear(&mut self) {
        for vals in &mut self.cache {
            match vals {
                Slot::Single(_, _) => {
                    *vals = Slot::Multi(Vec::new());
                }
                Slot::Multi(kvs) => {
                    kvs.clear();
                }
            }
        }
        self.count = 0;
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(u64, u64) -> bool,
    {
        let mut removed = 0;
        for vals in &mut self.cache {
            match vals {
                Slot::Single(k1, v1) => {
                    if !(f)(*k1, *v1) {
                        *vals = Slot::Multi(Vec::new());
                        removed += 1;
                    }
                }
                Slot::Multi(kvs) => {
                    let len = kvs.len();
                    kvs.retain(|kv| (f)(kv.0, kv.1));
                    removed += len - kvs.len();
                }
            }
        }
        self.count -= removed;
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    // pub fn iter(&self) -> Iter<'_> {
    //     Iter::new(&self.cache)
    // }

    // pub fn iter_mut(&mut self) -> IterMut<'_> {
    //     IterMut::new(&mut self.cache)
    // }

    // pub fn keys(&self) -> Keys<'_> {
    //     Keys { inner: self.iter() }
    // }

    // pub fn values(&self) -> Values<'_> {
    //     Values { inner: self.iter() }
    // }

    // pub fn values_mut(&mut self) -> ValuesMut<'_> {
    //     ValuesMut {
    //         inner: self.iter_mut(),
    //     }
    // }

    #[inline(always)]
    fn lim(&self) -> usize {
        if self.size == 0 {
            0
        } else {
            2usize.pow(self.size)
        }
    }

    fn increase_cache(&mut self) {
        self.size += 1;
        let new_lim = self.lim();
        self.mod_mask = new_lim - 1;
        for vals in core::mem::replace(
            &mut self.cache,
            alloc::vec![Slot::Multi(Vec::new()); new_lim],
        ) {
            match vals {
                Slot::Single(k1, v1) => {
                    let ix = mix(k1, self.mod_mask);
                    let vals = unsafe { self.cache.get_unchecked_mut(ix) };
                    match vals {
                        Slot::Single(ck, cv) => {
                            *vals = Slot::Multi(alloc::vec![Kv(*ck, *cv), Kv(k1, v1)]);
                        }
                        Slot::Multi(kvs) => {
                            if kvs.is_empty() {
                                *vals = Slot::Single(k1, v1);
                            } else {
                                kvs.push(Kv(k1, v1));
                            }
                        }
                    }
                }
                Slot::Multi(kvs) => {
                    for kv in kvs {
                        let k = kv.0;
                        let ix = mix(k, self.mod_mask);
                        let vals = unsafe { self.cache.get_unchecked_mut(ix) };
                        match vals {
                            Slot::Single(k1, v1) => {
                                *vals = Slot::Multi(alloc::vec![Kv(*k1, *v1), kv]);
                            }
                            Slot::Multi(kvs) => {
                                if kvs.is_empty() {
                                    *vals = Slot::Single(kv.0, kv.1);
                                } else {
                                    kvs.push(kv);
                                }
                            }
                        }
                    }
                }
            }
        }
        debug_assert!(
            self.cache.len() == self.lim(),
            "cache vector the wrong length, lim: {:?} cache: {:?}",
            self.lim(),
            self.cache.len()
        );
    }

    #[inline]
    fn increase_cache_if_needed(&mut self) -> bool {
        let initial_cache_len = self.cache.len();
        if self.cache.is_empty() {
            self.increase_cache();
        }
        while ((self.count * 1024) / self.cache.len()) > self.load_factor {
            self.increase_cache();
        }
        initial_cache_len != self.cache.len()
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn load(&self) -> u64 {
        self.cache
            .iter()
            .filter(|vals| match vals {
                Slot::Single(_, _) => true,
                Slot::Multi(kvs) => !kvs.is_empty(),
            })
            .count() as u64
    }

    pub fn load_rate(&self) -> f64 {
        (self.count as f64) / (self.cache.len() as f64) * 100f64
    }

    pub fn capacity(&self) -> usize {
        self.cache.len()
    }

    // pub fn assert_count(&self) -> bool {
    //     let count = self.cache.iter().flatten().count();
    //     self.count == count
    // }

    // fn collisions(&self) -> IntMap {
    //     let mut map = IntMap::new();
    //     for s in self.cache.iter() {
    //         let key = s.len() as u64;
    //         if key > 1 {
    //             if !map.contains_key(key) {
    //                 map.insert(key, 1);
    //             } else {
    //                 let counter = map.get_mut(key).unwrap();
    //                 *counter += 1;
    //             }
    //         }
    //     }
    //     map
    // }

    // pub fn entry(&mut self, key: u64) -> Entry<'_> {
    //     Entry::new(key, self)
    // }
}

impl Default for Int64Map {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Int64Map {
    fn eq(&self, other: &Int64Map) -> bool {
        self.count == other.count
            && self.cache.iter().all(|vals| match vals {
                Slot::Single(k1, v1) => other.get(*k1) == Some(*v1),
                Slot::Multi(kvs) => kvs.iter().all(|kv| other.get(kv.0) == Some(kv.1)),
            })
    }
}

impl Eq for Int64Map {}

// impl core::fmt::Debug for IntMap {
//     fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
//         fmt.debug_map().entries(self.iter()).finish()
//     }
// }

// impl<'a> IntoIterator for &'a IntMap {
//     type Item = (u64, u64);
//     type IntoIter = Iter<'a>;

//     fn into_iter(self) -> Self::IntoIter {
//         Iter::new(&self.cache)
//     }
// }

// #[derive(Clone)]
// pub struct Iter<'a> {
//     inner: core::iter::Flatten<core::slice::Iter<'a, Vec<Kv>>>,
// }

// impl<'a> DoubleEndedIterator for Iter<'a> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         self.inner.next_back().map(|r| (r.0, r.1))
//     }
// }

// impl<'a> Iter<'a> {
//     fn new(vec: &'a [Vec<Kv>]) -> Self {
//         Iter {
//             inner: vec.iter().flatten(),
//         }
//     }
// }

// impl<'a> Iterator for Iter<'a> {
//     type Item = (u64, u64);

//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.inner.next().map(|r| (r.0, r.1))
//     }
// }

// impl<'a> IntoIterator for &'a mut IntMap {
//     type Item = (&'a mut u64, &'a mut u64);
//     type IntoIter = IterMut<'a>;

//     fn into_iter(self) -> Self::IntoIter {
//         IterMut::new(&mut self.cache)
//     }
// }

// pub struct IterMut<'a> {
//     inner: core::iter::Flatten<core::slice::IterMut<'a, Vec<Kv>>>,
// }

// impl<'a> IterMut<'a> {
//     fn new(vec: &'a mut [Vec<Kv>]) -> Self {
//         IterMut {
//             inner: vec.iter_mut().flatten(),
//         }
//     }
// }

// impl<'a> Iterator for IterMut<'a> {
//     type Item = (&'a mut u64, &'a mut u64);

//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.inner.next().map(|x| (&mut x.0, &mut x.1))
//     }
// }

// impl<'a> DoubleEndedIterator for IterMut<'a> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         self.inner.next_back().map(|x| (&mut x.0, &mut x.1))
//     }
// }

// pub struct Keys<'a> {
//     pub(crate) inner: Iter<'a>,
// }

// impl<'a> Iterator for Keys<'a> {
//     type Item = u64;

//     #[inline]
//     fn next(&mut self) -> Option<u64> {
//         self.inner.next().map(|kv| kv.0)
//     }

//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.inner.size_hint()
//     }
// }

// impl<'a> DoubleEndedIterator for Keys<'a> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         self.inner.next_back().map(|(k, _)| k)
//     }
// }

// pub struct Values<'a> {
//     pub(crate) inner: Iter<'a>,
// }

// impl<'a> Iterator for Values<'a> {
//     type Item = u64;

//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.inner.next().map(|(_, v)| v)
//     }

//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.inner.size_hint()
//     }
// }

// pub struct ValuesMut<'a> {
//     pub(crate) inner: IterMut<'a>,
// }

// impl<'a> Iterator for ValuesMut<'a> {
//     type Item = &'a mut u64;

//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.inner.next().map(|x| x.1)
//     }

//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.inner.size_hint()
//     }
// }

// impl IntoIterator for IntMap {
//     type Item = (u64, u64);
//     type IntoIter = IntoIter;

//     fn into_iter(self) -> Self::IntoIter {
//         IntoIter::new(self.cache)
//     }
// }

// pub struct IntoIter {
//     inner: core::iter::Flatten<alloc::vec::IntoIter<Vec<Kv>>>,
// }

// impl IntoIter {
//     fn new(vec: Vec<Vec<Kv>>) -> Self {
//         IntoIter {
//             inner: vec.into_iter().flatten(),
//         }
//     }
// }

// impl Iterator for IntoIter {
//     type Item = (u64, u64);

//     #[inline]
//     fn next(&mut self) -> Option<(u64, u64)> {
//         self.inner.next().map(|kv| (kv.0, kv.1))
//     }
// }

// impl Extend<(u64, u64)> for IntMap {
//     #[inline]
//     fn extend<T: IntoIterator<Item = (u64, u64)>>(&mut self, iter: T) {
//         for elem in iter {
//             self.insert(elem.0, elem.1);
//         }
//     }
// }

// impl core::iter::FromIterator<(u64, u64)> for IntMap {
//     #[inline]
//     fn from_iter<T: IntoIterator<Item = (u64, u64)>>(iter: T) -> Self {
//         let iterator = iter.into_iter();
//         let (lower_bound, _) = iterator.size_hint();

//         let mut map = IntMap::with_capacity(lower_bound);
//         for elem in iterator {
//             map.insert(elem.0, elem.1);
//         }

//         map
//     }
// }

// impl core::iter::FromIterator<u128> for IntMap {
//     #[inline]
//     fn from_iter<T: IntoIterator<Item = u128>>(iter: T) -> Self {
//         let iterator = iter.into_iter();
//         let (lower_bound, _) = iterator.size_hint();

//         let mut map = IntMap::with_capacity(lower_bound);
//         for elem in iterator {
//             map.insert(elem as u64, (elem >> 64) as u64);
//         }

//         map
//     }
// }
