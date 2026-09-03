use crate::cpu::buffer::{Buffer, BufferId, BufferType, Flat, FlatCell, Full, Interpolated};
use crate::cpu::compiler::CompiledDensityFunction;
use temper_core::pos::{BlockPos, ChunkBlockPos, ChunkPos};
use crate::cpu::runtime::Operation;

pub trait WorkspaceStorable: BufferType {
    fn get_buffer<'a>(workspace: &'a Workspace, id: BufferId<Self>) -> Option<&'a Buffer<Self>>;
    fn get_buffer_mut<'a>(workspace: &'a mut Workspace, id: BufferId<Self>) -> Option<&'a mut Buffer<Self>>;
}

pub trait GetDstSrc<Dst: BufferType>: BufferType {
    fn get_dst_src<'a>(workspace: &'a mut Workspace, dst: BufferId<Dst>, src: BufferId<Self>) -> Option<(&'a mut Buffer<Dst>, &'a Buffer<Self>)>;
}

macro_rules! impl_workspace_field {
    ($ty:ty => $field:ident, [$($ty_b:ty => $field_b:ident),* $(,)?]) => {
        impl WorkspaceStorable for $ty {
            fn get_buffer<'a>(workspace: &'a Workspace, id: BufferId<$ty>) -> Option<&'a Buffer<$ty>> {
                workspace.$field.get(id.idx())
            }

            fn get_buffer_mut<'a>(workspace: &'a mut Workspace, id: BufferId<$ty>) -> Option<&'a mut Buffer<$ty>> {
                workspace.$field.get_mut(id.idx())
            }
        }

        impl GetDstSrc<$ty> for $ty {
            fn get_dst_src<'a>(workspace: &'a mut Workspace, dst: BufferId<$ty>, src: BufferId<$ty>) -> Option<(&'a mut Buffer<$ty>, &'a Buffer<$ty>)> {
                if dst.idx() == src.idx() {
                    return None;
                }

                split_two(&mut workspace.$field, dst.idx(), src.idx())
            }
        }

        $(
            impl GetDstSrc<$ty> for $ty_b {
                fn get_dst_src<'a>(workspace: &'a mut Workspace, dst: BufferId<$ty>, src: BufferId<$ty_b>) -> Option<(&'a mut Buffer<$ty>, &'a Buffer<$ty_b>)> {
                    if dst.idx() == src.idx() {
                        return None;
                    }

                    workspace.$field.get_mut(dst.idx()).and_then(|dst| Some((dst, workspace.$field_b.get(src.idx())?)))
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
            full: (0..density_function.full_buffer_count).map(|_| Buffer::new()).collect(),
            interpolated: (0..density_function.interpolated_buffer_count).map(|_| Buffer::new()).collect(),
            flat: (0..density_function.flat_buffer_count).map(|_| Buffer::new()).collect(),
            flat_cell: (0..density_function.flat_cell_buffer_count).map(|_| Buffer::new()).collect(),
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

    pub fn get_buffer<T: WorkspaceStorable>(&self, id: BufferId<T>) -> Option<&Buffer<T>> {
        T::get_buffer(self, id)
    }

    pub fn get_buffer_mut<T: WorkspaceStorable>(&mut self, id: BufferId<T>) -> Option<&mut Buffer<T>> {
        T::get_buffer_mut(self, id)
    }

    pub fn get_dst_src<Dst: BufferType, Src: BufferType + GetDstSrc<Dst>>(&mut self, dst: BufferId<Dst>, src: BufferId<Src>) -> Option<(&mut Buffer<Dst>, &Buffer<Src>)> {
        Src::get_dst_src(self, dst, src)
    }

    pub fn get_global_pos(&self, local_pos: ChunkBlockPos) -> BlockPos {
        self.current_pos.chunk_block(local_pos)
    }

    #[must_use]
    pub fn execute(&mut self) -> Option<()> {
        for operation in self.operations {
            operation.execute(self)?;
        }
        
        Some(())
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
