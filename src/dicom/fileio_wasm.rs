use js_sys::{
    Array,
    Function,
    Object,
    // ArrayBuffer,
    Uint8Array,
    // Uint8ClampedArray,
    // Uint16Array,
    // Float32Array
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;
use web_sys::{
    File,
    FileReader,
    // Performance,
    ProgressEvent,
};

use crate::dicom::*;

#[wasm_bindgen]
pub fn load_file(file: &File, f: Function) -> Result<JsValue, JsValue> {
    let file_reader = FileReader::new().map_err(|_| JsValue::from_str("Failed to create FileReader"))?;

    // Create the closure for handling the file load
    let cb = Closure::once_into_js(move |e: ProgressEvent| -> Result<JsValue, JsValue> {
        console::log_1(&JsValue::from_str("Reading file..."));

        // Extract the FileReader from the event (using `ok_or` to handle Option to Result)
        let file_reader = e.target()
            .and_then(|t| t.dyn_into::<FileReader>().ok())
            .ok_or_else(|| JsValue::from_str("Failed to cast target to FileReader"))?;

        // Extract the result (file content) from the FileReader
        let buffer = file_reader.result()
            .map_err(|_| JsValue::from_str("Failed to read file content"))?;

        console::log_1(&JsValue::from_str(&format!("Buffer: {:?}", buffer)));

        let u8arr: Uint8Array = Uint8Array::new(&buffer);

        // Measure the time taken to load the buffer into WebAssembly
        let p = web_sys::window()
            .and_then(|w| w.performance())
            .expect("Cannot get performance");
        let t0 = p.now();

        // Process the DICOM object from the buffer
        let dcm = DicomObject::from_array_buffer(&buffer).map_err(|err| {
            JsValue::from_str(&format!("Failed to parse DICOM object: {:?}", err))
        })?;

        let pixels = u8arr.to_vec();
        let pixels_u8arr = unsafe { Uint8Array::view(&pixels) };

        // Measure the time taken to load the buffer into WebAssembly
        let t1 = p.now();
        console::log_1(&JsValue::from_str(
            format!("Loading buffer into wasm: {} msecs", t1 - t0).as_ref(),
        ));

        // Parse the DICOM elements and handle any errors
        let ds = parseDicom(pixels_u8arr).map_err(|err| {
            JsValue::from_str(&format!("Failed to parse DICOM data: {:?}", err))
        })?;

        console::log_1(&ds);
        let elem = ds.elements();
        console::log_1(&elem);

        let obj = Object::try_from(&elem).ok_or_else(|| JsValue::from_str("Failed to create Object from elements"))?;
        let elem_arr = Object::entries(&obj);
        console::log_1(&elem_arr);

        // Access an element in the array
        let value_at_index_0 = elem_arr.get(0);
        let value = Array::from(&value_at_index_0).get(1);
        console::log_1(&value);

        // Call the provided function with the DICOM object
        f.call1(&JsValue::NULL, &JsValue::from(dcm)).map_err(|err| {
            JsValue::from_str(&format!("Function callback error: {:?}", err))
        })?;

        // Return success after processing
        Ok(JsValue::from(true))
    });

    // Set up the file reader to use the closure
    file_reader.set_onload(Some(cb.as_ref().unchecked_ref()));

    // Start reading the file as an array buffer
    file_reader.read_as_array_buffer(file).map_err(|e| JsValue::from_str(&format!("Error reading file: {:?}", e)))?;

    // Return a placeholder value since the actual result will be processed asynchronously
    Ok(JsValue::from(true))
}

#[wasm_bindgen]
pub fn parse_dcm_structure(files: &Array, f: Function) -> Result<JsValue, JsValue> {
    let count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let accumulator = Arc::new(Mutex::new(DCMStructure::new()));
    let len = files.length();
    for idx in 0..len {
        let file: File = files.get(idx).dyn_into()?;
        let file_reader = FileReader::new()?;
        let count_clone = count.clone();
        let acc_clone = accumulator.clone();
        let f_clone = f.clone();
        let cb = Closure::once_into_js(move |e: ProgressEvent| -> Result<JsValue, JsValue> {
            // console::log_1(&JsValue::from(count_clone.load(Ordering::SeqCst)));
            let buffer: JsValue = e
                .target()
                .ok_or(JsValue::from(
                    "Error when retreiving e.target in function parse_dcm_structure.",
                ))?
                .dyn_into::<FileReader>()?
                .result()?;
            let dcm = DicomObject::from_array_buffer(&buffer)?;
            if dcm.sop_class_uid()?.contains("1.2.840.10008.5.1.4.1.1.2") {
                acc_clone
                    .lock()
                    .expect("acquire mutex error in parse_dcm_structure")
                    .add_slice(&dcm)?;
            }
            if count_clone.fetch_add(1, Ordering::SeqCst) == len as usize - 1 {
                let _v =
                    f_clone.call1(&JsValue::null(), &acc_clone.lock().unwrap().to_js_obj()?)?;
            }
            Ok(JsValue::undefined())
        });

        file_reader.set_onload(Some(cb.as_ref().unchecked_ref()));
        file_reader.read_as_array_buffer(&file).expect("read error");
    }
    Ok(JsValue::undefined())
}

#[wasm_bindgen]
pub fn load_image_series_by_uid(
    files: &Array,
    uid: String,
    f: Function,
) -> Result<JsValue, JsValue> {
    let count: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let accumulator = Arc::new(Mutex::new(CTVolume::new()));
    let len = files.length();
    for idx in 0..len {
        let file: File = files.get(idx).dyn_into()?;
        let file_reader = FileReader::new()?;
        let count_clone = count.clone();
        let acc_clone = accumulator.clone();
        let f_clone = f.clone();
        let uid_clone = uid.clone();
        let cb = Closure::once_into_js(move |e: ProgressEvent| -> Result<JsValue, JsValue> {
            console::log_1(&JsValue::from(count_clone.load(Ordering::SeqCst)));
            let buffer: JsValue = e
                .target()
                .ok_or(JsValue::from(
                    "Error when retreiving e.target in function parse_dcm_structure.",
                ))?
                .dyn_into::<FileReader>()?
                .result()?;
            let dcm = DicomObject::from_array_buffer(&buffer)?;
            let s = dcm.series_instance_uid()?;
            // if !(s == uid_clone) {
            //     console::log_1(&JsValue::from(format!("skip {}", count_clone.load(Ordering::SeqCst))));
            //     return Ok(JsValue::undefined());
            // }
            if s == uid_clone {
                console::log_1(&JsValue::from(format!(
                    "add slice: {}",
                    count_clone.load(Ordering::SeqCst)
                )));
                if let Ok(ct) = CTImage::from_dicom_object(&dcm) {
                    acc_clone
                        .lock()
                        .expect("acquire mutex error in parse_dcm_structure")
                        .add_slice(ct);
                }
            }
            if count_clone.fetch_add(1, Ordering::SeqCst) == len as usize - 1 {
                let mutex = Arc::try_unwrap(acc_clone).unwrap();
                let volume = mutex.into_inner().unwrap();
                console::log_1(&JsValue::from(format!("------------------")));
                let _v = f_clone.call1(&JsValue::null(), &JsValue::from(volume))?;
            }
            Ok(JsValue::undefined())
        });

        file_reader.set_onload(Some(cb.as_ref().unchecked_ref()));
        file_reader.read_as_array_buffer(&file).expect("read error");
    }
    Ok(JsValue::undefined())
}
