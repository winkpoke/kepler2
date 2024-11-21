use super::dicom_helper::get_value;
use crate::define_dicom_struct;
use anyhow::{anyhow, Result};
use dicom_object::{FileDicomObject, InMemDicomObject};

define_dicom_struct!(CTImage, {
    (uid, String, "(0008,0018) SOPInstanceUID", false),              // Unique identifier for the image
    (series_uid, String, "(0020,000E) SeriesInstanceUID", false),  // SeriesID is required
    (rows, u16, "(0028,0010) Rows", false),                         // Rows (Mandatory)
    (columns, u16, "(0028,0011) Columns", false),                    // Columns (Mandatory)
    (pixel_spacing, (f32, f32), "(0028,0030) PixelSpacing", true),   // PixelSpacing (Optional)
    (slice_thickness, f32, "(0018,0050) SliceThickness", true),      // SliceThickness (Optional)
    (spacing_between_slices, f32, "(0018,0088) SpacingBetweenSlices", true), // SpacingBetweenSlices (Optional)
    (image_position_patient, (f32, f32, f32), "(0020,0032) ImagePositionPatient", true), // ImagePositionPatient (Optional)
    (image_orientation_patient, (f32, f32, f32, f32, f32, f32), "(0020,0037) ImageOrientationPatient", true), // ImageOrientationPatient (Optional)
    (rescale_slope, f32, "(0028,1053) RescaleSlope", true),          // RescaleSlope (Optional)
    (rescale_intercept, f32, "(0028,1052) RescaleIntercept", true),  // RescaleIntercept (Optional)
    (window_center, f32, "(0028,1050) WindowCenter", true),          // WindowCenter (Optional)
    (window_width, f32, "(0028,1051) WindowWidth", true),            // WindowWidth (Optional)
    (pixel_representation, u16, "(0028,0103) PixelRepresentation", false), // Pixel Representation (Mandatory, but important for interpretation)
    (pixel_data, Vec<u8>, "(7FE0,0010) PixelData", false)            // PixelData (Mandatory)
});

pub enum PixelData {
    Unsigned(Vec<u32>),
    Signed(Vec<i32>),
}

impl CTImage {
    // Function to parse the DICOM file and generate the CTImage structure
    pub fn from_bytes(dicom_data: &[u8]) -> Result<CTImage> {
        // Parse the DICOM file into a `FileDicomObject`
        let obj: FileDicomObject<InMemDicomObject> = FileDicomObject::from_reader(dicom_data)?;

        // Populate fields based on DICOM tags
        Ok(CTImage {
            uid: get_value::<String>(&obj, "SOPInstanceUID")
                .ok_or_else(|| anyhow!("Missing SOPInstanceUID"))?,
            series_uid: get_value::<String>(&obj, "SeriesInstanceUID")
                .ok_or_else(|| anyhow!("Missing SeriesInstanceUID"))?,
            rows: get_value::<u16>(&obj, "Rows").ok_or_else(|| anyhow!("Missing Rows"))?,
            columns: get_value::<u16>(&obj, "Columns").ok_or_else(|| anyhow!("Missing Columns"))?,
            pixel_spacing: {
                let spacing = get_value::<String>(&obj, "PixelSpacing");
                spacing.and_then(|v| {
                    let vals: Vec<f32> = v
                        .split('\\')
                        .filter_map(|s| s.parse::<f32>().ok())
                        .collect();
                    if vals.len() == 2 {
                        Some((vals[0], vals[1]))
                    } else {
                        None
                    }
                })
            },
            slice_thickness: get_value::<f32>(&obj, "SliceThickness"),
            spacing_between_slices: get_value::<f32>(&obj, "SpacingBetweenSlices"),
            image_position_patient: {
                let pos = get_value::<String>(&obj, "ImagePositionPatient");
                pos.and_then(|v| {
                    let vals: Vec<f32> = v
                        .split('\\')
                        .filter_map(|s| s.parse::<f32>().ok())
                        .collect();
                    if vals.len() == 3 {
                        Some((vals[0], vals[1], vals[2]))
                    } else {
                        None
                    }
                })
            },
            image_orientation_patient: {
                let orientation = get_value::<String>(&obj, "ImageOrientationPatient");
                orientation.and_then(|v| {
                    let vals: Vec<f32> = v
                        .split('\\')
                        .filter_map(|s| s.parse::<f32>().ok())
                        .collect();
                    if vals.len() == 6 {
                        Some((vals[0], vals[1], vals[2], vals[3], vals[4], vals[5]))
                    } else {
                        None
                    }
                })
            },
            rescale_slope: get_value::<f32>(&obj, "RescaleSlope"),
            rescale_intercept: get_value::<f32>(&obj, "RescaleIntercept"),
            window_center: get_value::<f32>(&obj, "WindowCenter"),
            window_width: get_value::<f32>(&obj, "WindowWidth"),
            pixel_representation: get_value::<u16>(&obj, "PixelRepresentation")
                .ok_or_else(|| anyhow!("Missing PixelRepresentation"))?,
            pixel_data: obj.element_by_name("PixelData")?.to_bytes()?.to_vec(), // Pixel data is mandatory
        })
    }

    pub fn get_pixel_data(self) -> PixelData {
        let rescale_slope = self.rescale_slope.unwrap_or(1.0); // Default slope = 1.0
        let rescale_intercept = self.rescale_intercept.unwrap_or(0.0); // Default intercept = 0.0

        match self.pixel_representation {
            0 => {
                // Unsigned pixel data (e.g., 16-bit unsigned integers)
                let data = self
                    .pixel_data
                    .chunks_exact(2) // Interpret each 2 bytes as one u16 value
                    .map(|chunk| {
                        let value = u16::from_le_bytes([chunk[0], chunk[1]]);
                        let rescaled = rescale_slope * value as f32 + rescale_intercept;
                        rescaled.round() as u32 // Rescale and round to nearest integer
                    })
                    .collect();
                PixelData::Unsigned(data)
            }
            1 => {
                // Signed pixel data (e.g., 16-bit signed integers)
                let data = self
                    .pixel_data
                    .chunks_exact(2) // Interpret each 2 bytes as one i16 value
                    .map(|chunk| {
                        let value = i16::from_le_bytes([chunk[0], chunk[1]]);
                        let rescaled = rescale_slope * value as f32 + rescale_intercept;
                        rescaled.round() as i32 // Rescale and round to nearest integer
                    })
                    .collect();
                PixelData::Signed(data)
            }
            _ => {
                panic!(
                    "Unsupported pixel_representation: {}",
                    self.pixel_representation
                );
            }
        }
    }
}
