use crate::cpu::buffer::{Add, BufferApplyTo, BufferId, BufferType, Div, Max, Min, Mul, Replace, Sub};
use crate::cpu::noise::NoiseAccessor;
use crate::cpu::runtime::{DensityResult, Operation};
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

#[derive(Debug)]
pub struct BufferMul<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> {
    pub dst: BufferId<Dst>,
    pub src: BufferId<Src>,
}

#[derive(Debug)]
pub struct ConstantMul<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub src: f32,
}

#[derive(Debug)]
pub struct NoiseMul<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub src: NoiseAccessor,
}

#[derive(Debug)]
pub struct BufferMin<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> {
    pub dst: BufferId<Dst>,
    pub src: BufferId<Src>,
}

#[derive(Debug)]
pub struct ConstantMin<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub src: f32,
}

#[derive(Debug)]
pub struct NoiseMin<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub src: NoiseAccessor,
}

#[derive(Debug)]
pub struct BufferMax<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> {
    pub dst: BufferId<Dst>,
    pub src: BufferId<Src>,
}

#[derive(Debug)]
pub struct ConstantMax<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub src: f32,
}

#[derive(Debug)]
pub struct NoiseMax<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub src: NoiseAccessor,
}

#[derive(Debug)]
pub struct BufferSub<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> {
    pub dst: BufferId<Dst>,
    pub src: BufferId<Src>,
}

#[derive(Debug)]
pub struct ConstantSub<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub src: f32,
}

#[derive(Debug)]
pub struct NoiseSub<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub src: NoiseAccessor,
}

#[derive(Debug)]
pub struct BufferDiv<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> {
    pub dst: BufferId<Dst>,
    pub src: BufferId<Src>,
}

#[derive(Debug)]
pub struct ConstantDiv<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub src: f32,
}

#[derive(Debug)]
pub struct NoiseDiv<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub src: NoiseAccessor,
}

impl<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> Operation for FillBuffer<Dst, Src> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;
        Src::apply_to::<Replace>(src, dst);
        Ok(())
    }

    unsafe fn execute_simd(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;

        // SAFETY: requirements passed to caller
        unsafe {
            Src::apply_to_simd::<Replace>(src, dst);
        }

        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for FillConstant<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.fill(self.src);
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for FillNoise<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let chunk_pos = workspace.current_pos;
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.pos_iter_mut().for_each(|(pos, v)| {
            *v = self.noise.noise(chunk_pos.chunk_block(pos));
        });
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for YClampedGradient<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
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
        Ok(())
    }
}

impl<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> Operation for BufferAdd<Dst, Src> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;
        Src::apply_to::<Add>(src, dst);
        Ok(())
    }

    unsafe fn execute_simd(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;

        // SAFETY: requirements passed to caller
        unsafe {
            Src::apply_to_simd::<Add>(src, dst);
        }

        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for ConstantAdd<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.iter_mut().for_each(|v| *v += self.src);
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for NoiseAdd<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let chunk_pos = workspace.current_pos;
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.pos_iter_mut()
            .for_each(|(pos, v)| *v += self.src.noise(chunk_pos.chunk_block(pos)));
        Ok(())
    }
}

impl<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> Operation for BufferMul<Dst, Src> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;
        Src::apply_to::<Mul>(src, dst);
        Ok(())
    }

    unsafe fn execute_simd(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;

        // SAFETY: requirements passed to caller
        unsafe {
            Src::apply_to_simd::<Mul>(src, dst);
        }

        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for ConstantMul<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.iter_mut().for_each(|v| *v *= self.src);
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for NoiseMul<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let chunk_pos = workspace.current_pos;
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.pos_iter_mut().for_each(|(pos, v)| {
            *v *= self.src.noise(chunk_pos.chunk_block(pos));
        });
        Ok(())
    }
}

impl<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> Operation for BufferMin<Dst, Src> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;
        Src::apply_to::<Min>(src, dst);
        Ok(())
    }

    unsafe fn execute_simd(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;
        
        // SAFETY: requirements passed to caller
        unsafe {
            Src::apply_to_simd::<Min>(src, dst);
        }
        
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for ConstantMin<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.iter_mut().for_each(|v| *v *= v.min(self.src));
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for NoiseMin<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let chunk_pos = workspace.current_pos;
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.pos_iter_mut().for_each(|(pos, v)| {
            *v = v.min(self.src.noise(chunk_pos.chunk_block(pos)));
        });
        Ok(())
    }
}

impl<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> Operation for BufferMax<Dst, Src> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;
        Src::apply_to::<Max>(src, dst);
        Ok(())
    }

    unsafe fn execute_simd(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;
        
        // SAFETY: requirements passed to caller
        unsafe {
            Src::apply_to_simd::<Max>(src, dst);
        }
        
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for ConstantMax<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.iter_mut().for_each(|v| *v *= v.max(self.src));
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for NoiseMax<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let chunk_pos = workspace.current_pos;
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.pos_iter_mut().for_each(|(pos, v)| {
            *v *= v.max(self.src.noise(chunk_pos.chunk_block(pos)));
        });
        Ok(())
    }
}

impl<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> Operation for BufferSub<Dst, Src> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;
        Src::apply_to::<Sub>(src, dst);
        Ok(())
    }

    unsafe fn execute_simd(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;
        
        // SAFETY: requirements passed to caller
        unsafe {
            Src::apply_to_simd::<Sub>(src, dst);
        }
        
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for ConstantSub<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.iter_mut().for_each(|v| *v -= self.src);
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for NoiseSub<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let chunk_pos = workspace.current_pos;
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.pos_iter_mut().for_each(|(pos, v)| {
            *v -= self.src.noise(chunk_pos.chunk_block(pos));
        });
        Ok(())
    }
}

impl<Dst: BufferType, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> Operation for BufferDiv<Dst, Src> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;
        Src::apply_to::<Div>(src, dst);
        Ok(())
    }

    unsafe fn execute_simd(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let (dst, src) = workspace.get_dst_src(self.dst, self.src)?;
        
        // SAFETY: requirements passed to caller
        unsafe {
            Src::apply_to_simd::<Div>(src, dst);
        }
        
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for ConstantDiv<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.iter_mut().for_each(|v| *v /= self.src);
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for NoiseDiv<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let chunk_pos = workspace.current_pos;
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.pos_iter_mut().for_each(|(pos, v)| {
            *v /= self.src.noise(chunk_pos.chunk_block(pos));
        });
        Ok(())
    }
}
