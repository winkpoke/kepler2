#[cfg_attr(not(target_arch="wasm32"), path="dicom.rs")]
#[cfg_attr(target_arch="wasm32", path="dicom_wasm.rs")]
mod dicom;

pub use dicom::*;

#[cfg_attr(not(target_arch="wasm32"), path="fileio.rs")]
#[cfg_attr(target_arch="wasm32", path="fileio_wasm.rs")]
pub mod fileio;

mod dicom_ai;