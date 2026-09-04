use crate::cpu::buffer::{Buffer, BufferId, BufferType, Flat, FlatCell, Full, Interpolated};
use crate::cpu::compiler::{CompiledDensityFunction, ToAnyBufferId};
use crate::cpu::runtime::{DensityError, DensityResult, Operation};
use temper_core::pos::{BlockPos, ChunkBlockPos, ChunkPos};

pub trait WorkspaceStorable: BufferType {
    fn get_buffer<'a>(
        workspace: &'a Workspace,
        id: BufferId<Self>,
    ) -> DensityResult<&'a Buffer<Self>>;
    fn get_buffer_mut<'a>(
        workspace: &'a mut Workspace,
        id: BufferId<Self>,
    ) -> DensityResult<&'a mut Buffer<Self>>;
}

pub trait GetDstSrc<Dst: BufferType>: BufferType {
    fn get_dst_src<'a>(
        workspace: &'a mut Workspace,
        dst: BufferId<Dst>,
        src: BufferId<Self>,
    ) -> DensityResult<(&'a mut Buffer<Dst>, &'a Buffer<Self>)>;

    fn get_dst_src_2<'a>(
        workspace: &'a mut Workspace,
        dst: BufferId<Dst>,
        src0: BufferId<Self>,
        src1: BufferId<Self>,
    ) -> DensityResult<(&'a mut Buffer<Dst>, &'a Buffer<Self>, &'a Buffer<Self>)>;

    fn get_dst_src_3<'a>(
        workspace: &'a mut Workspace,
        dst: BufferId<Dst>,
        src0: BufferId<Self>,
        src1: BufferId<Self>,
        src2: BufferId<Self>,
    ) -> DensityResult<(
        &'a mut Buffer<Dst>,
        &'a Buffer<Self>,
        &'a Buffer<Self>,
        &'a Buffer<Self>,
    )>;
}

