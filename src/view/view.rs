use super::Renderable;

pub trait View: Renderable + Send {
    fn position(&self) -> (i32, i32);
    fn dimensions(&self) -> (u32, u32);
    fn move_to(&mut self, pos: (i32, i32));
    fn resize(&mut self, dim: (u32, u32));
}