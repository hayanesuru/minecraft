#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub struct WyRand {
    pub seed: u64,
}

impl Default for WyRand {
    #[inline]
    fn default() -> Self {
        Self {
            seed: 0xd42a9a3d308d9321,
        }
    }
}

impl WyRand {
    #[inline]
    pub fn nx(&mut self) -> u64 {
        self.seed = self.seed.wrapping_add(0xa0761d6478bd642f);
        let x = (self.seed ^ 0xe7037ed1a0b428db) as u128;
        let t = (self.seed as u128).wrapping_mul(x);
        (t.wrapping_shr(64) ^ t) as u64
    }
}

impl rand_core::RngCore for WyRand {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.nx() as u32
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.nx()
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        rand_core::impls::fill_bytes_via_next(self, dest)
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl rand_core::SeedableRng for WyRand {
    type Seed = [u8; 8];

    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            seed: u64::from_ne_bytes(seed),
        }
    }

    #[inline]
    fn seed_from_u64(state: u64) -> Self {
        Self { seed: state }
    }

    #[inline]
    fn from_rng<R: rand_core::RngCore>(mut rng: R) -> Result<Self, rand_core::Error> {
        Ok(Self {
            seed: rng.next_u64(),
        })
    }
}

impl rand_core::CryptoRng for WyRand {}
