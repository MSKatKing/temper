pub mod cpu;
mod ir;

pub use ir::*;

#[inline]
pub fn deserialize_density_function(json: &str) -> serde_json::Result<DensityFunction> {
    serde_json::from_str(json)
}
