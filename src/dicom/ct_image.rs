use anyhow::{Result, anyhow};
use dicom_object::{FileDicomObject, InMemDicomObject};
use crate::define_dicom_struct;
use super::dicom_helper::get_value;


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
    (pixel_representation, u16, "(0028,0103) PixelRepresentation", true), // Pixel Representation (Optional, but important for interpretation)
    (pixel_data, Vec<u8>, "(7FE0,0010) PixelData", false)            // PixelData (Mandatory)
});

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
            pixel_representation: get_value::<u16>(&obj, "PixelRepresentation"),
            pixel_data: obj.element_by_name("PixelData")?.to_bytes()?.to_vec(), // Pixel data is mandatory
        })
    }
}
