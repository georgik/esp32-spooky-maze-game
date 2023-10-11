use spooky_core::engine::Action;
use spooky_core::movement_controller::MovementController;

pub struct RotaryMovementController
{
    last_action: Action,
}

impl RotaryMovementController
{
    pub fn new() -> Self {
        Self {
            last_action: Action::None,
        }
    }

    pub fn set_movement(&mut self, _action: Action) {
        self.last_action = _action;
    }
}

impl MovementController for RotaryMovementController
{
    fn set_active(&mut self, _index:usize) {
    }


    fn get_movement(&self) -> Action {
        self.last_action
    }

    fn tick(&mut self) {
        //self.last_action = Action::None;
    }
}
