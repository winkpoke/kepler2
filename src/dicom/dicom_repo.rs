use super::ct_image::CTImage;
use super::image_series::ImageSeries;
use super::patient::Patient;
use super::studyset::StudySet;
use crate::ct_volume::{CTVolume, CTVolumeGenerator};
use anyhow::{anyhow, Result};
use std::cmp::Ordering;
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct DicomRepo {
    patients: HashMap<String, Patient>, // Map of patient ID to Patient
    study_sets: HashMap<String, StudySet>, // Map of study ID to StudySet
    image_series: HashMap<String, ImageSeries>, // Map of series ID to ImageSeries
    ct_images: HashMap<String, CTImage>, // Map of image ID to CTImage
}

impl DicomRepo {
    // Constructor
    pub fn new() -> Self {
        DicomRepo {
            patients: HashMap::new(),
            study_sets: HashMap::new(),
            image_series: HashMap::new(),
            ct_images: HashMap::new(),
        }
    }

    // Add or update a patient
    pub fn add_patient(&mut self, patient: Patient) {
        self.patients.insert(patient.patient_id.clone(), patient);
    }

    // Add or update a study
    pub fn add_study(&mut self, study: StudySet) {
        self.study_sets.insert(study.uid.clone(), study);
    }

    // Add or update an image series
    pub fn add_image_series(&mut self, series: ImageSeries) {
        self.image_series.insert(series.uid.clone(), series);
    }

