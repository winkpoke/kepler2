use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};
use tokio::task;
use std::sync::Arc;

pub async fn read_and_process_files<F, E>(file_paths: Vec<PathBuf>, process: Arc<F>) -> io::Result<()>
where
    F: Fn(Vec<u8>) -> Result<(), E> + Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut tasks = Vec::new();

    for file_path in file_paths {
        let process = Arc::clone(&process);
        
        let task = task::spawn(async move {
            let mut file = File::open(&file_path).await?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).await?;
            
            // Call the process function and handle any error it returns
            process(contents).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            Ok::<(), io::Error>(())
        });

        tasks.push(task);
    }

    // Await all tasks and propagate errors if any occur
    for task in tasks {
        task.await??;
    }

    Ok(())
}

use std::io::Cursor;
use std::borrow::Cow;
// use bytemuck::cast_slice;
use bytemuck::checked::try_cast_slice;  
use dicom_core::Tag;
use dicom_object::{from_reader, InMemDicomObject};

/// Retrieve a DICOM element as a string.
fn get_element_as_str<'a>(dcm: &'a InMemDicomObject, name: &str) -> Result<Cow<'a, str>, &'static str> {
    match dcm.element_by_name(name) {
        Ok(ele) => ele.to_str().map(Cow::from).map_err(|_| "Error converting element to string"),
        Err(_) => Err("Error accessing element by name"),
    }
}

/// Retrieve pixel data as a slice of i16, if available.
fn get_pixel_data<'a>(dcm: &'a InMemDicomObject) -> Result<Cow<'a, [i16]>, &'static str> {
    match dcm.element(Tag(0x7FE0, 0x0010)) {
        Ok(ele) => match ele.to_bytes() {
            Ok(pixel_data_bytes) => {
                // Safely cast the byte slice to a slice of i16s
                match try_cast_slice::<_, i16>(pixel_data_bytes.as_ref()) {
                    Ok(pixels) => Ok(Cow::Owned(pixels.to_vec())),  // Owned Vec<i16>
                    Err(_) => Err("Pixel data length is not aligned to i16 size"),
                }
            }
            Err(_) => Err("Error converting element to bytes"),
        },
        Err(_) => Err("Pixel data element not found"),
    }
}

/// Process a DICOM file represented as raw bytes.
pub fn process(data: Vec<u8>) -> io::Result<()> {
    println!("Processing data of size: {}", data.len());

    // Wrap the data in a Cursor to simulate reading from a file
    let cursor = Cursor::new(data);

    // Attempt to open the DICOM object from the in-memory data
    let dcm = match from_reader(cursor) {
        Ok(dcm) => dcm,
        Err(e) => {
            println!("Failed to parse DICOM object: {:?}", e);
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Failed to parse DICOM object"));
        }
    };

    // Retrieve various DICOM elements by name using the helper function
    match get_element_as_str(&dcm, "PatientName") {
        Ok(patient_name) => println!("Patient Name: {:?}", patient_name),
        Err(e) => println!("Error retrieving Patient Name: {}", e),
    }

    match get_element_as_str(&dcm, "Modality") {
        Ok(modality) => println!("Modality: {:?}", modality),
        Err(e) => println!("Error retrieving Modality: {}", e),
    }

    match get_element_as_str(&dcm, "SliceLocation") {
        Ok(loc) => println!("Slice Location: {}", loc),
        Err(e) => println!("Error retrieving Slice Location: {}", e),
    }

    // Retrieve and handle pixel data
    match get_pixel_data(&dcm) {
        Ok(pixels) if !pixels.is_empty() => {
            println!("Pixel data (sample): {:?}", &pixels[0..10]);
        }
        Ok(_) => println!("No pixel data found."),
        Err(e) => println!("Error retrieving pixel data: {}", e),
    }

    Ok(())
}