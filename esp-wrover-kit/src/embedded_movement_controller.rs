use embedded_hal::digital::v2::InputPin;
use spooky_core::movement_controller::MovementController;
use crate::button_movement_controller::ButtonMovementController;

pub struct EmbeddedMovementController<U, D, L, R, Dy, T, S>
where
    U: InputPin,
    D: InputPin,
    L: InputPin,
    R: InputPin,
    Dy: InputPin,
    T: InputPin,
    S: InputPin,
{
    pub demo_movement_controller: spooky_core::demo_movement_controller::DemoMovementController,
    pub button_movement_controller: ButtonMovementController<U, D, L, R, Dy, T>,
    pub start_button: S,
    pub(crate) active_index: usize,
}

impl<U, D, L, R, Dy, T, S> EmbeddedMovementController<U, D, L, R, Dy, T, S>
where
    U: InputPin,
    D: InputPin,
    L: InputPin,
    R: InputPin,
    Dy: InputPin,
    T: InputPin,
    S: InputPin,
{
    pub fn new(
        demo_movement_controller: spooky_core::demo_movement_controller::DemoMovementController,
        button_movement_controller: ButtonMovementController<U, D, L, R, Dy, T>,
        start_button: S,
    ) -> Self {
        Self {
            demo_movement_controller,
            button_movement_controller,
            start_button,
            active_index: 0, // Initially, demo is active
        }
    }

    pub fn check_start_button(&mut self) {
        match self.start_button.is_low() {
            Ok(true) => {
                self.active_index = 1;
            },
            _ => (),
        }
    }

    pub fn is_demo(&self) -> bool {
        self.active_index == 0
    }
}

impl<U, D, L, R, Dy, T, S> MovementController for EmbeddedMovementController<U, D, L, R, Dy, T, S>
where
    U: InputPin,
    D: InputPin,
    L: InputPin,
    R: InputPin,
    Dy: InputPin,
    T: InputPin,
    S: InputPin,
{
    fn set_active(&mut self, index: usize) {
        self.active_index = index;
    }

    fn tick(&mut self) {
        self.check_start_button();

        match self.active_index {
            0 => self.demo_movement_controller.tick(),
            1 => self.button_movement_controller.tick(),
            _ => {}
        }
    }

    fn get_movement(&self) -> spooky_core::engine::Action {
        match self.active_index {
            0 => self.demo_movement_controller.get_movement(),
            1 => self.button_movement_controller.get_movement(),
            _ => spooky_core::engine::Action::None,
        }
    }
}
