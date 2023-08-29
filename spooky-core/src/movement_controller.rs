use crate::engine::Action;

pub trait MovementController {
    fn set_active(&mut self, index:usize);
    fn tick(&mut self);
    fn get_movement(&self) -> Action;
}
