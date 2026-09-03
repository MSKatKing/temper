mod math;

use std::fmt::Debug;
pub use math::*;
use crate::cpu::workspace::Workspace;

pub trait Operation: Debug {
    fn execute(&self, workspace: &mut Workspace) -> Option<()>;

    unsafe fn execute_simd(&self, _workspace: &mut Workspace) -> Option<()> {
        todo!()
    }
}
