mod rtree;

pub use rtree::*;

use std::ops::Range;
use temper_data::biomes::Biome;

#[derive(Clone)]
pub struct BiomeParameters {
    pub biome: &'static Biome,
    continentalness: Range<f64>,
    erosion: Range<f64>,
    humidity: Range<f64>,
    temperature: Range<f64>,
    weirdness: Range<f64>,
    depth: Range<f64>,
    offset: Range<f64>,
}

impl BiomeParameters {
    fn as_param_list(&self) -> ParameterSpace {
        [
            quantize(self.temperature.start)..quantize(self.temperature.end),
            quantize(self.humidity.start)..quantize(self.humidity.end),
            quantize(self.continentalness.start)..quantize(self.continentalness.end),
            quantize(self.erosion.start)..quantize(self.erosion.end),
            quantize(self.depth.start)..quantize(self.depth.end),
            quantize(self.weirdness.start)..quantize(self.weirdness.end),
            quantize(self.offset.start)..quantize(self.offset.end),
        ]
    }
}

include!(concat!(env!("OUT_DIR"), "/biome_params.rs"));

pub fn quantize(v: f64) -> i64 {
    (v * 10000.0) as i64
}
