use std::arch::x86_64::__m256;
use std::ops::Range;
use crate::cpu::buffer::{BufferApplyTo, BufferId, BufferOperation, BufferType};
use crate::cpu::runtime::{DensityResult, Operation};
use crate::cpu::workspace::{GetDstSrc, Workspace};

struct RangeChoiceA(Range<f32>);
struct RangeChoiceB;

#[derive(Debug)]
pub struct RangeChoice<Dst: BufferType, SrcA: BufferApplyTo<Dst> + GetDstSrc<Dst>, SrcB: BufferApplyTo<Dst> + GetDstSrc<Dst>> {
    pub dst: BufferId<Dst>,
    pub when_in_range: BufferId<SrcA>,
    pub when_out_of_range: BufferId<SrcB>,
    pub range: Range<f32>,
}

impl BufferOperation for RangeChoiceA {
    const READS_DST: bool = true;

    fn scalar(&self, src: f32, dst: f32) -> f32 {
        if self.0.contains(&dst) {
            src
        } else {
            f32::NAN
        }
    }

    unsafe fn simd(&self, src: __m256, dst: __m256) -> __m256 {
        todo!()
    }
}

impl BufferOperation for RangeChoiceB {
    const READS_DST: bool = true;

    fn scalar(&self, src: f32, dst: f32) -> f32 {
        if dst.is_nan() {
            src
        } else {
            dst
        }
    }

    unsafe fn simd(&self, src: __m256, dst: __m256) -> __m256 {
        todo!()
    }
}

impl<Dst: BufferType, SrcA: BufferApplyTo<Dst> + GetDstSrc<Dst>, SrcB: BufferApplyTo<Dst> + GetDstSrc<Dst>> Operation for RangeChoice<Dst, SrcA, SrcB> {
    fn execute(&self, workspace: &mut Workspace) -> DensityResult<()> {
        let (dst, when_in_range) = workspace.get_dst_src(self.dst, self.when_in_range)?;
        SrcA::apply_to(when_in_range, dst, RangeChoiceA(self.range.clone()));
        let (dst, when_out_of_range) = workspace.get_dst_src(self.dst, self.when_out_of_range)?;
        SrcB::apply_to(when_out_of_range, dst, RangeChoiceB);
        Ok(())
    }
}
