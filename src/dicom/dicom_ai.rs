// dicom_ai.rs

use super::{
    ct_image::CTImage, dicom_repo::DicomRepo, image_series::ImageSeries, patient::Patient,
    studyset::StudySet,
};

use std::io::Cursor;
use std::{borrow::Cow, collections::HashMap};
// use bytemuck::cast_slice;
use anyhow::{anyhow, Result};
use bytemuck::checked::try_cast_slice;
// use dicom_core::value::{PixelFragmentSequence, Value};
// use dicom_core::{DicomValue, PrimitiveValue, Tag};
use dicom_object::{from_reader, FileDicomObject, InMemDicomObject};

// #[cfg(test)]
// mod tests {
//     use super::*;

//     // Dummy DICOM data (minimal example)
//     fn get_dummy_dicom_patient() -> Vec<u8> {
//         vec![
//             0x44, 0x49, 0x43, 0x4D, // DICOM magic
//             // Patient info (simplified, would be more complex in real DICOM)
//             0x10, 0x20, 0x00, 0x00, // Patient ID tag
//             b'P', b'a', b't', b'i', b'e', b'n', b't',
//             b'1', // PatientID: Patient1
//                   // other relevant DICOM tags go here
//         ]
//     }

//     fn get_dummy_dicom_study() -> Vec<u8> {
//         vec![
//             0x44, 0x49, 0x43, 0x4D, // DICOM magic
//             // Study info
//             0x20, 0x10, 0x00, 0x00, // Study ID tag
//             b'S', b't', b'u', b'd', b'y',
//             b'1', // StudyID: Study1
//                   // other relevant DICOM tags
//         ]
//     }

//     #[test]
//     fn test_patient_parsing() -> Result<()> {
//         // Test Patient parsing from DICOM file (dummy DICOM data)
//         let dicom_data = get_dummy_dicom_patient();
//         let patient = Patient::from_file(&dicom_data)?;

//         assert_eq!(patient.patient_id, "Patient1");
//         assert_eq!(patient.name, "Patient1");
//         assert!(patient.birthdate.is_none());
//         assert!(patient.sex.is_none());

//         Ok(())
//     }

//     #[test]
//     fn test_study_parsing() -> Result<()> {
//         // Test Study parsing from DICOM file (dummy DICOM data)
//         let dicom_data = get_dummy_dicom_study();
//         let study = StudySet::from_file(&dicom_data)?;

//         assert_eq!(study.study_id, "Study1");
//         assert_eq!(study.uid, "Study1"); // Assuming StudyInstanceUID is the same as StudyID for simplicity
//         assert_eq!(study.patient_id, "Patient1");
//         assert!(study.date.is_empty()); // Dummy DICOM data has no date
//         assert!(study.description.is_none());

//         Ok(())
//     }

//     #[test]
//     fn test_dicom_repo_add_patient() -> Result<()> {
//         let mut repo = DicomRepo::new();
//         let dicom_data = get_dummy_dicom_patient();
//         let patient = Patient::from_file(&dicom_data)?;

//         // Add the patient to the repository
//         repo.add_patient(patient.clone());

//         // Verify the patient has been added
//         // assert_eq!(repo.patients.get(&patient.patient_id), Some(&patient));

//         Ok(())
//     }

//     #[test]
//     fn test_dicom_repo_add_study() -> Result<()> {
//         let mut repo = DicomRepo::new();
//         let dicom_data = get_dummy_dicom_study();
//         let study = StudySet::from_file(&dicom_data)?;

//         // Add the study to the repository
//         repo.add_study(study.clone());

//         // Verify the study has been added
//         // assert_eq!(repo.study_sets.get(&study.uid), Some(&study));

//         Ok(())
//     }

//     #[test]
//     fn test_dicom_repo_query_patient_study() -> Result<()> {
//         let mut repo = DicomRepo::new();
//         let dicom_patient_data = get_dummy_dicom_patient();
//         let patient = Patient::from_file(&dicom_patient_data)?;

//         let dicom_study_data = get_dummy_dicom_study();
//         let study = StudySet::from_file(&dicom_study_data)?;

//         // Add patient and study to the repository
//         repo.add_patient(patient.clone());
//         repo.add_study(study.clone());

//         // Query study by patient ID
//         let studies = repo.get_studies_by_patient(&patient.patient_id);
//         assert_eq!(studies.len(), 1);
//         // assert_eq!(studies[0], &study);

//         Ok(())
//     }

//     #[test]
//     fn test_dicom_repo_query_series_by_study() -> Result<()> {
//         let mut repo = DicomRepo::new();
//         let dicom_study_data = get_dummy_dicom_study();
//         let study = StudySet::from_file(&dicom_study_data)?;

//         let dummy_series = ImageSeries {
//             uid: "Series1".to_string(),
//             study_uid: study.uid.clone(),
//             modality: "CT".to_string(),
//             description: Some("Test series".to_string()),
//         };

