// dicom_ai.rs

#[macro_export]
macro_rules! define_dicom_struct {
    // Main macro to define a struct with fields, types, DICOM tags, and optionality
    ($name:ident, { $(($field_name:ident, $field_type:ty, $dicom_tag:expr, $is_optional:tt)),* $(,)? }) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            // Generate struct fields based on optionality
            $(
                pub $field_name: $crate::define_dicom_struct!(@optional $field_type, $is_optional),
            )*
        }

        impl $name {
            // Constructor function to create struct instances
            pub fn new($($field_name: $crate::define_dicom_struct!(@constructor_type $field_type, $is_optional)),*) -> Self {
                $name {
                    $(
                        $field_name,
                    )*
                }
            }

            // Function to format DICOM tags and their corresponding values into a String
            pub fn format_tags(&self) -> String {
                let mut result = String::new();
                $(
                    $crate::define_dicom_struct!(@to_string $field_name, $field_type, $dicom_tag, $is_optional, self, result);
                )*
                result
            }
        }
    };

    // Helper rule to wrap type in Option if the field is optional
    (@optional $field_type:ty, true) => {
        Option<$field_type>
    };
    (@optional $field_type:ty, false) => {
        $field_type
    };

    // Helper rule for constructor argument types
    (@constructor_type $field_type:ty, true) => {
        Option<$field_type>
    };
    (@constructor_type $field_type:ty, false) => {
        $field_type
    };

    // Helper rule to handle formatting for optional fields
    (@to_string $field_name:ident, $field_type:ty, $dicom_tag:expr, true, $self:ident, $result:ident) => {
        let value = match &$self.$field_name {
            Some(val) => format!("{}: Some({:?})\n", $dicom_tag, val),  // Directly use `val` (which is `String`)
            None => format!("{}: None (Optional)\n", $dicom_tag),
        };
        $result.push_str(value.as_str());
    };

    // Helper rule to handle formatting for mandatory fields
    (@to_string $field_name:ident, $field_type:ty, $dicom_tag:expr, false, $self:ident, $result:ident) => {
        $result.push_str(format!("{}: {:?}\n", $dicom_tag, &$self.$field_name).as_str());  // Borrow `String` as `&str`
    };
}

// Use the macro to define the Patient struct
define_dicom_struct!(Patient, {
    (id, String, "(0010,0020) PatientID", false),           // PatientID is required
    (name, String, "(0010,0010) PatientName", false),       // PatientName is required
    (birthdate, String, "(0010,0030) PatientBirthDate", true),  // PatientBirthDate is optional
    (sex, String, "(0010,0040) PatientSex", true)              // Sex is optional
});

// Use the macro to define the StudySet struct
define_dicom_struct!(StudySet, {
    (id, String, "(0020,0010) StudyID", false),           // StudyID is required
    (date, String, "(0020,000D) StudyDate", false),       // StudyDate is required
    (description, String, "(0008,1030) StudyDescription", true) // StudyDescription is optional
});

// Use the macro to define the ImageSeries struct
define_dicom_struct!(ImageSeries, {
    (id, String, "(0020,000E) SeriesInstanceUID", false),  // SeriesID is required
    (modality, String, "(0008,0060) Modality", false),     // Modality is required
    (description, String, "(0008,103E) SeriesDescription", true) // SeriesDescription is optional
});

// Define the CTImage struct with specific fields, types, DICOM tags, and optionality
define_dicom_struct!(CTImage, {
    (rows, u16, "(0028,0010) Rows", false),                             // Rows (Mandatory)
    (columns, u16, "(0028,0011) Columns", false),                        // Columns (Mandatory)
    (pixel_spacing, (f32, f32), "(0028,0030) PixelSpacing", true),       // PixelSpacing (Optional)
    (image_position_patient, (f32, f32, f32), "(0020,0032) ImagePositionPatient", true), // ImagePositionPatient (Optional)
    (image_orientation_patient, (f32, f32, f32, f32, f32, f32), "(0020,0037) ImageOrientationPatient", true), // ImageOrientationPatient (Optional)
    (pixel_data, Vec<u8>, "(7FE0,0010) PixelData", false),
    (slope, f32, "(0028,1053) RescaleSlope", true),                      // RescaleSlope (Optional)
    (intercept, f32, "(0028,1052) RescaleIntercept", true)               // RescaleIntercept (Optional)                // PixelData (Mandatory)
});

use std::borrow::Cow;
use std::io::Cursor;
// use bytemuck::cast_slice;
use anyhow::Result;
use bytemuck::checked::try_cast_slice;
// use dicom_core::value::{PixelFragmentSequence, Value};
// use dicom_core::{DicomValue, PrimitiveValue, Tag};
use dicom_object::{from_reader, FileDicomObject, InMemDicomObject};

