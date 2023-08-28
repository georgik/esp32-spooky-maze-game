use spooky_core::movement_controller::MovementController;
use crate::{button_movement_controller::ButtonMovementController, button_keyboard::ButtonEvent};
use spooky_core::engine::Action;

pub struct EmbeddedMovementController {
    pub demo_movement_controller: spooky_core::demo_movement_controller::DemoMovementController,
    pub button_movement_controller: ButtonMovementController,
    pub(crate) active_index: usize,
}

impl EmbeddedMovementController {
    pub fn new(
        demo_movement_controller: spooky_core::demo_movement_controller::DemoMovementController,
        button_movement_controller: ButtonMovementController,
    ) -> Self {
        Self {
            demo_movement_controller,
            button_movement_controller,
            active_index: 0, // Initially, demo is active
        }
    }

    pub fn react_to_event(&mut self, event: ButtonEvent) {
        match event {
            ButtonEvent::TeleportPressed => self.active_index = 0,
            _ => {
                if self.active_index == 1 {
                    self.button_movement_controller.react_to_event(event);
                }
            }
        }
    }
}

impl MovementController for EmbeddedMovementController {
    fn set_active(&mut self, index: usize) {
        self.active_index = index;
    }

    fn tick(&mut self) {
        match self.active_index {
            0 => self.demo_movement_controller.tick(),
            1 => {}, // No action required, events are processed in react_to_event
            _ => {}
        }
    }

    fn get_movement(&self) -> Action {
        match self.active_index {
            0 => self.demo_movement_controller.get_movement(),
            1 => self.button_movement_controller.get_movement(),
            _ => Action::None,
        }
    }
}
