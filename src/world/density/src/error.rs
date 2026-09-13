pub type DensityResult<T> = Result<T, DensityRuntimeError>;
pub type DensityCompileResult<T> = Result<T, DensityCompileError>;

pub enum DensityRuntimeError {
    StackUnderflow,
    PositionStackUnderflow,
    SplineRuntimeOutOfBounds {
        got: usize,
        max: usize,
    }
}

pub enum DensityCompileError {
    
}