impl Patient {
    // Function to parse the DICOM file and generate the Patient structure
    pub fn from_file(dicom_data: &[u8]) -> Result<Patient> {
        // Parse the DICOM file into a `FileDicomObject`
        let dicom_obj: FileDicomObject<InMemDicomObject> =
            FileDicomObject::from_reader(dicom_data)?;

        // Retrieve required fields
        let id = dicom_obj
            .element_by_name("PatientID")?
            .to_str()?
            .to_string();
        let name = dicom_obj
            .element_by_name("PatientName")?
            .to_str()?
            .to_string();

        // Optional fields
        let birthdate = dicom_obj
            .element_by_name("PatientBirthDate")
            .map(|v| v.to_str().ok().map(|v| v.to_string()))
            .ok()
            .flatten();
        let sex = dicom_obj
            .element_by_name("PatientSex")
            .map(|v| v.to_str().ok().map(|v| v.to_string()))
            .ok()
            .flatten();

        // Return the populated struct
        Ok(Patient {
            id,
            name,
            birthdate,
            sex,
        })
    }
}

impl StudySet {
    // Function to parse the DICOM file and generate the StudySet structure
    pub fn from_file(dicom_data: &[u8]) -> Result<StudySet> {
        // Parse the DICOM file into a `FileDicomObject`
        let dicom_obj: FileDicomObject<InMemDicomObject> =
            FileDicomObject::from_reader(dicom_data)?;

        // Retrieve required fields
        let id = dicom_obj.element_by_name("StudyID")?.to_str()?.to_string();
        let date = dicom_obj
            .element_by_name("StudyDate")?
            .to_str()?
            .to_string();

        // Optional fields
        let description = dicom_obj
            .element_by_name("StudyDescription")
            .map(|v| v.to_str().ok().map(|v| v.to_string()))
            .ok()
            .flatten(); // Flatten the Option<Option<String>> to Option<String>

        // Return the populated struct
        Ok(StudySet {
            id,
            date,
            description,
        })
    }
}

impl ImageSeries {
    // Function to parse the DICOM file and generate the ImageSeries structure
    pub fn from_file(dicom_data: &[u8]) -> Result<ImageSeries> {
        // Parse the DICOM file into a `FileDicomObject`
        let dicom_obj: FileDicomObject<InMemDicomObject> =
            FileDicomObject::from_reader(dicom_data)?;

        // Retrieve the modality to ensure the correct image type (CT)
        let modality = dicom_obj.element_by_name("Modality")?.to_str()?.to_string();

        // Check if the modality is CT, if not, return an error
        if modality != "CT" {
            return Err(anyhow::anyhow!("Expected CT image, but got {} image", modality).into());
        }

        // Retrieve required fields
        let id = dicom_obj
            .element_by_name("SeriesInstanceUID")?
            .to_str()?
            .to_string();
        let modality = dicom_obj.element_by_name("Modality")?.to_str()?.to_string();

        // Optional fields
        let description = dicom_obj
            .element_by_name("SeriesDescription")
            .map(|v| v.to_str().ok().map(|v| v.to_string()))
            .ok()
            .flatten();

        // Return the populated struct
        Ok(ImageSeries {
            id,
            modality,
            description,
        })
    }
}

