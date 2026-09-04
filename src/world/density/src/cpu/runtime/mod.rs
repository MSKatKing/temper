mod math;

use crate::cpu::workspace::Workspace;
pub use math::*;
use std::fmt::Debug;
use crate::cpu::compiler::AnyBufferId;

pub trait Operation: Debug + Send + Sync {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()>;

    unsafe fn execute_simd(&self, workspace: &mut Workspace) -> DensityResult<()> {
        // if types don't provide a SIMD alternative, default to the scalar implementation
        self.execute(workspace)
    }
}

pub type DensityResult<T> = Result<T, DensityError>;

#[derive(Debug)]
pub enum DensityError {
    MissingBuffer(AnyBufferId),
    DstSrcSameBuffer(AnyBufferId),
    InvalidDstSrc(AnyBufferId, AnyBufferId),
}
