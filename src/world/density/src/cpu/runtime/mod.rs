mod math;

use crate::cpu::buffer::BufferId;
use crate::cpu::compiler::AnyBufferId;
use crate::cpu::workspace::{GetDstSrc, Workspace, WorkspaceStorable};
use bevy_math::DVec3;
pub use math::*;
use std::fmt::Debug;
use temper_noise::NormalNoise;

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

#[derive(Debug)]
pub struct ShiftedNoise<Dst: WorkspaceStorable + GetDstSrc<Dst>> {
    pub dst: BufferId<Dst>,
    pub noise: NormalNoise,
    pub xz_scale: f32,
    pub y_scale: f32,
    pub shift_x: BufferId<Dst>,
    pub shift_y: BufferId<Dst>,
    pub shift_z: BufferId<Dst>,
}

impl<Dst: WorkspaceStorable + GetDstSrc<Dst>> Operation for ShiftedNoise<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let chunk_pos = workspace.current_pos;
        let (dst, x, y, z) = Dst::get_dst_src_3(
            workspace,
            self.dst,
            self.shift_x,
            self.shift_y,
            self.shift_z,
        )?;
        dst.pos_iter_mut()
            .zip(x.iter())
            .zip(y.iter())
            .zip(z.iter())
            .for_each(|((((pos, dst), x), y), z)| {
                let pos = chunk_pos.chunk_block(pos);
                *dst = self.noise.noise(DVec3::new(
                    ((pos.pos.x as f32 + x) * self.xz_scale) as f64,
                    ((pos.pos.y as f32 + y) * self.y_scale) as f64,
                    ((pos.pos.z as f32 + z) * self.xz_scale) as f64,
                )) as f32;
            });
        Ok(())
    }
}
