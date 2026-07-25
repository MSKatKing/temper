use crate::build_terrain_splines::build_terrain_splines;
use crate::density::{ChunkDensityField, TerrainDensitySettings};
use crate::splines::{ColumnShape, TerrainPoint, TerrainSplines};
use crate::{NormalGenerator, index3d, inverse_lerp_clamped, lerp};
use gen_core::{GenerationError, StageInput};
use std::sync::LazyLock;
use temper_core::block_state_id::BlockStateId;
use temper_core::pos::ChunkBlockPos;
use temper_macros::block;
use temper_world_format::{Chunk, ChunkNoises};

static TERRAIN_SPLINES: LazyLock<TerrainSplines> =
    LazyLock::new(|| build_terrain_splines().expect("hard-coded terrain splines must be valid"));

pub const CHUNK_WIDTH: usize = 16;
pub const COLUMN_COUNT: usize = CHUNK_WIDTH * CHUNK_WIDTH; // 256
impl NormalGenerator {
    pub(crate) fn generate_surface(&self, input: StageInput<'_>) -> Result<(), GenerationError> {
        let splines: &TerrainSplines = &TERRAIN_SPLINES;

        let settings = TerrainDensitySettings::default();

        let density = generate_density_field(input.target, splines, -64, 320, settings);

        for y_index in 0..density.height {
            let world_y = density.min_y + y_index as i32;

            for z in 0..CHUNK_WIDTH {
                for x in 0..CHUNK_WIDTH {
                    let index = index3d(x, y_index, z);

                    let value = density.values[index];

                    let block = if value > 0.0 {
                        block!("stone")
                    } else if world_y <= settings.sea_level as i32 {
                        block!("water", {level: 15})
                    } else {
                        block!("air")
                    };

                    input
                        .target
                        .set_block(ChunkBlockPos::new(x as u8, world_y as i16, z as u8), block);
                }
            }
        }
        Ok(())
    }
}

fn build_column_shapes(
    noises: &ChunkNoises,
    splines: &TerrainSplines,
) -> [ColumnShape; COLUMN_COUNT] {
    let mut shapes = [ColumnShape {
        offset_blocks: 0.0,
        factor: 1.0,
        jaggedness_blocks: 0.0,
    }; COLUMN_COUNT];

    for z in 0..CHUNK_WIDTH {
        for x in 0..CHUNK_WIDTH {
            let terrain_point = TerrainPoint::new(
                noises.continentalness[z][x],
                noises.erosion[z][x],
                noises.weirdness[z][x],
            );

            let shape = splines.sample(terrain_point);

            debug_assert!(
                shape.offset_blocks.is_finite(),
                "offset spline returned a non-finite value",
            );

            debug_assert!(
                shape.factor.is_finite() && shape.factor > 0.0,
                "factor spline returned an invalid value",
            );

            debug_assert!(
                shape.jaggedness_blocks.is_finite(),
                "jaggedness spline returned a non-finite value",
            );

            shapes[z * 16 + x] = shape;
        }
    }

    shapes
}

fn half_negative(value: f32) -> f32 {
    if value < 0.0 { value * 0.5 } else { value }
}

fn density_at(
    world_y: i32,
    shape: ColumnShape,
    jagged_noise: f32,
    base_3d_noise: f32,
    settings: TerrainDensitySettings,
) -> f32 {
    // The jaggedness spline says how many blocks of jagged displacement are
    // permitted here. The separate jagged noise supplies the local pattern.
    let jagged_height = shape.jaggedness_blocks * half_negative(jagged_noise);

    // This is the column's broad expected surface elevation.
    let target_surface = settings.sea_level + shape.offset_blocks + jagged_height;

    // Positive below the expected surface, negative above it.
    let vertical_density = (target_surface - world_y as f32) / settings.vertical_scale;

    // Factor controls how strongly the column follows its broad vertical
    // shape. Base 3D noise creates volumetric variation and overhangs.
    vertical_density * shape.factor + base_3d_noise * settings.base_3d_amplitude
}

fn apply_world_slides(
    mut density: f32,
    world_y: i32,
    min_y: i32,
    max_y: i32,
    settings: TerrainDensitySettings,
) -> f32 {
    if settings.bottom_slide_size > 0 {
        let bottom_end = min_y + settings.bottom_slide_size;

        let t = inverse_lerp_clamped(min_y as f32, bottom_end as f32, world_y as f32);

        // At min_y, force positive density.
        density = lerp(1.0, density, t);
    }

    if settings.top_slide_size > 0 {
        let top_start = max_y - settings.top_slide_size;

        let t = inverse_lerp_clamped(top_start as f32, max_y as f32, world_y as f32);

        // At max_y, force negative density.
        density = lerp(density, -1.0, t);
    }

    density
}

pub fn generate_density_field(
    chunk: &Chunk,
    splines: &TerrainSplines,
    min_y: i32,
    max_y: i32,
    settings: TerrainDensitySettings,
) -> ChunkDensityField {
    assert!(max_y > min_y);
    assert!(settings.vertical_scale > 0.0);
    assert!(settings.base_3d_amplitude >= 0.0);

    let height = (max_y - min_y) as usize;
    let expected_3d_length = COLUMN_COUNT * height;

    assert_eq!(
        chunk.noise.base3d.len(),
        expected_3d_length,
        "base 3D noise has the wrong dimensions",
    );

    let column_shapes = build_column_shapes(&chunk.noise, splines);

    let mut values = vec![0.0; expected_3d_length];

    for y_index in 0..height {
        let world_y = min_y + y_index as i32;

        for z in 0..CHUNK_WIDTH {
            for x in 0..CHUNK_WIDTH {
                let density_index = index3d(x, y_index, z);

                let shape = column_shapes[z * 16 + x];

                let density = density_at(
                    world_y,
                    shape,
                    chunk.noise.jaggedness[z][x],
                    chunk.noise.base3d[density_index],
                    settings,
                );

                let density = apply_world_slides(density, world_y, min_y, max_y, settings);

                debug_assert!(
                    density.is_finite(),
                    "density became non-finite at \
                     ({x}, {world_y}, {z})",
                );

                values[density_index] = density;
            }
        }
    }

    ChunkDensityField {
        min_y,
        height,
        values,
    }
}
