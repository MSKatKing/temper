use crate::index3d;

#[derive(Debug)]
pub struct ChunkDensityField {
    pub min_y: i32,
    pub height: usize,

    /// Same `[z][y][x]` layout as `base_3d`.
    pub values: Vec<f32>,
}

impl ChunkDensityField {
    pub fn get(&self, x: usize, world_y: i32, z: usize) -> f32 {
        let y_index = (world_y - self.min_y) as usize;

        self.values[index3d(x, y_index, z)]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TerrainDensitySettings {
    pub sea_level: f32,

    /// Number of vertical blocks corresponding to one density unit.
    ///
    /// Larger values allow the 3D noise to distort terrain across a larger
    /// vertical distance.
    pub vertical_scale: f32,

    /// Multiplier applied to the full-resolution 3D terrain noise.
    pub base_3d_amplitude: f32,

    /// Number of blocks near the bottom that are blended toward solid.
    pub bottom_slide_size: i32,

    /// Number of blocks near the top that are blended toward air.
    pub top_slide_size: i32,
}

impl Default for TerrainDensitySettings {
    fn default() -> Self {
        Self {
            sea_level: 64.0,
            vertical_scale: 24.0,
            base_3d_amplitude: 0.85,
            bottom_slide_size: 8,
            top_slide_size: 24,
        }
    }
}
