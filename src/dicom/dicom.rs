use std::collections::HashMap;

use anyhow::Result;
use bytemuck::cast_slice;
use dicom_core::Tag;
use dicom_object::{from_reader, open_file, FileDicomObject, InMemDicomObject};
use log::{debug, error, info, warn};


type UID = String;

#[derive(Debug, Clone)]
pub struct DicomObject {
    pub dcm: FileDicomObject<InMemDicomObject>,
}

impl DicomObject {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let f = std::io::Cursor::new(bytes);
        let dcm = from_reader(f)?;
        Ok(Self { dcm })
    }
}

#[derive(Debug, Default, Clone)]
pub struct ImageSeries {
    instance_uid: UID,
    number: String,
}

#[derive(Debug, Default, Clone)]
pub struct StudySet {
    instance_uid: String,
    id: String,
}

#[derive(Debug, Default, Clone)]
pub struct Patient {
    name: String,
    id: String,
    // study_sets: HashMap<UID, StudySet>,
}

#[derive(Debug, Default)]
pub struct DCMStructure {
    patients: HashMap<UID, Patient>,
    study_sets: HashMap<UID, StudySet>,
    image_series: HashMap<UID, ImageSeries>,
    patient_to_study_sets: HashMap<UID, Vec<UID>>, // Patient ID to StudySet Instance UID
    study_sets_to_image_series: HashMap<UID, Vec<UID>>, // StudySet Instance UID to ImageSeries Instance UID
}


pub fn read_dicom() -> Result<()> {
    warn!("reading dicom file");
    let bytes = include_bytes!("C:\\share\\imrt\\CT.RT001921_1.dcm");
    let f = std::io::Cursor::new(bytes);
    let dcm = from_reader(f)?;
    let patient_name = dcm.element_by_name("PatientName")?.to_str()?;
    let modality = dcm.element_by_name("Modality")?.to_str()?;
    let loc = dcm.element_by_name("SliceLocation")?.to_str()?;
    let pixel_data_bytes = dcm.element(Tag(0x7FE0, 0x0010))?.to_bytes()?;
    let pixels: &[i16] = cast_slice(&pixel_data_bytes);
    warn!("{:?}", patient_name);
    warn!("{:?}", modality);
    warn!("slice location: {}", loc);
    // warn!("{:?}", pixels);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dicom() -> Result<()> {
        let obj = open_file("C:\\share\\imrt\\CT.RT001921_1.dcm")?;
        let patient_name = obj.element_by_name("PatientName")?.to_str()?;
        let modality = obj.element_by_name("Modality")?.to_str()?;
        let pixel_data_bytes = obj.element(Tag(0x7FE0, 0x0010))?.to_bytes()?;
        let pixels: &[i16] = cast_slice(&pixel_data_bytes);
        println!("{:?}", patient_name);
        println!("{:?}", modality);
        println!("num of pxiels: {:?}", pixels.len());
        Ok(())
    }

    #[test]
    fn test_dicom_reader() -> Result<()> {
        let bytes = include_bytes!("C:\\share\\imrt\\CT.RT001921_1.dcm");
        let mut f = std::io::Cursor::new(bytes);
        let obj = from_reader(f)?;
        let patient_name = obj.element_by_name("PatientName")?.to_str()?;
        let modality = obj.element_by_name("Modality")?.to_str()?;
        let pixel_data_bytes = obj.element(Tag(0x7FE0, 0x0010))?.to_bytes()?;
        let pixels: &[i16] = cast_slice(&pixel_data_bytes);
        println!("{:?}", patient_name);
        println!("{:?}", modality);
        println!("num of pxiels: {:?}", pixels.len());
        Ok(())
    }
}
