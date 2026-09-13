mod json;

use temper_density::BoxedDensityFunction;

pub use json::JsonNoiseRouter;

pub struct NoiseRouter {
    // pub chunk_surface_level: BoxedDensityFunction,
    pub final_density: BoxedDensityFunction,
    // pub barrier: BoxedDensityFunction,
    // pub fluid_level_floodedness: BoxedDensityFunction,
    // pub fluid_level_spread: BoxedDensityFunction,
    // pub lava: BoxedDensityFunction,
    // pub vein_toggle: BoxedDensityFunction,
    // pub vein_ridged: BoxedDensityFunction,
    // pub vein_gap: BoxedDensityFunction,
    pub temperature: BoxedDensityFunction,
    pub vegetation: BoxedDensityFunction,
    pub continents: BoxedDensityFunction,
    pub erosion: BoxedDensityFunction,
    pub depth: BoxedDensityFunction,
    pub ridges: BoxedDensityFunction,
}