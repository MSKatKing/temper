use crate::NormalGenerator;
use gen_core::{GenerationError, StageInput};
use quick_noise::{Fbm, Perlin};

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

        let grid_3d = quick_noise::Grid::<3>::new(16, 384, 16);

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
