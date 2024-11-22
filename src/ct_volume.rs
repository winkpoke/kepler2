use anyhow::{Result, anyhow};
use std::cmp::Ordering;

// Define the CTVolume struct to hold 3D data
pub struct CTVolume {
    pub dimensions: (u16, u16, usize), // (rows, columns, number of slices)
    pub voxel_spacing: (f32, f32, f32), // (spacing_x, spacing_y, spacing_z)
    pub voxel_data: Vec<Vec<i16>>, // 3D voxel data flattened into slices
}

pub trait CTVolumeGenerator {
    fn generate_ct_volume(&self, image_series_id: &str) -> Result<CTVolume>;
}