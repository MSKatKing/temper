use crate::DensityFunction;
use crate::wrapped;
use crate::wrapped::WrappedDensityFunction;
use temper_core::pos::BlockPos;

#[derive(Debug)]
pub struct CacheAllInCell(pub Box<dyn DensityFunction>);

#[derive(Debug)]
pub struct CacheOnce(pub Box<dyn DensityFunction>);

#[derive(Debug)]
pub struct Cache2d(pub Box<dyn DensityFunction>);

#[derive(Debug)]
pub struct FlatCache(pub Box<dyn DensityFunction>);

#[derive(Debug)]
pub struct Interpolated(pub Box<dyn DensityFunction>);

impl DensityFunction for CacheAllInCell {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(wrapped::CacheAllInCell(self.0.wrap()))
    }
}

impl DensityFunction for CacheOnce {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(wrapped::CacheOnce(self.0.wrap()))
    }
}

impl DensityFunction for Cache2d {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(wrapped::Cache2d {
            inner: self.0.wrap(),
            last_pos: BlockPos::of(i32::MAX, i32::MAX, i32::MAX),
            last_value: 0.0,
        })
    }
}

impl DensityFunction for FlatCache {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(wrapped::FlatCache {
            inner: self.0.wrap(),
            last_pos: BlockPos::of(i32::MAX, i32::MAX, i32::MAX),
            last_value: 0.0,
        })
    }
}

impl DensityFunction for Interpolated {
    fn wrap(&self) -> Box<dyn WrappedDensityFunction + '_> {
        Box::new(wrapped::Interpolated {
            inner: self.0.wrap(),
            last_pos: BlockPos::of(i32::MAX, i32::MAX, i32::MAX),
            data: [0.0; 8],
        })
    }
}
