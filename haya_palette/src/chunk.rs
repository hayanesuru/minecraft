use crate::Biome;
use alloc::vec::Vec;
use hashbrown::HashTable;
use minecraft_data::{block, block_state};
use mser::cold_path;

const BLOCK_PER_CHUNK: usize = 4 * 4 * 4;
const INDIRECT4_PER_CHUNK: usize = BLOCK_PER_CHUNK / 2;
const INDIRECT2_PER_CHUNK: usize = BLOCK_PER_CHUNK / 4;
const INDEX_MASK: u64 = 0x3FFF_FFFF_FFFF_FFFF;
const VOID_AIR: block_state = block::void_air.state_default();

#[derive(Clone)]
pub struct Direct<T: Copy> {
    data: [T; BLOCK_PER_CHUNK],
}

#[derive(Clone)]
pub struct Indirect4<T: Copy> {
    palette: [T; 16],
    data: [u8; INDIRECT4_PER_CHUNK],
}

#[derive(Clone)]
pub struct Indirect2<T: Copy> {
    palette: [T; 4],
    data: [u8; INDIRECT2_PER_CHUNK],
}

#[derive(Clone)]
pub struct ChunkCache {
    pub direct: Vec<Direct<block_state>>,
    pub indirect2: Vec<Indirect2<block_state>>,
    pub indirect4: Vec<Indirect4<block_state>>,
    pub biome: Vec<Biome>,
    pub chunks: HashTable<(u64, u64)>,
    pub direct_key: Vec<u32>,
    pub indirect4_key: Vec<u32>,
    pub indirect2_key: Vec<u32>,
    pub single_key: Vec<u32>,
}

impl Default for ChunkCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkCache {
    pub const fn new() -> Self {
        Self {
            direct: Vec::new(),
            indirect2: Vec::new(),
            indirect4: Vec::new(),
            biome: Vec::new(),
            chunks: HashTable::new(),
            direct_key: Vec::new(),
            indirect4_key: Vec::new(),
            indirect2_key: Vec::new(),
            single_key: Vec::new(),
        }
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> block_state {
        let j = ((x & 3) | ((y & 3) << 2) | ((z & 3) << 4)) as usize;
        let chunk = pack(x >> 2, y >> 2, z >> 2);
        let t = match self.chunks.find(mix(chunk), |(k, _)| *k == chunk) {
            Some(t) => t.1,
            None => {
                cold_path();
                return VOID_AIR;
            }
        };
        let n = (t & INDEX_MASK) as usize;
        let ty = t >> 62;
        unsafe {
            // jump table
            match ty {
                3 => block_state::new(n as u16).unwrap_unchecked(),
                2 => self.indirect2.get_unchecked(n).get(j),
                1 => self.indirect4.get_unchecked(n).get(j),
                _ => *self.direct.get_unchecked(n).data.get_unchecked(j),
            }
        }
    }
}

impl<T: Copy> Indirect2<T> {
    unsafe fn get(&self, index: usize) -> T {
        unsafe {
            let b = *self.data.get_unchecked(index >> 2);
            let i = (b >> ((index & 3) << 1)) & 0x3;
            *self.palette.get_unchecked(i as usize)
        }
    }
}

impl<T: Copy> Indirect4<T> {
    unsafe fn get(&self, index: usize) -> T {
        unsafe {
            let b = *self.data.get_unchecked(index >> 1);
            let i = (b >> ((index & 1) << 2)) & 0xF;
            *self.palette.get_unchecked(i as usize)
        }
    }
}

#[inline]
fn pack(x: i32, y: i32, z: i32) -> u64 {
    let sx = (x & 0x3FF_FFFF) as i64;
    let sy = (y & 0xFFF) as i64;
    let sz = (z & 0x3FF_FFFF) as i64;
    ((sx << 38) | (sz << 12) | sy) as u64
}

#[inline]
fn mix(v: u64) -> u64 {
    let h = 11400714819323198485u64.wrapping_mul(v);
    (h >> 32) ^ h
}
