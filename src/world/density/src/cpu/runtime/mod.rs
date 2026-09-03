mod math;

use crate::cpu::workspace::Workspace;
pub use math::*;
use std::fmt::Debug;

pub trait Operation: Debug + Send + Sync {
    fn execute(&self, workspace: &mut Workspace) -> Option<()>;

    unsafe fn execute_simd(&self, workspace: &mut Workspace) -> Option<()> {
        // if types don't provide a SIMD alternative, default to the scalar implementation
        self.execute(workspace)
    }
}
