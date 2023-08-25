use crate::engine::Action;

pub trait MovementController {
    fn set_active(&mut self, index:usize);
    fn tick(&mut self);
    fn get_movement(&self) -> Action;
}

pub struct NoMovementController;

impl MovementController for NoMovementController {
    fn tick(&mut self) {}

    fn set_active(&mut self, _index:usize) {
    }

    fn get_movement(&self) -> Action {
        Action::None
    }
}