macro_rules! impl_workspace_field {
    ($ty:ty => $field:ident, [$($ty_b:ty => $field_b:ident),* $(,)?]) => {
        impl WorkspaceStorable for $ty {
            fn get_buffer<'a>(workspace: &'a Workspace, id: BufferId<$ty>) -> DensityResult<&'a Buffer<$ty>> {
                workspace.$field.get(id.idx()).ok_or(DensityError::MissingBuffer(<$ty as ToAnyBufferId>::convert_to_any(id)))
            }

            fn get_buffer_mut<'a>(workspace: &'a mut Workspace, id: BufferId<$ty>) -> DensityResult<&'a mut Buffer<$ty>> {
                workspace.$field.get_mut(id.idx()).ok_or(DensityError::MissingBuffer(<$ty as ToAnyBufferId>::convert_to_any(id)))
            }
        }

        impl GetDstSrc<$ty> for $ty {
            fn get_dst_src<'a>(workspace: &'a mut Workspace, dst: BufferId<$ty>, src: BufferId<$ty>) -> DensityResult<(&'a mut Buffer<$ty>, &'a Buffer<$ty>)> {
                if dst.idx() == src.idx() {
                    return Err(DensityError::DstSrcSameBuffer(<$ty as ToAnyBufferId>::convert_to_any(dst)));
                }

                split_two(&mut workspace.$field, dst.idx(), src.idx()).ok_or(DensityError::InvalidDstSrc(<$ty as ToAnyBufferId>::convert_to_any(dst), <$ty as ToAnyBufferId>::convert_to_any(src)))
            }

            fn get_dst_src_2<'a>(workspace: &'a mut Workspace, dst: BufferId<$ty>, src0: BufferId<Self>, src1: BufferId<Self>) -> DensityResult<(&'a mut Buffer<$ty>, &'a Buffer<Self>, &'a Buffer<Self>)> {
                split_three(&mut workspace.$field, dst.idx(), src0.idx(), src1.idx()).ok_or(DensityError::InvalidDstSrc(<$ty as ToAnyBufferId>::convert_to_any(dst), <$ty as ToAnyBufferId>::convert_to_any(src0)))
            }

            fn get_dst_src_3<'a>(workspace: &'a mut Workspace, dst: BufferId<$ty>, src0: BufferId<Self>, src1: BufferId<Self>, src2: BufferId<Self>) -> DensityResult<(&'a mut Buffer<$ty>, &'a Buffer<Self>, &'a Buffer<Self>, &'a Buffer<Self>)> {
                split_four(&mut workspace.$field, dst.idx(), src0.idx(), src1.idx(), src2.idx()).ok_or(DensityError::InvalidDstSrc(<$ty as ToAnyBufferId>::convert_to_any(dst), <$ty as ToAnyBufferId>::convert_to_any(src0)))
            }
        }

        $(
            impl GetDstSrc<$ty> for $ty_b {
                fn get_dst_src<'a>(workspace: &'a mut Workspace, dst: BufferId<$ty>, src: BufferId<$ty_b>) -> DensityResult<(&'a mut Buffer<$ty>, &'a Buffer<$ty_b>)> {
                    workspace.$field.get_mut(dst.idx()).and_then(|dst| Some((dst, workspace.$field_b.get(src.idx())?))).ok_or(DensityError::InvalidDstSrc(<$ty as ToAnyBufferId>::convert_to_any(dst), <$ty_b as ToAnyBufferId>::convert_to_any(src)))
                }

                fn get_dst_src_2<'a>(workspace: &'a mut Workspace, dst: BufferId<$ty>, src0: BufferId<Self>, src1: BufferId<Self>) -> DensityResult<(&'a mut Buffer<$ty>, &'a Buffer<Self>, &'a Buffer<Self>)> {
                    workspace.$field.get_mut(dst.idx())
                        .and_then(|dst| {
                            let (a, b) = split_two(&mut workspace.$field_b, src0.idx(), src1.idx())?;
                            Some((dst, &*a, b))
                        }).ok_or(DensityError::InvalidDstSrc(<$ty as ToAnyBufferId>::convert_to_any(dst), <$ty_b as ToAnyBufferId>::convert_to_any(src0)))
                }

                fn get_dst_src_3<'a>(workspace: &'a mut Workspace, dst: BufferId<$ty>, src0: BufferId<Self>, src1: BufferId<Self>, src2: BufferId<Self>) -> DensityResult<(&'a mut Buffer<$ty>, &'a Buffer<Self>, &'a Buffer<Self>, &'a Buffer<Self>)> {
                    workspace.$field.get_mut(dst.idx())
                        .and_then(|dst| {
                            let (a, b, c) = split_three(&mut workspace.$field_b, src0.idx(), src1.idx(), src2.idx())?;
                            Some((dst, &*a, b, c))
                        }).ok_or(DensityError::InvalidDstSrc(<$ty as ToAnyBufferId>::convert_to_any(dst), <$ty_b as ToAnyBufferId>::convert_to_any(src0)))
                }
            }
        )*
    };
}

impl_workspace_field!(
    Full => full,
    [
        Interpolated => interpolated,
        Flat => flat,
        FlatCell => flat_cell,
    ]
);

impl_workspace_field!(
    Interpolated => interpolated,
    [
        Full => full,
        Flat => flat,
        FlatCell => flat_cell,
    ]
);

impl_workspace_field!(
    Flat => flat,
    [
        Full => full,
        Interpolated => interpolated,
        FlatCell => flat_cell,
    ]
);

impl_workspace_field!(
    FlatCell => flat_cell,
    [
        Full => full,
        Interpolated => interpolated,
        Flat => flat,
    ]
);

pub struct Workspace<'func> {
    full: Vec<Buffer<Full>>,
    flat: Vec<Buffer<Flat>>,
    flat_cell: Vec<Buffer<FlatCell>>,
    interpolated: Vec<Buffer<Interpolated>>,
    pub operations: &'func [Box<dyn Operation>],

    pub current_pos: ChunkPos,
}

