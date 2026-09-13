use std::ops::Range;
use rstar::{RTreeObject, AABB, PointDistance, Envelope, Point};
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

impl RTreeObject for BiomeParameters {
    type Envelope = AABB<[i64; 6]>;

    fn envelope(&self) -> Self::Envelope {
        let min = [
            quantize(self.continentalness.start),
            quantize(self.erosion.start),
            quantize(self.humidity.start),
            quantize(self.temperature.start),
            quantize(self.weirdness.start),
            quantize(self.depth.start),
        ];
        let max = [
            quantize(self.continentalness.end),
            quantize(self.erosion.end),
            quantize(self.humidity.end),
            quantize(self.temperature.end),
            quantize(self.weirdness.end),
            quantize(self.depth.end),
        ];

        AABB::from_corners(min, max)
    }
}

impl PointDistance for BiomeParameters {
    fn distance_2(&self, point: &<Self::Envelope as Envelope>::Point) -> <<Self::Envelope as Envelope>::Point as Point>::Scalar {
        let mut squared_distance = 0;

        for (i, (min, max)) in self.as_param_list().iter().enumerate() {
            let p = point[i];

            if p < *min {
                squared_distance += (min - p).pow(2);
            } else if p > *max {
                squared_distance += (p - max).pow(2);
            }
        }

        squared_distance
    }
}

impl BiomeParameters {
    fn as_param_list(&self) -> [(i64, i64); 6] {
        [
            (quantize(self.continentalness.start), quantize(self.continentalness.end)),
            (quantize(self.erosion.start), quantize(self.erosion.end)),
            (quantize(self.humidity.start), quantize(self.humidity.end)),
            (quantize(self.temperature.start), quantize(self.temperature.end)),
            (quantize(self.weirdness.start), quantize(self.weirdness.end)),
            (quantize(self.depth.start), quantize(self.depth.end)),
        ]
    }
}

include!(concat!(env!("OUT_DIR"), "/biome_params.rs"));

pub fn quantize(v: f64) -> i64 {
    (v * 10000.0) as i64
}

pub fn unquantize(v: i64) -> f64 {
    (v as f64) / 10000.0
}
