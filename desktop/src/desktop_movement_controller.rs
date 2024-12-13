use crate::keyboard_movement_controller::KeyboardMovementController;
use embedded_graphics_simulator::sdl2::Keycode;
use spooky_core::demo_movement_controller::DemoMovementController;
use spooky_core::engine::Action;
use spooky_core::movement_controller::MovementController;

pub struct DesktopMovementControllerBuilder {
    pub demo_movement_controller: DemoMovementController,
    pub keyboard_movement_controller: KeyboardMovementController,
    pub(crate) active_index: usize,
    last_action: Action,
}

impl DesktopMovementControllerBuilder {
    pub fn new(
        demo_movement_controller: DemoMovementController,
        keyboard_movement_controller: KeyboardMovementController,
    ) -> Self {
        Self {
            demo_movement_controller,
            keyboard_movement_controller,
            active_index: 0,
            last_action: Action::None,
        }
    }

    pub fn handle_key(&mut self, keycode: Keycode) {
        self.keyboard_movement_controller.handle_key(keycode);
    }

    pub fn stop_movement(&mut self) {
        self.keyboard_movement_controller.stop_movement();
    }
}

impl MovementController for DesktopMovementControllerBuilder {
    fn set_active(&mut self, index: usize) {
        self.active_index = index;

        // match self.active_index {
        //     0 => self.last_action = Action::Stop,
        //     1 => self.last_action = Action::Start,
        //     _ => {}
        // };
    }

    fn tick(&mut self) {
        self.last_action = Action::None;
        match self.active_index {
            0 => self.demo_movement_controller.tick(),
            1 => self.keyboard_movement_controller.tick(),
            _ => {}
        }
    }

    fn get_movement(&self) -> spooky_core::engine::Action {
        if self.last_action != Action::None {
            return self.last_action;
        }

        match self.active_index {
            0 => self.demo_movement_controller.get_movement(),
            1 => self.keyboard_movement_controller.get_movement(),
            _ => spooky_core::engine::Action::None,
        }
    }
}
