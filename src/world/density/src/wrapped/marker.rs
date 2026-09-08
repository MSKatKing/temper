use crate::wrapped::WrappedDensityFunction;
use temper_core::pos::BlockPos;

pub struct CacheData {
    last_pos: BlockPos,
    last_value: f64,
}

impl Default for CacheData {
    fn default() -> Self {
        Self {
            last_pos: BlockPos::of(i32::MAX, i32::MAX, i32::MAX),
            last_value: 0.0,
        }
    }
}

pub enum MarkerDensityFunction {
    CacheAllInCell,
    Interpolated,
    CacheOnce(CacheData),
    Cache2d(CacheData),
    FlatCache(CacheData),
}

impl MarkerDensityFunction {
    pub fn execute(&mut self, inner: usize, func: &WrappedDensityFunction) -> f64 {
        match self {
            Self::CacheAllInCell => func.execute_inner(inner),
            Self::Interpolated => func.execute_inner(inner),
            Self::CacheOnce(data) => {
                if func.pos != data.last_pos {
                    data.last_pos = func.pos;
                    data.last_value = func.execute_inner(inner);
                }

                data.last_value
            }
            Self::Cache2d(data) => {
                let pos = BlockPos::of(
                    func.pos.pos.x,
                    0,
                    func.pos.pos.z,
                );

                if pos != data.last_pos {
                    data.last_pos = pos;
                    data.last_value = func.execute_inner(inner);
                }

                data.last_value
            }
            Self::FlatCache(data) => {
                let pos = BlockPos::of(
                    func.pos.pos.x & !3,
                    0,
                    func.pos.pos.z & !3,
                );

                if pos != data.last_pos {
                    data.last_pos = pos;
                    data.last_value = func.execute_inner(inner);
                }

                data.last_value
            }
        }
    }
}
