#![no_std]
#![allow(non_camel_case_types, clippy::new_without_default, clippy::manual_map)]

#[cfg(feature = "1_16")]
pub mod v1_16 {
    include!(concat!(env!("OUT_DIR"), "/1.16.5.rs"));
    include!("shared.rs");
}
#[cfg(feature = "1_17")]
pub mod v1_17 {
    include!(concat!(env!("OUT_DIR"), "/1.17.1.rs"));
    include!("shared.rs");
}
#[cfg(feature = "1_18")]
pub mod v1_18 {
    include!(concat!(env!("OUT_DIR"), "/1.18.2.rs"));
    include!("shared.rs");
}
#[cfg(feature = "1_19")]
pub mod v1_19 {
    include!(concat!(env!("OUT_DIR"), "/1.19.4.rs"));
    include!("shared.rs");
}
#[cfg(feature = "1_20")]
pub mod v1_20 {
    include!(concat!(env!("OUT_DIR"), "/1.20.6.rs"));
    include!("shared.rs");
}

#[macro_export]
macro_rules! encode_state {
    ($b:ident($x:expr)) => {
        block_state::new($x.encode() as raw_block_state + block::$b.state_index())
    };
}

#[macro_export]
macro_rules! decode_state {
    ($b:ident($x:expr)) => {
        $b::decode((($x.id() - block::$b.state_index()) as _))
    };
}

#[cold]
#[inline(always)]
const fn cold__() {}

#[derive(Copy, Clone)]
struct NameMap<T: 'static> {
    key: [u64; 4],
    disps: &'static [(u32, u32)],
    names: *const u8,
    vals: &'static [T],
}

fn hash(key: [u64; 4], name: &[u8], disps: &'static [(u32, u32)], len: u32) -> u32 {
    let hasher = highway::HighwayHasher::new(highway::Key(key));
    let [a, b] = highway::HighwayHash::hash128(hasher, name);
    let g = (a >> 32) as u32;
    let f1 = a as u32;
    let f2 = b as u32;
    let (d1, d2) = disps[(g % (disps.len() as u32)) as usize];
    d2.wrapping_add(f1.wrapping_mul(d1)).wrapping_add(f2) % len
}

impl NameMap<u16> {
    fn get(&self, name: &[u8]) -> Option<u16> {
        let index = hash(self.key, name, self.disps, self.vals.len() as u32);
        let v = unsafe { *self.vals.get_unchecked(index as usize) };
        let offset =
            unsafe { u32::from_ne_bytes(*self.names.add(4 * v as usize).cast::<[u8; 4]>()) };
        let len = unsafe {
            u16::from_ne_bytes(*self.names.add(offset as usize).cast::<[u8; 2]>()) as usize
        };
        let k = unsafe { core::slice::from_raw_parts(self.names.add(offset as usize + 2), len) };
        if name == k {
            Some(v)
        } else {
            None
        }
    }
}

impl NameMap<u8> {
    fn get(&self, name: &[u8]) -> Option<u8> {
        let index = hash(self.key, name, self.disps, self.vals.len() as u32);
        let v = unsafe { *self.vals.get_unchecked(index as usize) };
        let offset =
            unsafe { u32::from_ne_bytes(*self.names.add(4 * v as usize).cast::<[u8; 4]>()) };
        let len = unsafe {
            u16::from_ne_bytes(*self.names.add(offset as usize).cast::<[u8; 2]>()) as usize
        };
        let k = unsafe { core::slice::from_raw_parts(self.names.add(offset as usize + 2), len) };
        if name == k {
            Some(v)
        } else {
            None
        }
    }
}