impl CTImage {
    // Function to parse the DICOM file and generate the CTImage structure
    fn from_file(dicom_data: &[u8]) -> Result<CTImage> {
        // Parse the DICOM file into a `FileDicomObject`
        let dicom_obj: FileDicomObject<InMemDicomObject> =
            FileDicomObject::from_reader(dicom_data)?;

        // Directly retrieve required fields using element_by_name and specific methods like float32_slice
        let rows = dicom_obj.element_by_name("Rows")?.to_int()?; // Rows (u16)
        let columns = dicom_obj.element_by_name("Columns")?.to_int()?; // Columns (u16)

        // Optional fields: use float32_slice for f32 tuples or specific pixel data
        let pixel_spacing = dicom_obj
            .element_by_name("PixelSpacing")?
            .to_multi_float32()
            .ok()
            .map(|v| (v[0], v[1])); // PixelSpacing
        let image_position_patient = dicom_obj
            .element_by_name("ImagePositionPatient")?
            .to_multi_float32()
            .ok()
            .map(|v| (v[0], v[1], v[2])); // ImagePositionPatient
        let image_orientation_patient = dicom_obj
            .element_by_name("ImageOrientationPatient")?
            .to_multi_float32()
            .ok()
            .map(|v| (v[0], v[1], v[2], v[3], v[4], v[5])); // ImageOrientationPatient

        // Extract Pixel Data directly using the PixelData tag
        let pixel_data = dicom_obj.element_by_name("PixelData")?.to_bytes()?.to_vec(); // PixelData

        // Optional fields for Slope and Intercept
        let slope = dicom_obj
            .element_by_name("RescaleSlope")
            .map(|e| e.to_float32().ok())
            .ok()
            .flatten();

        let intercept = dicom_obj
            .element_by_name("RescaleIntercept")
            .map(|e| e.to_float32().ok())
            .ok()
            .flatten();

        // Return the populated struct
        Ok(CTImage {
            rows,
            columns,
            pixel_spacing,
            image_position_patient,
            image_orientation_patient,
            pixel_data,
            slope,
            intercept,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test the Patient struct
    #[test]
    fn test_patient_format_tags() {
        // Creating a Patient instance with both mandatory and optional fields
        let patient = Patient::new(
            String::from("12345"),            // id (mandatory)
            String::from("John Doe"),         // name (mandatory)
            Some(String::from("1990-01-01")), // birthdate (optional)
            None,                             // sex (optional)
        );

        // Capture the returned string from format_tags
        let output_str = patient.format_tags();

        // Check if the output contains the expected tags and values
        assert!(output_str.contains("(0010,0020) PatientID: \"12345\""));
        assert!(output_str.contains("(0010,0010) PatientName: \"John Doe\""));
        assert!(output_str.contains("(0010,0030) PatientBirthDate: Some(\"1990-01-01\")"));
        assert!(output_str.contains("(0010,0040) PatientSex: None (Optional)"));
    }

    // Test the StudySet struct
    #[test]
    fn test_study_set_format_tags() {
        let study_set = StudySet::new(
            String::from("Study123"),      // id (mandatory)
            String::from("2024-11-14"),    // date (mandatory)
            Some(String::from("CT Scan")), // description (optional)
        );

        // Capture the returned string from format_tags
        let output_str = study_set.format_tags();

        // Check if the output contains the expected tags and values
        assert!(output_str.contains("(0020,0010) StudyID: \"Study123\""));
        assert!(output_str.contains("(0020,000D) StudyDate: \"2024-11-14\""));
        assert!(output_str.contains("(0008,1030) StudyDescription: Some(\"CT Scan\")"));
    }

    // Test the ImageSeries struct
    #[test]
    fn test_image_series_format_tags() {
        let image_series = ImageSeries::new(
            String::from("Series123"),      // id (mandatory)
            String::from("CT"),             // modality (mandatory)
            Some(String::from("Brain CT")), // description (optional)
        );

        // Capture the returned string from format_tags
        let output_str = image_series.format_tags();

        // Check if the output contains the expected tags and values
        assert!(output_str.contains("(0020,000E) SeriesInstanceUID: \"Series123\""));
        assert!(output_str.contains("(0008,0060) Modality: \"CT\""));
        assert!(output_str.contains("(0008,103E) SeriesDescription: Some(\"Brain CT\")"));
    }

    fn test_from_file() -> Result<()> {
        // Use `include_bytes!` to include the DICOM file as a byte array
        // Replace this path with your actual DICOM file, ensuring it's in the correct directory
        let dicom_data: &[u8] = include_bytes!("C:\\share\\imrt\\CT.RT001921_1.dcm");

        // Call the from_file function to parse the DICOM data into CTImage

        let ct_image = CTImage::from_file(dicom_data)?;
        // Check the results
        println!("Rows: {}", ct_image.rows);
        println!("Columns: {}", ct_image.columns);

        if let Some(pixel_spacing) = ct_image.pixel_spacing {
            println!("PixelSpacing: {:?}", pixel_spacing);
        } else {
            println!("PixelSpacing: None");
        }

        if let Some(image_position_patient) = ct_image.image_position_patient {
            println!("ImagePositionPatient: {:?}", image_position_patient);
        } else {
            println!("ImagePositionPatient: None");
        }

        if let Some(image_orientation_patient) = ct_image.image_orientation_patient {
            println!("ImageOrientationPatient: {:?}", image_orientation_patient);
        } else {
            println!("ImageOrientationPatient: None");
        }

        println!("PixelData length: {}", ct_image.pixel_data.len());

        // RescaleSlope
        if let Some(slope) = ct_image.slope {
            println!("RescaleSlope: {}", slope);
        } else {
            println!("RescaleSlope: None");
        }

        // RescaleIntercept
        if let Some(intercept) = ct_image.intercept {
            println!("RescaleIntercept: {}", intercept);
        } else {
            println!("RescaleIntercept: None");
        }

        // Test Patient
        let patient = Patient::from_file(dicom_data)?;
        println!("{:?}", patient);

        // Test StudySet
        let study_set = StudySet::from_file(dicom_data)?;
        println!("{:?}", study_set);

        // Test ImageSeries
        let image_series = ImageSeries::from_file(dicom_data)?;
        println!("{:?}", image_series);

        // Return Ok to indicate success
        Ok(())
    }

    // Test case to verify from_file function
    #[test]
    fn test_parse_from_dicom() {
        // Call the test function and check that no error occurs
        assert!(test_from_file().is_ok());
    }
}