    // Add or update a CT image
    pub fn add_ct_image(&mut self, image: CTImage) {
        self.ct_images.insert(image.uid.clone(), image);
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();

        // Iterate over patients
        for patient in self.patients.values() {
            result.push_str(&format!("Patient: {}\n", patient.name));
            result.push_str(&format!("  ID: {}\n", patient.patient_id));
            result.push_str(&format!("  Birthdate: {:?}\n", patient.birthdate));
            result.push_str(&format!("  Sex: {:?}\n", patient.sex));

            // Find study sets for the patient
            for study_set in self
                .study_sets
                .values()
                .filter(|s| s.patient_id == patient.patient_id)
            {
                result.push_str(&format!("  StudySet: {}\n", study_set.uid));
                result.push_str(&format!("    Date: {}\n", study_set.date));
                result.push_str(&format!("    Description: {:?}\n", study_set.description));

                // Find image series for the study set
                for image_series in self
                    .image_series
                    .values()
                    .filter(|is| is.study_uid == study_set.uid)
                {
                    result.push_str(&format!("    ImageSeries: {}\n", image_series.uid));
                    result.push_str(&format!("      Modality: {}\n", image_series.modality));
                    result.push_str(&format!(
                        "      Description: {:?}\n",
                        image_series.description
                    ));

                    // Find CT images for the image series
                    for ct_image in self
                        .ct_images
                        .values()
                        .filter(|img| img.series_uid == image_series.uid)
                    {
                        result.push_str(&format!("      CTImage: {}\n", ct_image.uid));
                        result.push_str(&format!("        Rows: {}\n", ct_image.rows));
                        result.push_str(&format!("        Columns: {}\n", ct_image.columns));
                        result.push_str(&format!(
                            "        PixelSpacing: {:?}\n",
                            ct_image.pixel_spacing
                        ));
                    }
                }
            }
        }
        result
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl CTVolumeGenerator for DicomRepo {
    fn generate_ct_volume(&self, image_series_id: &str) -> Result<CTVolume> {
        use rayon::prelude::*;
        // Retrieve the ImageSeries by ID
        let series = self
            .image_series
            .get(image_series_id)
            .ok_or_else(|| anyhow!("ImageSeries with ID '{}' not found", image_series_id))?;

        // Collect all CTImages belonging to the ImageSeries
        let mut ct_images: Vec<&CTImage> = self
            .ct_images
            .values()
            .filter(|img| img.series_uid == series.uid)
            .collect();

        if ct_images.is_empty() {
            return Err(anyhow!(
                "No CTImages found for ImageSeries with ID '{}'",
                image_series_id
            ));
        }

        // Sort CTImages by their z-position (third component of ImagePositionPatient)
        ct_images.sort_by(|a, b| {
            let z_a = a.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
            let z_b = b.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
            z_a.partial_cmp(&z_b).unwrap_or(Ordering::Equal)
        });

        // Validate consistency of rows, columns, and retrieve metadata from the first image
        let rows = ct_images[0].rows;
        let columns = ct_images[0].columns;
        let pixel_spacing = ct_images[0]
            .pixel_spacing
            .ok_or_else(|| anyhow!("PixelSpacing is missing in the first CTImage"))?;
        let slice_thickness = ct_images[0].slice_thickness.unwrap_or(1.0);

        // Ensure all images have consistent dimensions
        if !ct_images
            .iter()
            .all(|img| img.rows == rows && img.columns == columns)
        {
            return Err(anyhow!(
                "Inconsistent image dimensions in ImageSeries '{}'",
                series.uid
            ));
        }

        let voxel_spacing = (pixel_spacing.0, pixel_spacing.1, slice_thickness);

        // Collect voxel data from each CTImage concurrently
        let voxel_data: Result<Vec<Vec<i16>>> = ct_images
            .par_iter() // `rayon::iter::ParallelIterator` for parallel processing
            .map(|img| img.get_pixel_data()) // `get_pixel_data` already returns Result<Vec<i16>>
            .collect(); // Collects into a Result<Vec<Vec<i16>>>

        // Return the constructed CTVolume
        Ok(CTVolume {
            dimensions: (rows, columns, ct_images.len()),
            voxel_spacing,
            voxel_data: voxel_data?, // Propagate any error if occurs
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl DicomRepo {
    pub fn get_all_patients(&self) -> Vec<&Patient> {
        self.patients.values().collect()
    }

    // Query patients
    pub fn get_patient(&self, patient_id: &str) -> Option<&Patient> {
        self.patients.get(patient_id)
    }

    // Query studies by patient
    pub fn get_studies_by_patient(&self, patient_id: &str) -> Vec<&StudySet> {
        self.study_sets
            .values()
            .filter(|s| s.patient_id == patient_id)
            .collect()
    }

    // Query series by study
    pub fn get_series_by_study(&self, study_id: &str) -> Vec<&ImageSeries> {
        self.image_series
            .values()
            .filter(|s| s.study_uid == study_id)
            .collect()
    }

    // Query images by series
    pub fn get_images_by_series(&self, series_id: &str) -> Vec<&CTImage> {
        self.ct_images
            .values()
            .filter(|img| img.series_uid == series_id)
            .collect()
    }
}

//------------------------------ WASM Code -------------------------------------

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
impl DicomRepo {
    // console.log(JSON.parse(patientsJson)); // Display the patient data
    pub fn get_all_patients(&self) -> Result<String, String> {
        // Collect all patients into a vector
        let patients: Vec<&Patient> = self.patients.values().collect();

        // Serialize to JSON
        serde_json::to_string(&patients).map_err(|err| err.to_string())
    }

    // Query a specific patient and return it as JSON
    pub fn get_patient(&self, patient_id: &str) -> Result<String, String> {
        self.patients
            .get(patient_id)
            .map(|patient| serde_json::to_string(patient).unwrap()) // Serialize patient to JSON
            .ok_or_else(|| format!("Patient with id {} not found", patient_id))
    }

    // Query studies by patient and return them as JSON
    pub fn get_studies_by_patient(&self, patient_id: &str) -> Result<String, String> {
        let studies: Vec<&StudySet> = self
            .study_sets
            .values()
            .filter(|study| study.patient_id == patient_id)
            .collect();

        serde_json::to_string(&studies).map_err(|err| err.to_string()) // Serialize studies to JSON
    }

    // Query series by study and return them as JSON
    pub fn get_series_by_study(&self, study_id: &str) -> Result<String, String> {
        let series: Vec<&ImageSeries> = self
            .image_series
            .values()
            .filter(|series| series.study_uid == study_id)
            .collect();

        serde_json::to_string(&series).map_err(|err| err.to_string()) // Serialize series to JSON
    }
    
    // Query images by series and return them as JSON
    pub fn get_images_by_series(&self, series_id: &str) -> Result<String, String> {
        let images: Vec<CTImage> = self
            .ct_images
            .values()
            .filter(|image| image.series_uid == series_id)
            .map(|image| {
                let mut cloned_image = image.clone(); // Clone the CTImage
                cloned_image.pixel_data.clear(); // Clear the pixel_data field
                cloned_image
            })
            .collect();

        serde_json::to_string(&images).map_err(|err| err.to_string()) // Serialize images to JSON
    }

    pub async fn generate_ct_volume(&self, image_series_id: &str) -> Result<CTVolume, JsValue> {
        // Retrieve the ImageSeries by ID
        let series = self
            .image_series
            .get(image_series_id)
            .ok_or_else(|| JsValue::from_str(&format!("ImageSeries with ID '{}' not found", image_series_id)))?;
        
        // Collect all CTImages belonging to the ImageSeries
        let mut ct_images: Vec<&CTImage> = self
            .ct_images
            .values()
            .filter(|img| img.series_uid == series.uid)
            .collect();
        
        if ct_images.is_empty() {
            return Err(JsValue::from_str(&format!(
                "No CTImages found for ImageSeries with ID '{}'",
                image_series_id
            )));
        }
        
        // Sort CTImages by their z-position (third component of ImagePositionPatient)
        ct_images.sort_by(|a, b| {
            let z_a = a.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
            let z_b = b.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
            z_a.partial_cmp(&z_b).unwrap_or(Ordering::Equal)
        });
        
        // Validate consistency of rows, columns, and retrieve metadata from the first image
        let rows = ct_images[0].rows;
        let columns = ct_images[0].columns;
        let pixel_spacing = ct_images[0]
            .pixel_spacing
            .ok_or_else(|| JsValue::from_str("PixelSpacing is missing in the first CTImage"))?;
        let slice_thickness = ct_images[0].slice_thickness.unwrap_or(1.0);
        
        // Ensure all images have consistent dimensions
        if !ct_images.iter().all(|img| img.rows == rows && img.columns == columns) {
            return Err(JsValue::from_str(&format!(
                "Inconsistent image dimensions in ImageSeries '{}'",
                series.uid
            )));
        }
        
        let voxel_spacing = (pixel_spacing.0, pixel_spacing.1, slice_thickness);
        
        // Collect voxel data from each CTImage sequentially
        let mut voxel_data = Vec::new();
        for img in &ct_images {
            let pixels = img.get_pixel_data().map_err(|e| JsValue::from_str(&e.to_string()))?;
            voxel_data.push(pixels);
        }
        
        // Return the constructed CTVolume
        Ok(CTVolume {
            dimensions: (rows, columns, ct_images.len()),
            voxel_spacing,
            voxel_data,
        })
    }
    
}