//         // Add study and series to the repository
//         repo.add_study(study.clone());
//         repo.add_image_series(dummy_series.clone());

//         // Query series by study UID
//         let series = repo.get_series_by_study(&study.uid);
//         assert_eq!(series.len(), 1);
//         // assert_eq!(series[0], &dummy_series);

//         Ok(())
//     }

//     #[test]
//     fn test_from_file() -> Result<()> {
//         // Use `include_bytes!` to include the DICOM file as a byte array
//         // Replace this path with your actual DICOM file, ensuring it's in the correct directory
//         let dicom_data: &[u8] = include_bytes!("C:\\share\\imrt\\CT.RT001921_1.dcm");

//         // Call the from_file function to parse the DICOM data into CTImage

//         let ct_image = CTImage::from_file(dicom_data);
//         match &ct_image {
//             Ok(_) => println!("reading CT image ok"),
//             Err(err) => eprint!("error: {:?}", err),
//         }
//         let ct_image = ct_image.unwrap();

//         // Check the results
//         println!("id: {}", ct_image.uid);
//         println!("Rows: {}", ct_image.rows);
//         println!("Columns: {}", ct_image.columns);

//         if let Some(pixel_spacing) = ct_image.pixel_spacing {
//             println!("PixelSpacing: {:?}", pixel_spacing);
//         } else {
//             println!("PixelSpacing: None");
//         }

//         if let Some(image_position_patient) = ct_image.image_position_patient {
//             println!("ImagePositionPatient: {:?}", image_position_patient);
//         } else {
//             println!("ImagePositionPatient: None");
//         }

//         if let Some(image_orientation_patient) = ct_image.image_orientation_patient {
//             println!("ImageOrientationPatient: {:?}", image_orientation_patient);
//         } else {
//             println!("ImageOrientationPatient: None");
//         }

//         // println!("PixelData length: {}", ct_image.pixel_data.len());

//         // RescaleSlope
//         if let Some(slope) = ct_image.rescale_slope {
//             println!("RescaleSlope: {}", slope);
//         } else {
//             println!("RescaleSlope: None");
//         }

//         // RescaleIntercept
//         if let Some(intercept) = ct_image.rescale_intercept {
//             println!("RescaleIntercept: {}", intercept);
//         } else {
//             println!("RescaleIntercept: None");
//         }

//         // Test Patient
//         let patient = Patient::from_file(dicom_data)?;
//         println!("{:?}", patient);

//         // Test StudySet
//         let study_set = StudySet::from_file(dicom_data)?;
//         println!("{:?}", study_set);

//         // Test ImageSeries
//         let image_series = ImageSeries::from_file(dicom_data)?;
//         println!("{:?}", image_series);

//         // Return Ok to indicate success
//         Ok(())
//     }

//     // Test case to verify from_file function
//     #[test]
//     fn test_parse_from_dicom() {
//         // Call the test function and check that no error occurs
//         assert!(test_from_file().is_ok());
//     }

//     use std::fs;
//     use std::path::Path;

//     #[test]
//     fn test_generate_clinical_dataset_from_folder() {
//         // Set the directory containing the DICOM files
//         let folder_path = "C:\\share\\imrt\\";

//         // Ensure the folder exists
//         assert!(
//             Path::new(folder_path).exists(),
//             "Test folder does not exist: {}",
//             folder_path
//         );

//         // Initialize an empty DicomRepo
//         let mut dataset = DicomRepo::new();

//         // Read all files in the folder
//         let files = fs::read_dir(folder_path).expect("Failed to read test folder");
//         for entry in files {
//             if let Ok(file) = entry {
//                 let path = file.path();
//                 if path.is_file() {
//                     // Read file data
//                     let dicom_data = fs::read(&path).expect("Failed to read DICOM file");

//                     // Attempt to parse the DICOM data into known structures
//                     if let Ok(patient) = Patient::from_file(&dicom_data) {
//                         dataset.add_patient(patient);
//                     }
//                     if let Ok(study) = StudySet::from_file(&dicom_data) {
//                         dataset.add_study(study);
//                     }
//                     if let Ok(series) = ImageSeries::from_file(&dicom_data) {
//                         dataset.add_image_series(series);
//                     }
//                     if let Ok(ct_image) = CTImage::from_file(&dicom_data) {
//                         dataset.add_ct_image(ct_image);
//                     }
//                 }
//             }
//         }
//         // Output the formatted dataset for debugging
//         let formatted_worklist = dataset.to_string();
//         println!("{}", formatted_worklist);

//         // Basic validation checks
//         assert!(!dataset.patients.is_empty(), "No patients found in dataset");
//         assert!(
//             !dataset.study_sets.is_empty(),
//             "No studies found in dataset"
//         );
//         assert!(
//             !dataset.image_series.is_empty(),
//             "No image series found in dataset"
//         );
//         assert!(
//             !dataset.ct_images.is_empty(),
//             "No CT images found in dataset"
//         );
//     }
// }