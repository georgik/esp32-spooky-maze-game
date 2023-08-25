use spooky_core::engine::Action;
use spooky_core::universe::MovementController;
use embedded_graphics_simulator::sdl2::Keycode;

pub struct KeyboardMovementController {
    last_action: Action,
}

impl KeyboardMovementController {
    pub fn new() -> Self {
        KeyboardMovementController {
            last_action: Action::None,
        }
    }

    pub fn handle_key(&mut self, keycode: Keycode) {
        self.last_action = match keycode {
            Keycode::Left | Keycode::A => Action::Left,
            Keycode::Right | Keycode::D => Action::Right,
            Keycode::Up | Keycode::W => Action::Up,
            Keycode::Down | Keycode::S => Action::Down,
            Keycode::Return => Action::Teleport,
            Keycode::Space => Action::PlaceDynamite,
            _ => Action::None,
        };
    }

    pub fn stop_movement(&mut self) {
        self.last_action = Action::None;
    }
}

impl MovementController for KeyboardMovementController {
    fn get_movement(&self) -> Action {
        self.last_action
    }

    fn tick(&mut self) {
    }
}
