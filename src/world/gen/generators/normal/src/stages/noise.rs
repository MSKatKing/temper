use crate::NormalGenerator;
use gen_core::{GenerationError, StageInput};
use quick_noise::{Fbm, Perlin};

const MIN_Y: i32 = -64;

impl NormalGenerator {
    pub(crate) fn generate_noises(&self, input: StageInput<'_>) -> Result<(), GenerationError> {
        let grid_2d = quick_noise::Grid::<2>::new(16, 16)
            .grid_position(input.pos.x(), input.pos.z())
            .seed(self.seed as i64);

        let mut end_grid_2d = [0f32; 256];

        // Continentalness
        grid_2d
            .builder::<Fbm, Perlin>()
            // Hardcoding seeds here is fine since it is combined with the grid/world seed and so each
            // noise type will be different from each other but the same when using the same world seed
            .seed(1)
            .frequency(1.0 / 2048.0)
            .octaves(4)
            .lacunarity(2.0)
            .persistence(0.45)
            .fill(end_grid_2d.as_mut_slice());

        end_grid_2d.chunks(16).enumerate().for_each(|(idx, chunk)| {
            assert!(idx < 16);
            assert_eq!(chunk.len(), 16);
            input.target.noise.continentalness[idx] =
                chunk.try_into().expect("Chunk length mismatch");
        });

        // Erosion
        grid_2d
            .builder::<Fbm, Perlin>()
            .seed(2)
            .frequency(1.0 / 600.0)
            .octaves(4)
            .lacunarity(2.0)
            .persistence(0.45)
            .fill(end_grid_2d.as_mut_slice());

        end_grid_2d.chunks(16).enumerate().for_each(|(idx, chunk)| {
            assert!(idx < 16);
            assert_eq!(chunk.len(), 16);
            input.target.noise.erosion[idx] = chunk.try_into().expect("Chunk length mismatch");
        });

        // Weirdness
        grid_2d
            .builder::<Fbm, Perlin>()
            .seed(3)
            .frequency(1.0 / 400.0)
            .octaves(3)
            .lacunarity(2.0)
            .persistence(0.5)
            .fill(end_grid_2d.as_mut_slice());

        end_grid_2d.chunks(16).enumerate().for_each(|(idx, chunk)| {
            assert!(idx < 16);
            assert_eq!(chunk.len(), 16);
            input.target.noise.weirdness[idx] = chunk.try_into().expect("Chunk length mismatch");
        });

        // Temperature
        grid_2d
            .builder::<Fbm, Perlin>()
            .seed(4)
            .frequency(1.0 / 1200.0)
            .octaves(3)
            .lacunarity(2.0)
            .persistence(0.4)
            .fill(end_grid_2d.as_mut_slice());

        end_grid_2d.chunks(16).enumerate().for_each(|(idx, chunk)| {
            assert!(idx < 16);
            assert_eq!(chunk.len(), 16);
            input.target.noise.temperature[idx] = chunk.try_into().expect("Chunk length mismatch");
        });

        // Humidity
        grid_2d
            .builder::<Fbm, Perlin>()
            .seed(5)
            .frequency(1.0 / 800.0)
            .octaves(3)
            .lacunarity(2.0)
            .persistence(0.45)
            .fill(end_grid_2d.as_mut_slice());

        end_grid_2d.chunks(16).enumerate().for_each(|(idx, chunk)| {
            assert!(idx < 16);
            assert_eq!(chunk.len(), 16);
            input.target.noise.humidity[idx] = chunk.try_into().expect("Chunk length mismatch");
        });

        // Jagged
        grid_2d
            .builder::<Fbm, Perlin>()
            .seed(6)
            .frequency(1.0 / 80.0)
            .octaves(3)
            .lacunarity(2.0)
            .persistence(0.5)
            .fill(end_grid_2d.as_mut_slice());

        end_grid_2d.chunks(16).enumerate().for_each(|(idx, chunk)| {
            assert!(idx < 16);
            assert_eq!(chunk.len(), 16);
            input.target.noise.jaggedness[idx] = chunk.try_into().expect("Chunk length mismatch");
        });

        let grid_3d = quick_noise::Grid::<3>::new(16, 384, 16)
            .sample_position(input.pos.pos.x, MIN_Y, input.pos.pos.y)
            .seed(self.seed as i64);

        grid_3d
            .builder::<Fbm, Perlin>()
            .seed(7)
            .frequency(1.0 / 80.0)
            .scaling(1.0, 0.5, 1.0)
            .octaves(3)
            .lacunarity(2.0)
            .persistence(0.5)
            .fill(input.target.noise.base3d.as_mut_slice());

        grid_3d
            .builder::<Fbm, Perlin>()
            .seed(7)
            .frequency(1.0 / 100.0)
            .scaling(1.0, 0.6, 1.0)
            .octaves(3)
            .lacunarity(2.0)
            .persistence(0.5)
            .fill(input.target.noise.cheese_caves.as_mut_slice());

        grid_3d
            .builder::<Fbm, Perlin>()
            .seed(7)
            .frequency(1.0 / 50.0)
            .scaling(1.0, 0.7, 1.0)
            .octaves(2)
            .lacunarity(2.0)
            .persistence(0.45)
            .fill(input.target.noise.spaghetti_caves.as_mut_slice());

        grid_3d
            .builder::<Fbm, Perlin>()
            .seed(7)
            .frequency(1.0 / 20.0)
            .scaling(1.0, 0.75, 1.0)
            .octaves(2)
            .lacunarity(2.0)
            .persistence(0.45)
            .fill(input.target.noise.spaghetti_caves.as_mut_slice());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use gen_core::{GenStage, StageInput, StageNeighborhood};
    use temper_core::pos::ChunkPos;
    use temper_world_format::Chunk;

    use super::*;

    fn generate_noise(seed: u64, pos: ChunkPos) -> Chunk {
        let generator = NormalGenerator::new(seed);
        let mut chunk = Chunk::new_empty();

        generator
            .generate_noises(StageInput::new(
                pos,
                GenStage::NOISE,
                &mut chunk,
                StageNeighborhood::empty(),
            ))
            .expect("normal noise generation should succeed");

        chunk
    }

    #[test]
    fn base_3d_noise_uses_chunk_world_position() {
        let origin = generate_noise(10, ChunkPos::new(0, 0));
        let east = generate_noise(10, ChunkPos::new(1, 0));

        assert_ne!(
            origin.noise.base3d, east.noise.base3d,
            "3D terrain noise must not repeat the same volume in every chunk",
        );
    }
}
