#[derive(Debug, Clone, PartialEq)]
pub struct NoiseParameter {
    pub first_octave: i32,
    pub amplitudes: &'static [f32],
}
impl NoiseParameter {
    pub const AQUIFER_BARRIER: NoiseParameter = NoiseParameter {
        first_octave: -3i32,
        amplitudes: &[1f32],
    };
    pub const AQUIFER_FLUID_LEVEL_FLOODEDNESS: NoiseParameter = NoiseParameter {
        first_octave: -7i32,
        amplitudes: &[1f32],
    };
    pub const AQUIFER_FLUID_LEVEL_SPREAD: NoiseParameter = NoiseParameter {
        first_octave: -5i32,
        amplitudes: &[1f32],
    };
    pub const AQUIFER_LAVA: NoiseParameter = NoiseParameter {
        first_octave: -1i32,
        amplitudes: &[1f32],
    };
    pub const BADLANDS_PILLAR: NoiseParameter = NoiseParameter {
        first_octave: -2i32,
        amplitudes: &[1f32, 1f32, 1f32, 1f32],
    };
    pub const BADLANDS_PILLAR_ROOF: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[1f32],
    };
    pub const BADLANDS_SURFACE: NoiseParameter = NoiseParameter {
        first_octave: -6i32,
        amplitudes: &[1f32, 1f32, 1f32],
    };
    pub const CALCITE: NoiseParameter = NoiseParameter {
        first_octave: -9i32,
        amplitudes: &[1f32, 1f32, 1f32, 1f32],
    };
    pub const CAVE_CHEESE: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[0.5f32, 1f32, 2f32, 1f32, 2f32, 1f32, 0f32, 2f32, 0f32],
    };
    pub const CAVE_ENTRANCE: NoiseParameter = NoiseParameter {
        first_octave: -7i32,
        amplitudes: &[0.4f32, 0.5f32, 1f32],
    };
    pub const CAVE_LAYER: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[1f32],
    };
    pub const CLAY_BANDS_OFFSET: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[1f32],
    };
    pub const CONTINENTALNESS: NoiseParameter = NoiseParameter {
        first_octave: -9i32,
        amplitudes: &[1f32, 1f32, 2f32, 2f32, 2f32, 1f32, 1f32, 1f32, 1f32],
    };
    pub const CONTINENTALNESS_LARGE: NoiseParameter = NoiseParameter {
        first_octave: -11i32,
        amplitudes: &[1f32, 1f32, 2f32, 2f32, 2f32, 1f32, 1f32, 1f32, 1f32],
    };
    pub const EROSION: NoiseParameter = NoiseParameter {
        first_octave: -9i32,
        amplitudes: &[1f32, 1f32, 0f32, 1f32, 1f32],
    };
    pub const EROSION_LARGE: NoiseParameter = NoiseParameter {
        first_octave: -11i32,
        amplitudes: &[1f32, 1f32, 0f32, 1f32, 1f32],
    };
    pub const GRAVEL: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[1f32, 1f32, 1f32, 1f32],
    };
    pub const GRAVEL_LAYER: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[
            1f32,
            1f32,
            1f32,
            1f32,
            0f32,
            0f32,
            0f32,
            0f32,
            0.013333334f32,
        ],
    };
    pub const ICE: NoiseParameter = NoiseParameter {
        first_octave: -4i32,
        amplitudes: &[1f32, 1f32, 1f32, 1f32],
    };
    pub const ICEBERG_PILLAR: NoiseParameter = NoiseParameter {
        first_octave: -6i32,
        amplitudes: &[1f32, 1f32, 1f32, 1f32],
    };
    pub const ICEBERG_PILLAR_ROOF: NoiseParameter = NoiseParameter {
        first_octave: -3i32,
        amplitudes: &[1f32],
    };
    pub const ICEBERG_SURFACE: NoiseParameter = NoiseParameter {
        first_octave: -6i32,
        amplitudes: &[1f32, 1f32, 1f32],
    };
    pub const JAGGED: NoiseParameter = NoiseParameter {
        first_octave: -16i32,
        amplitudes: &[
            1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32,
            1f32, 1f32,
        ],
    };
    pub const NETHER_TEMPERATURE: NoiseParameter = NoiseParameter {
        first_octave: -7i32,
        amplitudes: &[1f32, 1f32],
    };
    pub const NETHER_VEGETATION: NoiseParameter = NoiseParameter {
        first_octave: -7i32,
        amplitudes: &[1f32, 1f32],
    };
    pub const NETHER_STATE_SELECTOR: NoiseParameter = NoiseParameter {
        first_octave: -4i32,
        amplitudes: &[1f32],
    };
    pub const NETHER_WART: NoiseParameter = NoiseParameter {
        first_octave: -3i32,
        amplitudes: &[1f32, 0f32, 0f32, 0.9f32],
    };
    pub const NETHERRACK: NoiseParameter = NoiseParameter {
        first_octave: -3i32,
        amplitudes: &[1f32, 0f32, 0f32, 0.35f32],
    };
    pub const NOODLE: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[1f32],
    };
    pub const NOODLE_RIDGE_A: NoiseParameter = NoiseParameter {
        first_octave: -7i32,
        amplitudes: &[1f32],
    };
    pub const NOODLE_RIDGE_B: NoiseParameter = NoiseParameter {
        first_octave: -7i32,
        amplitudes: &[1f32],
    };
    pub const NOODLE_THICKNESS: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[1f32],
    };
    pub const OFFSET: NoiseParameter = NoiseParameter {
        first_octave: -3i32,
        amplitudes: &[1f32, 1f32, 1f32, 0f32],
    };
    pub const ORE_GAP: NoiseParameter = NoiseParameter {
        first_octave: -5i32,
        amplitudes: &[1f32],
    };
    pub const ORE_VEIN_A: NoiseParameter = NoiseParameter {
        first_octave: -7i32,
        amplitudes: &[1f32],
    };
    pub const ORE_VEIN_B: NoiseParameter = NoiseParameter {
        first_octave: -7i32,
        amplitudes: &[1f32],
    };
    pub const ORE_VEININESS: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[1f32],
    };
    pub const PACKED_ICE: NoiseParameter = NoiseParameter {
        first_octave: -7i32,
        amplitudes: &[1f32, 1f32, 1f32, 1f32],
    };
    pub const PATCH: NoiseParameter = NoiseParameter {
        first_octave: -5i32,
        amplitudes: &[1f32, 0f32, 0f32, 0f32, 0f32, 0.013333334f32],
    };
    pub const PILLAR: NoiseParameter = NoiseParameter {
        first_octave: -7i32,
        amplitudes: &[1f32, 1f32],
    };
    pub const PILLAR_RARENESS: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[1f32],
    };
    pub const PILLAR_THICKNESS: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[1f32],
    };
    pub const POWDER_SNOW: NoiseParameter = NoiseParameter {
        first_octave: -6i32,
        amplitudes: &[1f32, 1f32, 1f32, 1f32],
    };
    pub const RIDGE: NoiseParameter = NoiseParameter {
        first_octave: -7i32,
        amplitudes: &[1f32, 2f32, 1f32, 0f32, 0f32, 0f32],
    };
    pub const SOUL_SAND_LAYER: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[
            1f32,
            1f32,
            1f32,
            1f32,
            0f32,
            0f32,
            0f32,
            0f32,
            0.013333334f32,
        ],
    };
    pub const SPAGHETTI_2D: NoiseParameter = NoiseParameter {
        first_octave: -7i32,
        amplitudes: &[1f32],
    };
    pub const SPAGHETTI_2D_ELEVATION: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[1f32],
    };
    pub const SPAGHETTI_2D_MODULATOR: NoiseParameter = NoiseParameter {
        first_octave: -11i32,
        amplitudes: &[1f32],
    };
    pub const SPAGHETTI_2D_THICKNESS: NoiseParameter = NoiseParameter {
        first_octave: -11i32,
        amplitudes: &[1f32],
    };
    pub const SPAGHETTI_3D_1: NoiseParameter = NoiseParameter {
        first_octave: -7i32,
        amplitudes: &[1f32],
    };
    pub const SPAGHETTI_3D_2: NoiseParameter = NoiseParameter {
        first_octave: -7i32,
        amplitudes: &[1f32],
    };
    pub const SPAGHETTI_3D_RARITY: NoiseParameter = NoiseParameter {
        first_octave: -11i32,
        amplitudes: &[1f32],
    };
    pub const SPAGHETTI_3D_THICKNESS: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[1f32],
    };
    pub const SPAGHETTI_ROUGHNESS: NoiseParameter = NoiseParameter {
        first_octave: -5i32,
        amplitudes: &[1f32],
    };
    pub const SPAGHETTI_ROUGHNESS_MODULATOR: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[1f32],
    };
    pub const SULFUR_CAVE_GRADIENT: NoiseParameter = NoiseParameter {
        first_octave: -5i32,
        amplitudes: &[1f32, 0f32, 1f32],
    };
    pub const SURFACE: NoiseParameter = NoiseParameter {
        first_octave: -6i32,
        amplitudes: &[1f32, 1f32, 1f32],
    };
    pub const SURFACE_SECONDARY: NoiseParameter = NoiseParameter {
        first_octave: -6i32,
        amplitudes: &[1f32, 1f32, 0f32, 1f32],
    };
    pub const SURFACE_SWAMP: NoiseParameter = NoiseParameter {
        first_octave: -2i32,
        amplitudes: &[1f32],
    };
    pub const TEMPERATURE: NoiseParameter = NoiseParameter {
        first_octave: -10i32,
        amplitudes: &[1.5f32, 0f32, 1f32, 0f32, 0f32, 0f32],
    };
    pub const TEMPERATURE_LARGE: NoiseParameter = NoiseParameter {
        first_octave: -12i32,
        amplitudes: &[1.5f32, 0f32, 1f32, 0f32, 0f32, 0f32],
    };
    pub const VEGETATION: NoiseParameter = NoiseParameter {
        first_octave: -8i32,
        amplitudes: &[1f32, 1f32, 0f32, 0f32, 0f32, 0f32],
    };
    pub const VEGETATION_LARGE: NoiseParameter = NoiseParameter {
        first_octave: -10i32,
        amplitudes: &[1f32, 1f32, 0f32, 0f32, 0f32, 0f32],
    };
    pub fn get_by_name<'a>(name: &str) -> Option<&'a Self> {
        match name {
            "minecraft:aquifer_barrier" => Some(&Self::AQUIFER_BARRIER),
            "minecraft:aquifer_fluid_level_floodedness" => {
                Some(&Self::AQUIFER_FLUID_LEVEL_FLOODEDNESS)
            }
            "minecraft:aquifer_fluid_level_spread" => Some(&Self::AQUIFER_FLUID_LEVEL_SPREAD),
            "minecraft:aquifer_lava" => Some(&Self::AQUIFER_LAVA),
            "minecraft:badlands_pillar" => Some(&Self::BADLANDS_PILLAR),
            "minecraft:badlands_pillar_roof" => Some(&Self::BADLANDS_PILLAR_ROOF),
            "minecraft:badlands_surface" => Some(&Self::BADLANDS_SURFACE),
            "minecraft:calcite" => Some(&Self::CALCITE),
            "minecraft:cave_cheese" => Some(&Self::CAVE_CHEESE),
            "minecraft:cave_entrance" => Some(&Self::CAVE_ENTRANCE),
            "minecraft:cave_layer" => Some(&Self::CAVE_LAYER),
            "minecraft:clay_bands_offset" => Some(&Self::CLAY_BANDS_OFFSET),
            "minecraft:continentalness" => Some(&Self::CONTINENTALNESS),
            "minecraft:continentalness_large" => Some(&Self::CONTINENTALNESS_LARGE),
            "minecraft:erosion" => Some(&Self::EROSION),
            "minecraft:erosion_large" => Some(&Self::EROSION_LARGE),
            "minecraft:gravel" => Some(&Self::GRAVEL),
            "minecraft:gravel_layer" => Some(&Self::GRAVEL_LAYER),
            "minecraft:ice" => Some(&Self::ICE),
            "minecraft:iceberg_pillar" => Some(&Self::ICEBERG_PILLAR),
            "minecraft:iceberg_pillar_roof" => Some(&Self::ICEBERG_PILLAR_ROOF),
            "minecraft:iceberg_surface" => Some(&Self::ICEBERG_SURFACE),
            "minecraft:jagged" => Some(&Self::JAGGED),
            "minecraft:nether/temperature" => Some(&Self::NETHER_TEMPERATURE),
            "minecraft:nether/vegetation" => Some(&Self::NETHER_VEGETATION),
            "minecraft:nether_state_selector" => Some(&Self::NETHER_STATE_SELECTOR),
            "minecraft:nether_wart" => Some(&Self::NETHER_WART),
            "minecraft:netherrack" => Some(&Self::NETHERRACK),
            "minecraft:noodle" => Some(&Self::NOODLE),
            "minecraft:noodle_ridge_a" => Some(&Self::NOODLE_RIDGE_A),
            "minecraft:noodle_ridge_b" => Some(&Self::NOODLE_RIDGE_B),
            "minecraft:noodle_thickness" => Some(&Self::NOODLE_THICKNESS),
            "minecraft:offset" => Some(&Self::OFFSET),
            "minecraft:ore_gap" => Some(&Self::ORE_GAP),
            "minecraft:ore_vein_a" => Some(&Self::ORE_VEIN_A),
            "minecraft:ore_vein_b" => Some(&Self::ORE_VEIN_B),
            "minecraft:ore_veininess" => Some(&Self::ORE_VEININESS),
            "minecraft:packed_ice" => Some(&Self::PACKED_ICE),
            "minecraft:patch" => Some(&Self::PATCH),
            "minecraft:pillar" => Some(&Self::PILLAR),
            "minecraft:pillar_rareness" => Some(&Self::PILLAR_RARENESS),
            "minecraft:pillar_thickness" => Some(&Self::PILLAR_THICKNESS),
            "minecraft:powder_snow" => Some(&Self::POWDER_SNOW),
            "minecraft:ridge" => Some(&Self::RIDGE),
            "minecraft:soul_sand_layer" => Some(&Self::SOUL_SAND_LAYER),
            "minecraft:spaghetti_2d" => Some(&Self::SPAGHETTI_2D),
            "minecraft:spaghetti_2d_elevation" => Some(&Self::SPAGHETTI_2D_ELEVATION),
            "minecraft:spaghetti_2d_modulator" => Some(&Self::SPAGHETTI_2D_MODULATOR),
            "minecraft:spaghetti_2d_thickness" => Some(&Self::SPAGHETTI_2D_THICKNESS),
            "minecraft:spaghetti_3d_1" => Some(&Self::SPAGHETTI_3D_1),
            "minecraft:spaghetti_3d_2" => Some(&Self::SPAGHETTI_3D_2),
            "minecraft:spaghetti_3d_rarity" => Some(&Self::SPAGHETTI_3D_RARITY),
            "minecraft:spaghetti_3d_thickness" => Some(&Self::SPAGHETTI_3D_THICKNESS),
            "minecraft:spaghetti_roughness" => Some(&Self::SPAGHETTI_ROUGHNESS),
            "minecraft:spaghetti_roughness_modulator" => Some(&Self::SPAGHETTI_ROUGHNESS_MODULATOR),
            "minecraft:sulfur_cave_gradient" => Some(&Self::SULFUR_CAVE_GRADIENT),
            "minecraft:surface" => Some(&Self::SURFACE),
            "minecraft:surface_secondary" => Some(&Self::SURFACE_SECONDARY),
            "minecraft:surface_swamp" => Some(&Self::SURFACE_SWAMP),
            "minecraft:temperature" => Some(&Self::TEMPERATURE),
            "minecraft:temperature_large" => Some(&Self::TEMPERATURE_LARGE),
            "minecraft:vegetation" => Some(&Self::VEGETATION),
            "minecraft:vegetation_large" => Some(&Self::VEGETATION_LARGE),
            _ => None,
        }
    }
}
