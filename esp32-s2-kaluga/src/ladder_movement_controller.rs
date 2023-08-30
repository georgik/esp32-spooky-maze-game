use spooky_core::engine::Action;
use spooky_core::movement_controller::MovementController;

pub struct LadderMovementController {
    last_action: Action,
    resistor_value: u32, // Substitute with actual type for the resistor value
}

impl LadderMovementController {
    pub fn new(resistor_value: u32) -> Self { // Substitute with actual type for the resistor value
        Self {
            last_action: Action::None,
            resistor_value,
        }
    }

    fn update_last_action(&mut self) {
        // Update self.last_action based on the specific resistor values for Kaluga
        if self.resistor_value > 4000 && self.resistor_value < 5000 {
            self.last_action = Action::Right;
        } else if self.resistor_value >= 5000 && self.resistor_value < 6000 {
            self.last_action = Action::Left;
        } else if self.resistor_value >= 6000 && self.resistor_value < 7000 {
            self.last_action = Action::Down;
        } else if self.resistor_value >= 7000 && self.resistor_value < 8180 {
            self.last_action = Action::Up;
        } else {
            self.last_action = Action::None;
        }
    }
}

impl MovementController for LadderMovementController {
    fn set_active(&mut self, _index: usize) {
        // Implementation for set_active, if required
    }

    fn get_movement(&self) -> Action {
        self.last_action
    }

    fn tick(&mut self) {
        // Update the last_action based on the latest resistor value
        self.update_last_action();
    }
}
