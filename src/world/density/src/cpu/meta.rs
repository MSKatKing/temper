use std::num::NonZeroUsize;

pub struct FunctionMetadata {
    full_offset: NonZeroUsize,
    flat_offset: NonZeroUsize,
    flat_cell_offset: NonZeroUsize,
    interpolated_offset: NonZeroUsize,
    buffer_count: usize,
}