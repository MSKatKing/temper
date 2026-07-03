use std::borrow::Borrow;
use std::ops::{Not, Rem};
use crate::random::{seed_from_hash, seed_from_pos, u128_high, u128_low, u64_to_u128, upgrade_seed_mixed, PositionalRandom, RandomSource};

pub struct XoroshiroRandomSource {
    seed: u128,
}

pub struct XoroshiroPositionalRandom {
    seed: u128,
}

impl XoroshiroRandomSource {
    const F32_UNIT: f32 = 5.9604645e-8f32;
    const F64_UNIT: f32 = 1.110223e-16f32;

    pub fn new(seed: u64) -> XoroshiroRandomSource {
        XoroshiroRandomSource {
            seed: upgrade_seed_mixed(seed),
        }
    }

    pub fn new_parts(low: u64, high: u64) -> XoroshiroRandomSource {
        if (low | high) == 0 {
            Self::new_u128(0x6a09e667f3bcc9099e3779b97f4a7c15)
        } else {
            Self::new_u128(u64_to_u128(low, high))
        }
    }

    fn new_u128(seed: u128) -> XoroshiroRandomSource {
        XoroshiroRandomSource {
            seed,
        }
    }

    fn next_bits(&mut self, count: u8) -> u64 {
        debug_assert!(count <= 64);
        self.next_u64() >> (64 - count)
    }
}

impl XoroshiroPositionalRandom {
    fn new(seed_low: u64, seed_high: u64) -> XoroshiroPositionalRandom {
        XoroshiroPositionalRandom {
            seed: u64_to_u128(seed_low, seed_high),
        }
    }
}

impl RandomSource for XoroshiroRandomSource {
    fn fork(&mut self) -> Self
    where
        Self: Sized,
    {
        XoroshiroRandomSource::new_parts(self.next_u64(), self.next_u64())
    }

    fn fork_positional(&mut self) -> impl PositionalRandom<Self>
    where
        Self: Sized,
    {
        XoroshiroPositionalRandom::new(self.next_u64(), self.next_u64())
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u32_bounded(&mut self, limit: u32) -> u32 {
        let limit = limit as u64;
        let mut random_bits = self.next_u32() as u64;
        let mut multiplied_random_bits = random_bits.wrapping_mul(limit);
        let mut fractional_part = multiplied_random_bits & 4294967295u64;

        if fractional_part < limit {
            let unbiased_buckets_start_idx = limit.not().wrapping_add(1).rem(limit);
            while fractional_part < unbiased_buckets_start_idx {
                random_bits = self.next_u32() as u64;
                multiplied_random_bits = random_bits.wrapping_mul(limit);
                fractional_part = multiplied_random_bits & 4294967295u64;
            }
        }

        (multiplied_random_bits >> 32) as u32
    }

    fn next_u64(&mut self) -> u64 {
        let s0 = u128_low(self.seed);
        let s1 = u128_high(self.seed);
        let res = s0.wrapping_add(s1).rotate_left(17).wrapping_add(s0);
        let s1 = s1 ^ s0;
        let low = s0.rotate_left(49) ^ s1 ^ s1 << 21;
        let high = s1.rotate_left(28);
        self.seed = u64_to_u128(low, high);
        res
    }

    fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1u64) != 0u64
    }

    fn next_f32(&mut self) -> f32 {
        self.next_bits(24) as f32 * Self::F32_UNIT
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_bits(53) as f32 * Self::F64_UNIT) as f64
    }

    fn consume_count(&mut self, count: usize) {
        for _ in 0..count {
            let _ = self.next_u64();
        }
    }
}

impl PositionalRandom<XoroshiroRandomSource> for XoroshiroPositionalRandom {
    fn spawn_at(&self, x: i32, y: i32, z: i32) -> XoroshiroRandomSource {
        let pos_seed = seed_from_pos(x, y, z);
        XoroshiroRandomSource::new_parts(pos_seed ^ u128_low(self.seed), u128_high(self.seed))
    }

    fn spawn_from_seed(&self, seed: u64) -> XoroshiroRandomSource {
        let seed = u64_to_u128(seed, seed);
        XoroshiroRandomSource::new_u128(seed ^ self.seed)
    }

    fn spawn_from_hash<S: Borrow<str>>(&self, hash: S) -> XoroshiroRandomSource {
        let seed = seed_from_hash(hash.borrow());
        XoroshiroRandomSource::new_u128(seed ^ self.seed)
    }
}