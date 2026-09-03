use crate::cpu::buffer::{Add, BufferApplyTo, BufferId, BufferType, Replace};
use crate::cpu::noise::NoiseAccessor;
use crate::cpu::runtime::Operation;
use crate::cpu::workspace::{GetDstSrc, Workspace, WorkspaceStorable};
use std::fmt::Debug;
use std::ops::RangeInclusive;
use temper_core::math::lerp;

#[derive(Debug)]
#[allow(dead_code)]
pub struct FillBuffer<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> {
    pub dst: BufferId<Dst>,
    pub src: BufferId<Src>,
}

#[derive(Debug)]
pub struct FillConstant<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub src: f32,
}

#[derive(Debug)]
pub struct FillNoise<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub noise: NoiseAccessor,
}

#[derive(Debug)]
pub struct YClampedGradient<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub y_range: RangeInclusive<i16>,
    pub value_range: RangeInclusive<f32>,
}

#[derive(Debug)]
pub struct BufferAdd<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> {
    pub dst: BufferId<Dst>,
    pub src: BufferId<Src>,
}

#[derive(Debug)]
pub struct ConstantAdd<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub src: f32,
}

#[derive(Debug)]
pub struct NoiseAdd<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub src: NoiseAccessor,
}

impl<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> Operation for FillBuffer<Dst, Src> {
    fn execute(&self, workspace: &mut Workspace) -> Option<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;
        Src::apply_to::<Replace>(src, dst);
        Some(())
    }

    unsafe fn execute_simd(&self, workspace: &mut Workspace) -> Option<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;

        // SAFETY: requirements passed to caller
        unsafe {
            Src::apply_to_simd::<Replace>(src, dst);
        }

        Some(())
    }
}

impl<Dst: WorkspaceStorable> Operation for FillConstant<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> Option<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.fill(self.src);
        Some(())
    }
}

impl<Dst: WorkspaceStorable> Operation for FillNoise<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> Option<()> {
        let chunk_pos = workspace.current_pos;
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.pos_iter_mut().for_each(|(pos, v)| {
            *v = self.noise.noise(chunk_pos.chunk_block(pos));
        });
        Some(())
    }
}

impl<Dst: WorkspaceStorable> Operation for YClampedGradient<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> Option<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.pos_iter_mut().for_each(|(pos, v)| {
            if pos.y() > *self.y_range.end() {
                *v = *self.value_range.end()
            } else {
                *v = lerp(
                    (pos.y() as f64 - *self.y_range.start() as f64)
                        / (self.y_range.end() - self.y_range.start()) as f64,
                    [
                        *self.value_range.start() as f64,
                        *self.value_range.end() as f64,
                    ],
                ) as f32;
            }
        });
        Some(())
    }
}

impl<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> Operation for BufferAdd<Dst, Src> {
    fn execute(&self, workspace: &mut Workspace) -> Option<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;
        Src::apply_to::<Add>(src, dst);
        Some(())
    }

    unsafe fn execute_simd(&self, workspace: &mut Workspace) -> Option<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;

        // SAFETY: requirements passed to caller
        unsafe {
            Src::apply_to_simd::<Add>(src, dst);
        }

        Some(())
    }
}

impl<Dst: WorkspaceStorable> Operation for ConstantAdd<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> Option<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.iter_mut().for_each(|v| *v += self.src);
        Some(())
    }
}

impl<Dst: WorkspaceStorable> Operation for NoiseAdd<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> Option<()> {
        let chunk_pos = workspace.current_pos;
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.pos_iter_mut()
            .for_each(|(pos, v)| *v += self.src.noise(chunk_pos.chunk_block(pos)));
        Some(())
    }
}
