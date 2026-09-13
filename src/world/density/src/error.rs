pub type DensityResult<T> = Result<T, DensityRuntimeError>;
pub type DensityCompileResult<T> = Result<T, DensityCompileError>;

pub enum DensityRuntimeError {
    StackUnderflow,
}

pub enum DensityCompileError {
    
}
