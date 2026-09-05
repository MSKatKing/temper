use std::arch::x86_64::__m256;
use crate::cpu::buffer::{BufferId, BufferOperation};
use crate::cpu::noise::NoiseAccessor;
use crate::cpu::runtime::{DensityResult, Operation};
use crate::cpu::workspace::{Workspace, WorkspaceStorable};

#[derive(Debug)]
pub struct SqueezeBuffer<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
}

#[derive(Debug)]
pub struct SqueezeNoise<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub noise: NoiseAccessor,
}

struct Abs;

#[derive(Debug)]
pub struct AbsBuffer<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
}

#[derive(Debug)]
pub struct AbsNoise<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub noise: NoiseAccessor,
}

struct Clamp { min: f32, max: f32 }

#[derive(Debug)]
pub struct ClampBuffer<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub min: f32,
    pub max: f32,
}

#[derive(Debug)]
pub struct ClampNoise<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub noise: NoiseAccessor,
    pub min: f32,
    pub max: f32,
}

struct Pow { amt: i32 }

#[derive(Debug)]
pub struct PowBuffer<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub amt: i32,
}

#[derive(Debug)]
pub struct PowNoise<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub noise: NoiseAccessor,
    pub amt: i32,
}

struct NegativeDecay { amt: f32 }

#[derive(Debug)]
pub struct NegativeDecayBuffer<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub amt: f32,
}

#[derive(Debug)]
pub struct NegativeDecayNoise<Dst: WorkspaceStorable> {
    pub dst: BufferId<Dst>,
    pub noise: NoiseAccessor,
    pub amt: f32,
}

impl<Dst: WorkspaceStorable> Operation for SqueezeBuffer<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.iter_mut().for_each(|v| {
            // first two terms of maclaurin series for (1-cos(x))/x
            *v = *v / 2.0 - v.powi(3) / 24.0;
        });
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for SqueezeNoise<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let chunk_pos = workspace.current_pos;
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.pos_iter_mut().for_each(|(pos, v)| {
            let val = self.noise.noise(chunk_pos.chunk_block(pos));
            *v = val / 2.0 - val.powi(3) / 24.0;
        });
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for AbsBuffer<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        Dst::apply_to_self(dst, Abs);
        Ok(())
    }

    unsafe fn execute_simd(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;

        // SAFETY: requirements passed to caller
        unsafe {
            Dst::apply_to_self_simd(dst, Abs);
        }

        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for AbsNoise<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let chunk_pos = workspace.current_pos;
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.pos_iter_mut().for_each(|(pos, v)| {
            *v = self.noise.noise(chunk_pos.chunk_block(pos)).abs();
        });
        Ok(())
    }
}

impl BufferOperation for Abs {
    const READS_DST: bool = false;

    fn scalar(&self, src: f32, _: f32) -> f32 {
        src.abs()
    }

    unsafe fn simd(&self, src: __m256, _: __m256) -> __m256 {
        // SAFETY: requirements passed to caller
        unsafe {
            // clear the sign bit
            std::arch::x86_64::_mm256_and_ps(
                src,
                std::arch::x86_64::_mm256_set1_ps(f32::from_bits(0x7FFF_FFFF))
            )
        }
    }
}

impl BufferOperation for Clamp {
    const READS_DST: bool = false;

    fn scalar(&self, src: f32, _: f32) -> f32 {
        src.clamp(self.min, self.max)
    }

    unsafe fn simd(&self, src: __m256, _: __m256) -> __m256 {
        // SAFETY: requirements passed to caller
        unsafe {
            std::arch::x86_64::_mm256_max_ps(
                std::arch::x86_64::_mm256_set1_ps(self.min),
                std::arch::x86_64::_mm256_min_ps(
                    std::arch::x86_64::_mm256_set1_ps(self.max),
                    src,
                )
            )
        }
    }
}

impl<Dst: WorkspaceStorable> Operation for ClampBuffer<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        Dst::apply_to_self(dst, Clamp { min: self.min, max: self.max });
        Ok(())
    }

    unsafe fn execute_simd(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;

        // SAFETY: requirements passed to caller
        unsafe {
            Dst::apply_to_self_simd(dst, Clamp { min: self.min, max: self.max });
        }

        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for ClampNoise<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let chunk_pos = workspace.current_pos;
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.pos_iter_mut().for_each(|(pos, v)| {
            *v = self.noise.noise(chunk_pos.chunk_block(pos)).clamp(self.min, self.max);
        });
        Ok(())
    }
}

impl BufferOperation for Pow {
    const READS_DST: bool = false;

    fn scalar(&self, src: f32, _: f32) -> f32 {
        src.powi(self.amt)
    }

    unsafe fn simd(&self, src: __m256, _: __m256) -> __m256 {
        let mut pow = src;
        
        // SAFETY: requirements passed to caller
        unsafe {
            for _ in 0..(self.amt.abs() - 1) {
                pow = std::arch::x86_64::_mm256_mul_ps(pow, src);
            }
        }
        
        if self.amt.is_negative() {
            // SAFETY: requirements passed to caller
            unsafe {
                std::arch::x86_64::_mm256_div_ps(
                    std::arch::x86_64::_mm256_set1_ps(1.0),
                    pow,
                )
            }
        } else {
            pow
        }
    }
}

impl<Dst: WorkspaceStorable> Operation for PowBuffer<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        Dst::apply_to_self(dst, Pow { amt: self.amt });
        Ok(())
    }

    unsafe fn execute_simd(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        
        // SAFETY: requirements passed to caller
        unsafe {
            Dst::apply_to_self_simd(dst, Pow { amt: self.amt });
        }
        
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for PowNoise<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let chunk_pos = workspace.current_pos;
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.pos_iter_mut().for_each(|(pos, v)| {
            *v = self.noise.noise(chunk_pos.chunk_block(pos)).powi(self.amt as i32);
        });
        Ok(())
    }
}

impl BufferOperation for NegativeDecay {
    const READS_DST: bool = false;
    
    fn scalar(&self, src: f32, _: f32) -> f32 {
        if src.is_sign_negative() {
            src / self.amt
        } else {
            src
        }
    }

    unsafe fn simd(&self, src: __m256, dst: __m256) -> __m256 {
        todo!()
    }
}

impl<Dst: WorkspaceStorable> Operation for NegativeDecayBuffer<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        Dst::apply_to_self(dst, NegativeDecay { amt: self.amt });
        Ok(())
    }
}

impl<Dst: WorkspaceStorable> Operation for NegativeDecayNoise<Dst> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let chunk_pos = workspace.current_pos;
        let dst = workspace.get_buffer_mut(self.dst)?;
        dst.pos_iter_mut().for_each(|(pos, v)| { 
            let noise = self.noise.noise(chunk_pos.chunk_block(pos));
            *v = if noise.is_sign_negative() {
                noise / self.amt
            } else {
                noise
            }
        });
        Ok(())
    }
}
