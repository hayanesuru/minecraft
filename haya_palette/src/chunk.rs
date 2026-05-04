use crate::Palette;
use alloc::vec::Vec;
use hashbrown::HashTable;
use mser::cold_path;

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
    pub chunks: HashTable<(u64, u64)>,
    pub direct_key: Vec<u32>,
    pub indirect4_key: Vec<u32>,
    pub indirect2_key: Vec<u32>,
    pub single_key: Vec<u32>,
}

impl<T: Palette> Default for ChunkCache<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Palette> ChunkCache<T> {
    pub const fn new() -> Self {
        Self {
            direct: Vec::new(),
            indirect2: Vec::new(),
            indirect4: Vec::new(),
            chunks: HashTable::new(),
            direct_key: Vec::new(),
            indirect4_key: Vec::new(),
            indirect2_key: Vec::new(),
            single_key: Vec::new(),
        }
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> T {
        let j = ((x & 3) | ((y & 3) << 2) | ((z & 3) << 4)) as usize;
        let chunk = pack(x >> 2, y >> 2, z >> 2);
        let t = match self.chunks.find(mix(chunk), |x| x.0 == chunk) {
            Some(t) => t.1,
            None => {
                cold_path();
                return T::default();
            }
        };
        let n = (t & INDEX_MASK) as usize;
        let ty = t >> 62;
        unsafe {
            match ty {
                3 => T::from_id(n as u32),
                2 => self.indirect2.get_unchecked(n).get(j),
                1 => self.indirect4.get_unchecked(n).get(j),
                _ => *self.direct.get_unchecked(n).data.get_unchecked(j),
            }
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

const PRIME_MAX_A: u32 = u32::MAX - 4;
const PRIME_MAX_B: u32 = u32::MAX - 16;
const PRIME_MAX_C: u32 = u32::MAX - 64;

#[inline]
fn pack(x: i32, y: i32, z: i32) -> u64 {
    let sx = (x & 0x3FF_FFFF) as i64;
    let sy = (y & 0xFFF) as i64;
    let sz = (z & 0x3FF_FFFF) as i64;
    ((sx << 38) | (sz << 12) | sy) as u64
}

// 32 bits
#[inline]
fn mix(v: u64) -> u64 {
    let x = (v >> 38) as i32 as u32;
    let y = ((v << 52) >> 52) as i32 as u32;
    let z = ((v << 26) >> 38) as i32 as u32;
    let m = PRIME_MAX_A
        .wrapping_mul(x)
        .wrapping_add(PRIME_MAX_B.wrapping_mul(y))
        .wrapping_add(PRIME_MAX_C.wrapping_mul(z));
    m as u64
}
