use crate::cpu::runtime::{DensityResult, Operation};
use std::arch::x86_64::__m256;
use crate::cpu::buffer::{BufferApplyTo, BufferId, BufferOperation, BufferType};
use crate::cpu::workspace::{GetDstSrc, Workspace, WorkspaceStorable};

const MANTISSA_MASK: u32 = 0x007F_FFFF;

struct IntervalMark<'a>(&'a [f32]);
struct IntervalFill(usize);

#[derive(Debug)]
pub struct IntervalSelect<Dst: WorkspaceStorable, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> {
    pub dst: BufferId<Dst>,
    pub thresholds: Vec<f32>,
    pub functions: Vec<BufferId<Src>>,
}

impl BufferOperation for IntervalMark<'_> {
    const READS_DST: bool = false;

    fn scalar(&self, src: f32, _: f32) -> f32 {
        let mut i = 0;
        while i < self.0.len() && src > self.0[i] {
            i += 1;
        }

        f32::from_bits((!0 & !MANTISSA_MASK) + i as u32)
    }

    unsafe fn simd(&self, src: __m256, dst: __m256) -> __m256 {
        todo!()
    }
}

impl BufferOperation for IntervalFill {
    const READS_DST: bool = true;

    fn scalar(&self, src: f32, dst: f32) -> f32 {
        let idx = (f32::to_bits(src) & MANTISSA_MASK) as usize;
        if idx == self.0 {
            dst
        } else {
            src
        }
    }

    unsafe fn simd(&self, src: __m256, dst: __m256) -> __m256 {
        todo!()
    }
}

impl<Dst: WorkspaceStorable, Src: BufferApplyTo<Dst> + GetDstSrc<Dst>> Operation for IntervalSelect<Dst, Src> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let dst = workspace.get_buffer_mut(self.dst)?;
        Dst::apply_to_self(dst, IntervalMark(&self.thresholds));

        for (i, src) in self.functions.iter().enumerate() {
            let (dst, src) = workspace.get_dst_src(self.dst, *src)?;
            Src::apply_to(src, dst, IntervalFill(i));
        }

        Ok(())
    }
}
