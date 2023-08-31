use crate::engine::Action;
use crate::movement_controller::MovementController;

pub struct NoMovementController;

impl NoMovementController {
    pub fn new() -> Self {
        NoMovementController
    }
}

impl MovementController for NoMovementController {
    fn tick(&mut self) {}

    fn set_active(&mut self, _index:usize) {
    }

    fn get_movement(&self) -> Action {
        Action::None
    }
}