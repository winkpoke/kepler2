use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use anyhow::Result;

use super::*;

/// Parses a list of DICOM files asynchronously and constructs a `DicomRepo`.
/// 
/// # Arguments
/// - `file_paths`: A vector of file paths to DICOM files.
/// - `callback`: A function to call with the constructed `DicomRepo`.
pub async fn parse_dcm_directories(
    directories: Vec<&str>,
) -> Result<DicomRepo> {
    // Shared repository and counter tracker
    let repo = Arc::new(Mutex::new(DicomRepo::new()));
    let count = Arc::new(AtomicUsize::new(0));

    let mut file_paths = Vec::new();

    // Collect all files from the provided directories
    for dir_path in directories {
        let mut entries = fs::read_dir(dir_path).await.map_err(|err| {
            eprintln!("Error reading directory {}: {}", dir_path, err);
            anyhow::Error::new(err)
        })?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                file_paths.push(path);
            }
        }
    }

    // Process files concurrently
    let mut tasks = vec![];
    for file_path in file_paths {
        let repo_clone = Arc::clone(&repo);
        let count_clone = Arc::clone(&count);

        let task: tokio::task::JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async move {
            // Open the file asynchronously
            let mut file = File::open(&file_path).await.map_err(|err| {
                eprintln!("Error opening file {}: {}", file_path.display(), err);
                anyhow::Error::new(err)
            })?;

            // Read the file contents into a buffer
            let mut buffer = vec![];
            file.read_to_end(&mut buffer).await.map_err(|err| {
                eprintln!("Error reading file {}: {}", file_path.display(), err);
                anyhow::Error::new(err)
            })?;

            // Parse the DICOM data and update the repository
            {
                let mut repo = repo_clone.lock().await;
                if let Ok(patient) = Patient::from_file(&buffer) {
                    repo.add_patient(patient);
                }
                if let Ok(study) = StudySet::from_file(&buffer) {
                    repo.add_study(study);
                }
                if let Ok(series) = ImageSeries::from_file(&buffer) {
                    repo.add_image_series(series);
                }
                if let Ok(ct_image) = CTImage::from_file(&buffer) {
                    repo.add_ct_image(ct_image);
                }
            }

            count_clone.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    for task in tasks {
        match task.await {
            Ok(Ok(())) => { /* Task succeeded */ }
            Ok(Err(err)) => eprintln!("Task error: {}", err),
            Err(join_err) => eprintln!("Task panicked or was cancelled: {:?}", join_err),
        }
    }

    // Extract the final repository and return it
    let repo = repo.lock().await;
    Ok((*repo).clone())
}