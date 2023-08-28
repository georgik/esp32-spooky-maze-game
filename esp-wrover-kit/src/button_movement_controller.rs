use spooky_core::engine::Action;
use spooky_core::movement_controller::MovementController;

use crate::button_keyboard::ButtonEvent;

pub struct ButtonMovementController {
    last_action: Action,
}

impl ButtonMovementController {
    pub fn new() -> Self {
        Self {
            last_action: Action::None,
        }
    }

    pub fn react_to_event(&mut self, event: ButtonEvent) {
        match event {
            ButtonEvent::UpPressed => self.last_action = Action::Up,
            ButtonEvent::DownPressed => self.last_action = Action::Down,
            ButtonEvent::LeftPressed => self.last_action = Action::Left,
            ButtonEvent::RightPressed => self.last_action = Action::Right,
            ButtonEvent::DynamitePressed => self.last_action = Action::PlaceDynamite,
            ButtonEvent::TeleportPressed => self.last_action = Action::Teleport,
            ButtonEvent::NoEvent => self.last_action = Action::None,
        }
    }
}

impl MovementController for ButtonMovementController {
    fn set_active(&mut self, _index: usize) {
    }

    fn get_movement(&self) -> Action {
        self.last_action
    }

    fn tick(&mut self) {
        // The logic for setting `last_action` is now moved to `react_to_event`
    }
}
