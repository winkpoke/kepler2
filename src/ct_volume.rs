use anyhow::Result;
use std::fmt;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;


// Define the CTVolume struct to hold 3D data
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone)]
pub struct CTVolume {
    pub(crate) dimensions: (u16, u16, usize), // (rows, columns, number of slices)
    pub(crate) voxel_spacing: (f32, f32, f32), // (spacing_x, spacing_y, spacing_z)
    // pub(crate) voxel_data: Vec<Vec<i16>>, // 3D voxel data flattened into slices
    pub(crate) voxel_data: Vec<i16>, // 3D voxel data 
}

impl fmt::Debug for CTVolume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CTVolume")
            .field("dimensions", &self.dimensions)
            .field("voxel_spacing", &self.voxel_spacing)
            .field("voxel_data", &format!("{} slices", self.voxel_data.len()))
            .finish()
    }
}

pub trait CTVolumeGenerator {
    fn generate_ct_volume(&self, image_series_id: &str) -> Result<CTVolume>;
}

pub struct Geometry {
    volumes: Vec<CTVolume>,
    base: crate::coordinates::Matrix4x4<f32>,
}