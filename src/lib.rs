#![feature(duration_millis_float)]

use log::{debug, error, info, warn};
use wgpu::Label;
use std::{iter, sync::Arc};
// use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

use view::Renderable;
// use view::TransverseView;
use view::TransverseView;

// mod texture;
pub mod coordinates;
pub mod ct_volume;
pub mod dicom;
mod texture_3d;
mod view;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ct_volume::*;
use dicom::*;

use std::time::Instant;

fn list_files_in_directory(dir: &str) -> io::Result<Vec<PathBuf>> {
    let mut file_paths = Vec::new();

    // Open the directory and iterate over its contents
    for entry in fs::read_dir(dir)? {
        let entry = entry?; // unwrap the result of read_dir
        let path = entry.path();

        // Check if the entry is a file (not a directory)
        if path.is_file() {
            file_paths.push(path); // Add the full path to the list
        }
    }

    Ok(file_paths)
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

struct SagittalView {
    view: view::View,
}

impl view::Renderable for SagittalView {
    fn update(&mut self, queue: &wgpu::Queue) {
        // Update the rotation angle, e.g., incrementing it over time
        self.view.uniforms.vert.rotation_angle_y -= 0.01; // Update rotation angle
        self.view.uniforms.vert.rotation_angle_z -= 0.01; // Update rotation angle
        if self.view.uniforms.frag.slice <= 0.0 {
            self.view.uniforms.frag.slice = 1.0;
        } else {
            self.view.uniforms.frag.slice -= 0.01;
        }

        queue.write_buffer(
            &self.view.uniform_vert_buffer,
            0,
            bytemuck::cast_slice(&[self.view.uniforms.vert]),
        );
        queue.write_buffer(
            &self.view.uniform_frag_buffer,
            0,
            bytemuck::cast_slice(&[self.view.uniforms.frag]),
        );
    }

    fn render(&mut self, render_pass: &mut wgpu::RenderPass) -> Result<(), wgpu::SurfaceError> {
        render_pass.set_pipeline(&self.view.render_pipeline); // 2.
        render_pass.set_viewport(800.0, 0.0, 800.0, 800.0, 0.0, 1.0);
        render_pass.set_bind_group(0, &self.view.texture_bind_group, &[]);
        render_pass.set_bind_group(1, &self.view.uniform_vert_bind_group, &[]);
        render_pass.set_bind_group(2, &self.view.uniform_frag_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.view.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.view.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.view.num_indices, 0, 0..1);
        Ok(())
    }
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: &'a Window,
    texture: texture_3d::Texture,
    transverse_view: Vec<TransverseView>,
    // sagittal_view: SagittalView,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            // backends: wgpu::Backends::PRIMARY,
            backends: wgpu::Backends::DX12,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            // backends: wgpu::BROWSER_WEBGPU,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits {
                            max_texture_dimension_3d: 1024,
                            ..wgpu::Limits::downlevel_webgl2_defaults()
                        }
                    } else {
                        wgpu::Limits::default()
                    },
                    memory_hints: Default::default(),
                },
                // Some(&std::path::Path::new("trace")), // Trace path
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            // format: surface_format,
            format: wgpu::TextureFormat::Rgba8Unorm,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };
        println!("format: {:?}", surface_format);

        error!("size: {}, {}", size.width, size.height);
        println!("print size: {}, {}", size.width, size.height);
        if size.width > 0 && size.height > 0 {
            surface.configure(&device, &config);
        }

        // let diffuse_bytes =
        //     include_bytes!("../image/Free-Crochet-Baby-Tiger-Amigurumi-Pattern.png");
        // include_bytes!("../image/CT.png");
        // println!("len = {}", diffuse_bytes.len());
        #[cfg(target_arch = "wasm32")]
        let texture =
            // texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "Baby Tiger").unwrap();
            texture_3d::Texture::from_file_at_compile_time(&device, &queue, "CT", 512, 512, 10).unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        let texture = {
            // Start the timer
            let start_time = Instant::now();

            let file_names = list_files_in_directory("C:\\share\\imrt").unwrap();
            let repo = dicom::fileio::parse_dcm_directories(vec![
                "C:\\share\\imrt",
                "C:\\share\\head_mold",
            ])
            .await
            .unwrap();
            println!("DicomRepo:\n{}", repo.to_string());
            println!("Patients:\n{:?}", repo.get_all_patients());
            // Stop the timer
            let elapsed_time = start_time.elapsed();

            // Print the repository and performance details
            // println!("Parsed repository: {:?}", repo);
            println!(
                "Parsing completed in {:.1} ms.",
                elapsed_time.as_millis_f32()
            );

            let start_time = Instant::now();
            let vol = repo
                .generate_ct_volume("1.2.392.200036.9116.2.5.1.144.3437232930.1426478676.964561")
                .unwrap();
            let elapsed_time = start_time.elapsed();
            println!(
                "CTVolume being generated in {:.1} ms.",
                elapsed_time.as_millis_f32()
            );
            let mut ct_images = repo.get_images_by_series("1.2.392.200036.9116.2.5.1.144.3437232930.1426478676.964561");
            ct_images.sort_by(|a, b| {
                let z_a = a.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
                let z_b = b.image_position_patient.map(|pos| pos.2).unwrap_or(0.0);
                z_a.partial_cmp(&z_b).unwrap_or(std::cmp::Ordering::Equal)
            });
            let ct_image = ct_images[0];
            let (ox, oy, oz0) = ct_images[0].image_position_patient.unwrap();
            let (_, _, oz1) = ct_images.last().unwrap().image_position_patient.unwrap();
            let dz = oz1 - oz0;
            let nx = ct_image.rows as f32;
            let ny = ct_image.columns as f32;
            let space = ct_image.pixel_spacing.unwrap();

            use crate::coordinates::*;
            let m_uv = [nx*space.0, 0.0,        0.0, ox,
                        0.0,        ny*space.1, 0.0, oy,
                        0.0,        0.0,        dz,  oz0,
                        0.0,        0.0,        0.0, 1.0];
            let matrix_uv = Matrix4x4::<f32>::from_array(m_uv);
            println!("{:?}", matrix_uv);
            let d = f32::max(nx * space.0, ny * space.1);
            let m_screen = [d,    0.0,  0.0, ox,
                            0.0,  0.0,  d,   oy,
                            0.0,  -d,   0.0, (oz0+oz1)/2.0+d/2.0,
                            0.0,  0.0,  0.0, 1.0];
            let matrix_screen = Matrix4x4::<f32>::from_array(m_screen);
            println!("{:?}", matrix_screen);
            
            
            let base_uv = Base::<f32> {
                label: "CT Volume: UV".to_string(),
                matrix: matrix_uv,
            };
            println!("{:?}", matrix_uv.inv());

            let base_screen = Base::<f32> {
                label: "CT Volume: screen".to_string(),
                matrix: matrix_screen,
            };
            let base = base_screen.to_base(&base_uv);
            println!("{:?}", base);

            println!("CT Volume:\n{:#?}", vol);
            let voxel_data: Vec<u16> = vol.voxel_data.iter().map(|x| (*x + 1000) as u16).collect();
            let voxel_data: &[u8] = bytemuck::cast_slice(&voxel_data);
            texture_3d::Texture::from_bytes(
                &device,
                &queue,
                voxel_data,
                "CT Volume",
                vol.dimensions.0 as u32,
                vol.dimensions.1 as u32,
                vol.dimensions.2 as u32,
            ).unwrap()
        };


        println!("supported texture formats: {:?}", surface_caps.formats);
        println!("format: {:?}", config.format);
        let mut transverse_view = Vec::<TransverseView>::new();
        for i in 0..4 {
            let view = TransverseView::new(&device, &texture, i, 0.00, 0.005 * (i as f32) / 2.0);
            transverse_view.push(view);
        }
        // let sagittal_view = SagittalView {
        //     view: view::View::new(&device, &texture, wgsl_path),
        // };

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            texture,
            transverse_view,
            // sagittal_view,
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        for i in 0..self.transverse_view.len() {
            self.transverse_view[i].update(&self.queue);
        }
        // self.sagittal_view.update(&self.queue);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.5,
                            b: 0.5,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            for i in 0..self.transverse_view.len() {
                self.transverse_view[i].render(&mut render_pass)?;
            }
            // self.sagittal_view.render(&mut render_pass)?;
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
#[cfg(target_arch = "wasm32")]
pub async fn init() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run() {
    // cfg_if::cfg_if! {
    //     if #[cfg(target_arch = "wasm32")] {
    //         std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    //         console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
    //     } else {
    //         env_logger::init();
    //     }
    // }
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();

    warn!("Start the program ...");

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");

        let _ = window.request_inner_size(PhysicalSize::new(800, 800));
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = window.request_inner_size(PhysicalSize::new(800, 800));

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(&window).await;
    let mut surface_configured = false;

    log::info!("Starting the event loop ...");
    event_loop
        .run(move |event, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == state.window().id() => {
                    if !state.input(event) {
                        // UPDATED!
                        match event {
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        state: ElementState::Pressed,
                                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                                        ..
                                    },
                                ..
                            } => control_flow.exit(),
                            WindowEvent::Resized(physical_size) => {
                                log::info!("physical_size: {physical_size:?}");
                                surface_configured = true;
                                state.resize(*physical_size);
                            }
                            WindowEvent::RedrawRequested => {
                                // This tells winit that we want another frame after this one
                                state.window().request_redraw();

                                if !surface_configured {
                                    return;
                                }

                                state.update();
                                match state.render() {
                                    Ok(_) => {}
                                    // Reconfigure the surface if it's lost or outdated
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => state.resize(state.size),
                                    // The system is out of memory, we should probably quit
                                    Err(wgpu::SurfaceError::OutOfMemory) => {
                                        log::error!("OutOfMemory");
                                        control_flow.exit();
                                    }

                                    // This happens when the a frame takes too long to present
                                    Err(wgpu::SurfaceError::Timeout) => {
                                        log::warn!("Surface timeout")
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}
