use crate::DensityFunction;
use crate::wrapped::{push_op, CacheData, FlattenedDensityFunction, MarkerDensityFunction};

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
    fn wrap<'a>(&'a self, ops: &mut Vec<FlattenedDensityFunction<'a>>) -> usize {
        self.0.wrap(ops)
    }
}

impl DensityFunction for CacheOnce {
    fn wrap<'a>(&'a self, ops: &mut Vec<FlattenedDensityFunction<'a>>) -> usize {
        self.0.wrap(ops)
    }
}

impl DensityFunction for Cache2d {
    fn wrap<'a>(&'a self, ops: &mut Vec<FlattenedDensityFunction<'a >>) -> usize {
        push_op(ops, |ops| {
            FlattenedDensityFunction::Marker {
                op: MarkerDensityFunction::Cache2d(CacheData::default()),
                arg: self.0.wrap(ops)
            }
        })
    }
}

impl DensityFunction for FlatCache {
    fn wrap<'a>(&'a self, ops: &mut Vec<FlattenedDensityFunction<'a>>) -> usize {
        push_op(ops, |ops| {
            FlattenedDensityFunction::Marker {
                op: MarkerDensityFunction::FlatCache(CacheData::default()),
                arg: self.0.wrap(ops)
            }
        })
    }
}

impl DensityFunction for Interpolated {
    fn wrap<'a>(&'a self, ops: &mut Vec<FlattenedDensityFunction<'a>>) -> usize {
        self.0.wrap(ops)
    }
}
