use std::borrow::Borrow;
use std::range::RangeInclusive;
use crate::pos::BlockPos;

/// Supplies various random functions for structs. Based on `net.minecraft.util.RandomSource`.
pub trait RandomSource {
    fn fork(&mut self) -> Self where Self: Sized;
    fn fork_positional(&mut self) -> impl PositionalRandom<Self> where Self: Sized;

    fn next_u32(&mut self) -> u32;
    fn next_u32_bounded(&mut self, limit: u32) -> u32;
    fn next_u32_range<Range: Into<RangeInclusive<u32>>>(&mut self, range: Range) -> u32 {
        let range = range.into();
        self.next_u32_bounded(range.last - range.start + 1) + range.start
    }

    fn next_u64(&mut self) -> u64;
    fn next_bool(&mut self) -> bool;
    fn next_f32(&mut self) -> f32;
    fn next_f64(&mut self) -> f64;
}

/// Provides a way to spawn various RandomSources from input seeds.
pub trait PositionalRandom<R: RandomSource> {
    fn spawn_at(&self, x: i32, y: i32, z: i32) -> R;
    fn spawn_from_seed(&self, seed: u64) -> R;
    fn spawn_from_hash<S: Borrow<str>>(&self, hash: S) -> R;

    fn spawn_at_pos(&self, pos: BlockPos) -> R {
        self.spawn_at(pos.pos.x, pos.pos.y, pos.pos.z)
    }
}