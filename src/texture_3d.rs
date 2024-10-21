use anyhow::*;
use log::{debug, error, info};
use wgpu::util::DeviceExt;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    // Read a 3D texture from bytes
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
        width: u32,
        height: u32,
        depth: u32,
    ) -> Result<Self> {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: depth,
        };
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D3,
                format: wgpu::TextureFormat::Rg8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(2 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );
        Ok(Self { texture, view, sampler })
    }

    // Function to read a 3D texture from a file at compile time
    pub fn from_file_at_compile_time(device: &wgpu::Device, queue: &wgpu::Queue, label: &str, width: u32, height: u32, depth: u32) -> Result<Self> {
        // Load the binary texture file
        let bytes = include_bytes!("../image/combined_pixel_array3.bin");
        // let bytes_u16: &[u16] = bytemuck::cast_slice(bytes);
        // let bytes_u32: Vec<u32> = bytes_u16.iter().map(|&v| v as u32).collect();
        // let bytes_u8: &[u8] = bytemuck::cast_slice(&bytes_u32);
        // println!("len: {}", bytes_u8.len());
        Self::from_bytes(device, queue, bytes, label, width, height, depth)
    }
}
