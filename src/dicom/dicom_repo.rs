use std::collections::HashMap;
use super::patient::Patient;
use super::studyset::StudySet;
use super::ct_image::CTImage;
use super::image_series::ImageSeries;

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
        self.image_series
            .insert(series.uid.clone(), series);
    }

    // Add or update a CT image
    pub fn add_ct_image(&mut self, image: CTImage) {
        self.ct_images.insert(image.uid.clone(), image);
    }

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
                    result.push_str(&format!(
                        "    ImageSeries: {}\n",
                        image_series.uid
                    ));
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