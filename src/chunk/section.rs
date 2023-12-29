use super::{BiomePC, BlockPC};
use std::alloc::Global;
use minecraft_data::{biome, block_state};

#[derive(Clone)]
pub struct Section {
    pub states: BlockPC,
    pub biomes: BiomePC,
    pub non_air: u16,
}

impl Default for Section {
    #[inline]
    fn default() -> Self {
        Self {
            states: BlockPC::new(block_state::AIR, Global),
            biomes: BiomePC::new(biome::plains, Global),
            non_air: 0,
        }
    }
}

impl Section {
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.non_air == 0
    }

    #[inline]
    pub const fn get_index(x: usize, y: usize, z: usize) -> usize {
        x | (z << 4) | (y << 8)
    }

    #[inline]
    pub const fn get_index_vec(vec: glam::IVec3) -> usize {
        ((vec.x & 0xf) as usize) | (((vec.z & 0xf) as usize) << 4) | (((vec.y & 0xf) as usize) << 8)
    }

    /// # Safety
    ///
    /// `index` < 0x1000
    #[inline]
    pub unsafe fn get_block(&self, index: usize) -> block_state {
        self.states.get(index)
    }

    #[inline]
    pub fn set_all_block(section: &mut Self, block: block_state) {
        section.states = BlockPC::new(block, Global);
        if block != block_state::AIR {
            section.non_air = 4096;
        } else {
            section.non_air = 0;
        }
    }

    /// # Safety
    ///
    /// `index` < 0x1000
    #[inline]
    pub unsafe fn set_block(&mut self, block: block_state, index: usize) -> block_state {
        let old = self.states.set(index, block);
        if (old == block_state::AIR) != (block == block_state::AIR) {
            if old == block_state::AIR {
                self.non_air += 1;
            } else {
                self.non_air -= 1;
            }
        }
        old
    }
}