impl Workspace<'_> {
    pub fn new(density_function: &CompiledDensityFunction) -> Workspace<'_> {
        Workspace {
            full: (0..density_function.full_buffer_count)
                .map(|_| Buffer::new())
                .collect(),
            interpolated: (0..density_function.interpolated_buffer_count)
                .map(|_| Buffer::new())
                .collect(),
            flat: (0..density_function.flat_buffer_count)
                .map(|_| Buffer::new())
                .collect(),
            flat_cell: (0..density_function.flat_cell_buffer_count)
                .map(|_| Buffer::new())
                .collect(),
            operations: &density_function.ops,
            current_pos: ChunkPos::new(0, 0),
        }
    }

    pub fn set_pos(&mut self, pos: ChunkPos) {
        self.current_pos = pos;
    }

    pub fn out(&self) -> &Buffer<Full> {
        &self.full[0]
    }

    pub fn get_buffer<T: WorkspaceStorable>(&self, id: BufferId<T>) -> DensityResult<&Buffer<T>> {
        T::get_buffer(self, id)
    }

    pub fn get_buffer_mut<T: WorkspaceStorable>(
        &mut self,
        id: BufferId<T>,
    ) -> DensityResult<&mut Buffer<T>> {
        T::get_buffer_mut(self, id)
    }

    pub fn get_dst_src<Dst: BufferType, Src: BufferType + GetDstSrc<Dst>>(
        &mut self,
        dst: BufferId<Dst>,
        src: BufferId<Src>,
    ) -> DensityResult<(&mut Buffer<Dst>, &Buffer<Src>)> {
        Src::get_dst_src(self, dst, src)
    }

    pub fn get_global_pos(&self, local_pos: ChunkBlockPos) -> BlockPos {
        self.current_pos.chunk_block(local_pos)
    }

    pub fn execute(&mut self) -> DensityResult<()> {
        if is_x86_feature_detected!("avx2") {
            for operation in self.operations {
                // SAFETY: avx2 is enabled if we are here
                unsafe {
                    operation.execute_simd(self)?;
                }
            }
        } else {
            for operation in self.operations {
                operation.execute(self)?;
            }
        }

        Ok(())
    }
}

fn split_two<T>(slice: &mut [T], a: usize, b: usize) -> Option<(&mut T, &T)> {
    if a == b {
        return None;
    }

    if a < b {
        let (left, right) = slice.split_at_mut(b);
        Some((&mut left[a], &right[0]))
    } else {
        let (left, right) = slice.split_at_mut(a);
        Some((&mut right[0], &mut left[0]))
    }
}

fn split_three<T>(slice: &mut [T], a: usize, b: usize, c: usize) -> Option<(&mut T, &T, &T)> {
    if a == b || b == c || a == c || a >= slice.len() || b >= slice.len() || c >= slice.len() {
        return None;
    }

    let mut indices = [(a, 0), (b, 1), (c, 2)];
    indices.sort_by(|a, b| a.0.cmp(&b.0));

    let [(i0, t0), (i1, t1), (i2, t2)] = indices;

    let (s0, rem) = slice.split_at_mut(i1);
    let (s1, rem) = rem.split_at_mut(i2 - i1);

    let r0 = &mut s0[i0];
    let r1 = &mut s1[0];
    let r2 = &mut rem[0];

    let mut splits = [None, None, None];
    for (refs, ref_num) in [(r0, t0), (r1, t1), (r2, t2)] {
        splits[ref_num] = Some(refs)
    }

    Some((
        splits[0].take().unwrap(),
        &*splits[1].take().unwrap(),
        &*splits[2].take().unwrap(),
    ))
}

fn split_four<T>(
    slice: &mut [T],
    a: usize,
    b: usize,
    c: usize,
    d: usize,
) -> Option<(&mut T, &T, &T, &T)> {
    if a == b || b == c || a == c || a >= slice.len() || b >= slice.len() || c >= slice.len() {
        return None;
    }

    if a == d || b == d || c == d || d >= slice.len() {
        return None;
    }

    let mut indices = [(a, 0), (b, 1), (c, 2), (d, 3)];
    indices.sort_by(|a, b| a.0.cmp(&b.0));

    let [(i0, t0), (i1, t1), (i2, t2), (i3, t3)] = indices;

    let (s0, rem) = slice.split_at_mut(i1);
    let (s1, rem) = rem.split_at_mut(i2 - i1);
    let (s2, rem) = rem.split_at_mut(i3 - i2);

    let r0 = &mut s0[i0];
    let r1 = &mut s1[0];
    let r2 = &mut s2[0];
    let r3 = &mut rem[0];

    let mut splits = [None, None, None, None];
    for (refs, ref_num) in [(r0, t0), (r1, t1), (r2, t2), (r3, t3)] {
        splits[ref_num] = Some(refs)
    }

    Some((
        splits[0].take().unwrap(),
        &*splits[1].take().unwrap(),
        &*splits[2].take().unwrap(),
        &*splits[3].take().unwrap(),
    ))
}
