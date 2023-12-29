mod light;
mod paletted;
mod section;

pub use self::light::NibbleArray;
pub use self::paletted::PalettedContainer;
pub use self::section::Section;
use crate::nbt::{Compound, COMPOUND};
use crate::{UnsafeWriter, Write, V21};
use minecraft_data::{biome, block_entity_type, block_state};

type BlockPC<A = std::alloc::Global> = PalettedContainer<
    block_state,
    16,
    { (usize::BITS - block_state::MAX.leading_zeros()) as u8 },
    { 16 * 16 * 16 },
    A,
>;

type BiomePC<A = std::alloc::Global> = PalettedContainer<
    biome,
    4,
    { (usize::BITS - biome::MAX.leading_zeros()) as u8 },
    { 4 * 4 * 4 },
    A,
>;

#[derive(Clone, Copy)]
pub struct ChunkData<'a> {
    pub sections: &'a [Section],
    pub block_entities: &'a [BlockEntity],
}

#[derive(Clone)]
pub struct BlockEntity {
    pub xz: u8,
    pub y: i16,
    pub ty: block_entity_type,
    pub nbt: Compound,
}

impl Write for ChunkData<'_> {
    fn write(&self, w: &mut UnsafeWriter) {
        let len = self
            .sections
            .iter()
            .map(|x| x.states.len() + x.biomes.len())
            .sum::<usize>()
            + self.sections.len() * 2;

        V21(len as u32).write(w);
        for section in self.sections.iter() {
            section.non_air.write(w);
            section.states.write(w);
            section.biomes.write(w);
        }
        V21(self.block_entities.len() as u32).write(w);
        for x in self.block_entities.iter() {
            let BlockEntity { xz, y, ty, nbt } = x;
            (*xz).write(w);
            (*y).write(w);
            (*ty).write(w);
            w.write_byte(COMPOUND);
            nbt.write(w);
        }
    }

    fn len(&self) -> usize {
        let mut len = self.sections.len() * 2
            + self
                .sections
                .iter()
                .map(|x| x.states.len() + x.biomes.len())
                .sum::<usize>();

        len += V21(len as u32).len();
        len += V21(self.block_entities.len() as u32).len();
        for x in self.block_entities.iter() {
            let BlockEntity {
                xz: _,
                y: _,
                ty,
                nbt,
            } = x;
            len += 4;
            len += ty.len();
            len += nbt.len();
        }

        len
    }
}

#[derive(Copy, Clone)]
pub struct LightData<'a> {
    pub sky_light: &'a [Option<NibbleArray>],
    pub block_light: &'a [Option<NibbleArray>],
}

impl Write for LightData<'_> {
    #[inline]
    fn write(&self, w: &mut UnsafeWriter) {
        let mut sky_light = 0_u64;
        let mut sky_light_empty = 0_u64;
        let mut block_light = 0_u64;
        let mut block_light_empty = 0_u64;
        let mut n = 1_u64;
        let mut len_sky = 0_u32;
        let mut len_block = 0_u32;
        for x in self.sky_light {
            if x.is_some() {
                sky_light |= n;
                len_sky += 1;
            } else {
                sky_light_empty |= n;
            }
            n <<= 1;
        }
        n = 1;
        for x in self.block_light {
            if x.is_some() {
                block_light |= n;
                len_block += 1;
            } else {
                block_light_empty |= n;
            }
            n <<= 1;
        }
        w.write_byte(1);
        sky_light.write(w);
        w.write_byte(1);
        block_light.write(w);
        w.write_byte(1);
        sky_light_empty.write(w);
        w.write_byte(1);
        block_light_empty.write(w);

        V21(len_sky).write(w);
        for light in self.sky_light.iter().flatten() {
            V21(2048).write(w);
            w.write(light.as_bytes());
        }

        V21(len_block).write(w);
        for light in self.block_light.iter().flatten() {
            V21(2048).write(w);
            w.write(light.as_bytes());
        }
    }

    #[inline]
    fn len(&self) -> usize {
        let mut len = 9 * 4;

        let mut len_sky = 0_u32;
        let mut len_block = 0_u32;
        for x in self.sky_light {
            if x.is_some() {
                len_sky += 1;
                len += V21(2048).len() + 2048;
            }
        }
        for x in self.block_light {
            if x.is_some() {
                len_block += 1;
                len += V21(2048).len() + 2048;
            }
        }
        len += V21(len_sky).len();
        len += V21(len_block).len();
        len
    }
}
