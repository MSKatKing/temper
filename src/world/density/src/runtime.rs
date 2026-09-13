use temper_core::pos::BlockPos;
use crate::error::{DensityResult, DensityRuntimeError};
use crate::opcode::DensityOpcode;

pub type DensityStack = Vec<f64>;

pub struct DensityCache {
    last_pos: BlockPos,
    last_value: f64,
}

pub struct DensityRuntime<'compiled> {
    main: &'compiled [DensityOpcode],
    spline_runtimes: &'compiled [Vec<DensityOpcode>],
    stack: DensityStack,
    pos_stack: Vec<BlockPos>,
    caches: Vec<DensityCache>,
}

enum MovementResult {
    Continue,
    JumpTo(usize),
}

impl DensityRuntime<'_> {
    pub fn execute_at(&mut self, pos: BlockPos) -> DensityResult<f64> {
        self.stack.clear();
        self.pos_stack.clear();
        
        self.pos_stack.push(pos);
        
        let mut position = 0;
        while position < self.main.len() {
            match self.main[position].execute(self)? {
                MovementResult::Continue => position += 1,
                MovementResult::JumpTo(jump) => position = jump,
            }
        }
        
        self.stack.pop().ok_or(DensityRuntimeError::StackUnderflow)
    }
    
    fn execute_spline_at(&mut self, index: usize) -> DensityResult<f64> {
        if index >= self.spline_runtimes.len() {
            return Err(DensityRuntimeError::SplineRuntimeOutOfBounds {
                got: index,
                max: self.spline_runtimes.len() - 1,
            })
        }
        
        let runtime = &self.spline_runtimes[index];
        let mut position = 0;
        while position < runtime.len() {
            match runtime[position].execute(self)? {
                MovementResult::Continue => position += 1,
                MovementResult::JumpTo(jump) => position = jump,
            }
        }
        
        self.stack.pop().ok_or(DensityRuntimeError::StackUnderflow)
    }
    
    fn current_pos(&self) -> DensityResult<&BlockPos> {
        self.pos_stack.last().ok_or(DensityRuntimeError::PositionStackUnderflow)
    }
    
    fn push_pos(&mut self, pos: BlockPos) {
        self.pos_stack.push(pos);
    }
    
    fn pop_pos(&mut self) -> DensityResult<BlockPos> {
        self.pos_stack.pop().ok_or(DensityRuntimeError::PositionStackUnderflow)
    }
}

impl DensityOpcode {
    // We always want this inlined, it should only be used above in the runtime execute functions.
    #[inline(always)]
    fn execute(&self, runtime: &mut DensityRuntime) -> DensityResult<MovementResult> {
        todo!()
    }
}
