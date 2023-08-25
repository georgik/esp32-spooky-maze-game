use spooky_core::movement_controller::MovementController;
use crate::keyboard_movement_controller::KeyboardMovementController;
use spooky_core::demo_movement_controller::DemoMovementController;
use embedded_graphics_simulator::sdl2::Keycode;

pub struct DesktopMovementControllerBuilder {
    pub demo_movement_controller: DemoMovementController,
    pub keyboard_movement_controller: KeyboardMovementController,
    pub(crate) active_index: usize,
}

impl DesktopMovementControllerBuilder {
    pub fn handle_key(&mut self, keycode: Keycode) {
        self.keyboard_movement_controller.handle_key(keycode);
    }

    pub fn stop_movement(&mut self) {
        self.keyboard_movement_controller.stop_movement();
    }
}

impl MovementController for DesktopMovementControllerBuilder {

    fn set_active(&mut self, index:usize) {
        self.active_index = index;
    }

    fn tick(&mut self) {
        match self.active_index {
            0 => self.demo_movement_controller.tick(),
            1 => self.keyboard_movement_controller.tick(),
            _ => {}
        }
    }

    fn get_movement(&self) -> spooky_core::engine::Action {

        match self.active_index {
            0 => self.demo_movement_controller.get_movement(),
            1 => self.keyboard_movement_controller.get_movement(),
            _ => spooky_core::engine::Action::None,
        }
    }
}
