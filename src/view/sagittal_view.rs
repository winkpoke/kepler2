use crate::geometry::GeometryBuilder;
use crate::{view, DicomRepo};
use crate::texture_3d::Texture;

pub struct SagittalView {
    view: view::View,
    r_speed: f32,
    s_speed: f32,
    idx: i32,
}

impl SagittalView {
    pub fn new(device: &wgpu::Device, texture: &Texture, idx: i32, r_speed: f32, s_speed: f32, repo: &DicomRepo) -> Self {
        let base_screen = GeometryBuilder::build_sagittal_base(&repo);
        let base_uv = GeometryBuilder::build_uv_base(&repo);

        let transform_matrix = base_screen.to_base(&base_uv);
        println!("row major: {:?}", transform_matrix);

        let transform_matrix = transform_matrix.transpose(); // row major to column major
        println!("column major: {:?}", transform_matrix);

        let wgsl_path: &str = include_str!("../shader/shader_tex.wgsl");
        let view = view::View::new(&device, &texture, wgsl_path, transform_matrix);

        Self {
            view,
            r_speed,
            s_speed,
            idx,
        }
    }
}

impl view::Renderable for SagittalView {
    fn update(&mut self, queue: &wgpu::Queue) {
        // Update the rotation angle, e.g., incrementing it over time
        self.view.uniforms.vert.rotation_angle_y += self.r_speed; //0.05; // Update rotation angle
        // self.view.uniforms.vert.rotation_angle_z += self.r_speed; //0.05; // Update rotation angle
        if self.view.uniforms.frag.slice >= 1.0 {
            self.view.uniforms.frag.slice = 0.0;
        } else {
            self.view.uniforms.frag.slice += self.s_speed; //0.005;
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
        let width = 400;
        let x: f32 = (self.idx % (800 / width))  as f32 * width as f32;
        let y: f32 = (self.idx / (800 / width)) as f32 * width as f32;
        render_pass.set_viewport(x, y, width as f32, width as f32, 0.0, 1.0);
        render_pass.set_bind_group(0, &self.view.texture_bind_group, &[]);
        render_pass.set_bind_group(1, &self.view.uniform_vert_bind_group, &[]);
        render_pass.set_bind_group(2, &self.view.uniform_frag_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.view.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.view.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.view.num_indices, 0, 0..1);
        Ok(())
    }
}