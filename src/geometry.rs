use crate::{coord::{Base, Matrix4x4}, dicom::DicomRepo, CTImage};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub struct GeometryBuilder<'a> {
    repo: Option<&'a DicomRepo>,
    sorted_image_series: Option<Vec<&'a CTImage>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl <'a> GeometryBuilder<'a> {
    pub fn new() -> Self {
        Self {
            repo: None,
            sorted_image_series: None,
        }
    }

    pub fn dicom_repo(self, repo: &'a DicomRepo) -> Self {
        Self {
            repo: Some(repo),
            ..self
        }
    }

    // pub fn build_uv_base(&mut self, repo: &'a DicomRepo) -> Base<f32> {
    //     if self.sorted_image_series.is_none() {
    //         let mut sorted_image_series: Vec<&'a CTImage> = repo.get_images_by_series("1.2.392.200036.9116.2.5.1.144.3437232930.1426478676.964561");
    //         sorted_image_series.sort_by(|a, b| {
    //             let z_a = a.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
    //             let z_b = b.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
    //             z_a.partial_cmp(&z_b).unwrap_or(std::cmp::Ordering::Equal)
    //         });
    //         self.sorted_image_series = Some(sorted_image_series);
    //     }
    //     let images = self.sorted_image_series.as_ref().unwrap();
    //     let ct_image = images[0];
    //     let (ox, oy, oz0) = images[0].image_position_patient.unwrap();
    //     let (_, _, oz1) = images.last().unwrap().image_position_patient.unwrap();
    //     let dz = oz1 - oz0;
    //     let nx = ct_image.rows as f32;
    //     let ny = ct_image.columns as f32;
    //     let space = ct_image.pixel_spacing.unwrap();
    //     let m_uv = [nx*space.0, 0.0,        0.0, ox,
    //                 0.0,        ny*space.1, 0.0, oy,
    //                 0.0,        0.0,        dz,  oz0,
    //                 0.0,        0.0,        0.0, 1.0];
    //     let matrix_uv = Matrix4x4::<f32>::from_array(m_uv);
    //     let base_uv = Base::<f32> {
    //         label: "CT Volume: UV".to_string(),
    //         matrix: matrix_uv,
    //     };
    //     return base_uv;

    // }

    pub fn build_uv_base(repo: &'a DicomRepo) -> Base<f32> {
        let mut sorted_image_series: Vec<&'a CTImage> = repo.get_images_by_series("1.2.392.200036.9116.2.5.1.144.3437232930.1426478676.964561");
        sorted_image_series.sort_by(|a, b| {
            let z_a = a.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
            let z_b = b.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
            z_a.partial_cmp(&z_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        // self.sorted_image_series = Some(sorted_image_series);
        // let images = self.sorted_image_series.as_ref().unwrap();
        let images = sorted_image_series;
        let ct_image = images[0];
        let (ox, oy, oz0) = images[0].image_position_patient.unwrap();
        let (_, _, oz1) = images.last().unwrap().image_position_patient.unwrap();
        let dz = oz1 - oz0;
        let nx = ct_image.rows as f32;
        let ny = ct_image.columns as f32;
        let space = ct_image.pixel_spacing.unwrap();
        let m_uv = [nx*space.0, 0.0,        0.0, ox,
                    0.0,        ny*space.1, 0.0, oy,
                    0.0,        0.0,        dz,  oz0,
                    0.0,        0.0,        0.0, 1.0];
        let matrix_uv = Matrix4x4::<f32>::from_array(m_uv);
        let base_uv = Base::<f32> {
            label: "CT Volume: UV".to_string(),
            matrix: matrix_uv,
        };
        return base_uv;

    }

    pub fn build_transverse_base(repo: &DicomRepo) -> Base<f32> {
        let mut sorted_image_series = repo.get_images_by_series("1.2.392.200036.9116.2.5.1.144.3437232930.1426478676.964561");
        sorted_image_series.sort_by(|a, b| {
            let z_a = a.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
            let z_b = b.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
            z_a.partial_cmp(&z_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        let ct_image = sorted_image_series[0];
        let (ox, oy, oz0) = sorted_image_series[0].image_position_patient.unwrap();
        let (_, _, oz1) = sorted_image_series.last().unwrap().image_position_patient.unwrap();
        let dz = oz1 - oz0;
        let nx = ct_image.rows as f32;
        let ny = ct_image.columns as f32;
        let space = ct_image.pixel_spacing.unwrap();

        let d = f32::max(nx * space.0, ny * space.1);
        let m_screen = [d,    0.0,  0.0,       ox,
                        0.0,  d,    0.0,       oy,
                        0.0,  0.0,  oz1 - oz0, oz0,
                        0.0,  0.0,  0.0, 1.0];
        let matrix_screen = Matrix4x4::<f32>::from_array(m_screen);
        let base_screen = Base::<f32> {
            label: "CT Volume: screen".to_string(),
            matrix: matrix_screen,
        };
        base_screen
    }

    pub fn build_sagittal_base(repo: &DicomRepo) -> Base<f32> {
        let mut sorted_image_series = repo.get_images_by_series("1.2.392.200036.9116.2.5.1.144.3437232930.1426478676.964561");
        sorted_image_series.sort_by(|a, b| {
            let z_a = a.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
            let z_b = b.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
            z_a.partial_cmp(&z_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        let ct_image = sorted_image_series[0];
        let (ox, oy, oz0) = sorted_image_series[0].image_position_patient.unwrap();
        let (_, _, oz1) = sorted_image_series.last().unwrap().image_position_patient.unwrap();
        let dz = oz1 - oz0;
        let nx = ct_image.rows as f32;
        let ny = ct_image.columns as f32;
        let space = ct_image.pixel_spacing.unwrap();

        let d = f32::max(nx * space.0, ny * space.1);
        // let m_screen = [d,    0.0,  0.0, ox,
        //                 0.0,  0.0,  d,   oy,
        //                 0.0,  -d,   0.0, (oz0+oz1)/2.0+d/2.0,
        //                 0.0,  0.0,  0.0, 1.0];

        let m_screen = [d,    0.0,  0.0, ox,
                        0.0,  0.0,  d / 4.0,   (oy + ny*space.1) / 2.0 - d / 2.8,
                        0.0,  -d,   0.0, (oz0+oz1)/2.0+d/2.0,
                        0.0,  0.0,  0.0, 1.0];
        let matrix_screen = Matrix4x4::<f32>::from_array(m_screen);
        let base_screen = Base::<f32> {
            label: "CT Volume: screen".to_string(),
            matrix: matrix_screen,
        };
        base_screen
    }

    pub fn build_coronal_base(repo: &DicomRepo) -> Base<f32> {
        let mut sorted_image_series = repo.get_images_by_series("1.2.392.200036.9116.2.5.1.144.3437232930.1426478676.964561");
        sorted_image_series.sort_by(|a, b| {
            let z_a = a.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
            let z_b = b.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
            z_a.partial_cmp(&z_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        let ct_image = sorted_image_series[0];
        let (ox, oy, oz0) = sorted_image_series[0].image_position_patient.unwrap();
        let (_, _, oz1) = sorted_image_series.last().unwrap().image_position_patient.unwrap();
        let dz = oz1 - oz0;
        let nx = ct_image.rows as f32;
        let ny = ct_image.columns as f32;
        let space = ct_image.pixel_spacing.unwrap();

        let d = f32::max(nx * space.0, ny * space.1);
        let m_screen = [0.0,  0.0,    d, ox,
                          d,  0.0,  0.0, oy,
                        0.0,  -d,   0.0, (oz0 + oz1) / 2.0 + d / 2.0,
                        0.0,  0.0,  0.0, 1.0];
        let m_screen = [0.0,  0.0,    d/2.0, (ox + nx*space.0)/2.0 - d/2.0,
                        d,  0.0,    0.0, oy,
                        0.0,  -d,   0.0, (oz0 + oz1) / 2.0 + d / 2.0,
                        0.0,  0.0,  0.0, 1.0];
        let matrix_screen = Matrix4x4::<f32>::from_array(m_screen);
        let base_screen = Base::<f32> {
            label: "CT Volume: screen".to_string(),
            matrix: matrix_screen,
        };
        println!("d = {}", d);
        base_screen
    }

    pub fn build_oblique_base(repo: &DicomRepo) -> Base<f32> {
        let mut sorted_image_series = repo.get_images_by_series("1.2.392.200036.9116.2.5.1.144.3437232930.1426478676.964561");
        sorted_image_series.sort_by(|a, b| {
            let z_a = a.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
            let z_b = b.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
            z_a.partial_cmp(&z_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        let ct_image = sorted_image_series[0];
        let (ox, oy, oz0) = sorted_image_series[0].image_position_patient.unwrap();
        let (_, _, oz1) = sorted_image_series.last().unwrap().image_position_patient.unwrap();
        let dz = oz1 - oz0;
        let nx = ct_image.rows as f32;
        let ny = ct_image.columns as f32;
        let space = ct_image.pixel_spacing.unwrap();
    
        let d = f32::max(nx * space.0, ny * space.1);
        let m_screen = [0.0,  0.0,    d, ox,
                          d,  0.0,  0.0, oy,
                        0.0,  -d,   0.0, (oz0 + oz1) / 2.0 + d / 2.0,
                        0.0,  0.0,  0.0, 1.0];
        let m_screen = [0.0,  0.0,    d/2.0, (ox + nx*space.0)/2.0 - d/2.0,
                        d,  0.0,    0.0, oy,
                        0.0,  -d,   0.0, (oz0 + oz1) / 2.0 + d / 2.0,
                        0.0,  0.0,  0.0, 1.0];
        let rotation = [ 0.9330,  0.2500, -0.2588, 0.0,
                        -0.1853,  0.9504,  0.2500, 0.0,     
                         0.3085, -0.1853,  0.9330, 0.0,
                            0.0,     0.0,     0.0, 1.0,]; 
        let matrix_screen = Matrix4x4::<f32>::from_array(m_screen);
        let matrix_rot = Matrix4x4::<f32>::from_array(rotation);
        let matrix_screen = matrix_screen * matrix_rot;
        let base_screen = Base::<f32> {
            label: "CT Volume: screen".to_string(),
            matrix: matrix_screen,
        };
        base_screen
    }
}

