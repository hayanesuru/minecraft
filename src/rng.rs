use rand_core::{CryptoRng, RngCore, SeedableRng};

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

impl RngCore for WyRand {
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

impl SeedableRng for WyRand {
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
    fn from_rng<R: RngCore>(mut rng: R) -> Result<Self, rand_core::Error> {
        Ok(Self {
            seed: rng.next_u64(),
        })
    }
}

impl CryptoRng for WyRand {}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(align(16))]
pub struct Xoroshiro128PlusPlus {
    pub s0: u64,
    pub s1: u64,
}

impl Default for Xoroshiro128PlusPlus {
    #[inline]
    fn default() -> Self {
        Self {
            s0: 0x888C728A799BB7D3,
            s1: 0x5EF162171CFD90B8,
        }
    }
}

impl Xoroshiro128PlusPlus {
    const ZERO: Self = Self { s0: 0, s1: 0 };

    pub fn check(self) -> Self {
        if self != Self::ZERO {
            self
        } else {
            Self::default()
        }
    }

    #[inline]
    pub fn nx(&mut self) -> u64 {
        let x = self.s0;
        let mut y = self.s1;
        let k = (x + y).rotate_left(17) + x;
        y ^= x;
        self.s0 = x.rotate_left(49) ^ y ^ (y << 21);
        self.s1 = y.rotate_left(28);
        k
    }
}

impl RngCore for Xoroshiro128PlusPlus {
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

impl SeedableRng for Xoroshiro128PlusPlus {
    type Seed = [u8; 16];

    #[inline]
    fn from_seed(seed: [u8; 16]) -> Self {
        let [a1, a2, a3, a4, a5, a6, a7, a8, b1, b2, b3, b4, b5, b6, b7, b8] = seed;
        Self {
            s0: u64::from_ne_bytes([a1, a2, a3, a4, a5, a6, a7, a8]),
            s1: u64::from_ne_bytes([b1, b2, b3, b4, b5, b6, b7, b8]),
        }
        .check()
    }

    #[inline]
    fn seed_from_u64(state: u64) -> Self {
        Self {
            s0: state,
            s1: 0x99136675D5F4612F,
        }
    }

    #[inline]
    fn from_rng<R: RngCore>(mut rng: R) -> Result<Self, rand_core::Error> {
        let x = Self {
            s0: rng.next_u64(),
            s1: rng.next_u64(),
        }
        .check();
        Ok(x)
    }
}
