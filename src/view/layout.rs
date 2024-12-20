use super::{Renderable, View};

pub struct Layout {
    dim: (u32, u32),
    pub(crate) views: Vec<Box<dyn View>>, // A collection of views
}

impl Layout {
    pub fn new(dim: (u32, u32)) -> Self {
        Self {
            dim,
            views: Vec::new(),
        }
    }

    pub fn add_view(&mut self, mut view: Box<dyn View>) {
        if self.views.len() < 4 {
            let idx = self.views.len() as u32;
            let d = self.dim.0 / 2;
            let pos_x: i32 = (idx % 2 * d) as i32;
            let pos_y: i32 = if idx < 2 {0} else {d as i32};
            view.move_to((pos_x, pos_y));
            view.resize((d, d));
            self.views.push(view);
        } else {
            unreachable!()
        }
    }
}

impl Renderable for Layout {
    fn update(&mut self, queue: &wgpu::Queue) {
        for v in &mut self.views {
            v.update(queue);
        }
    }

    fn render(&mut self, render_pass: &mut wgpu::RenderPass) -> Result<(), wgpu::SurfaceError> {
        for v in &mut self.views {
            v.render(render_pass)?;
        }
        Ok(())
    }
}