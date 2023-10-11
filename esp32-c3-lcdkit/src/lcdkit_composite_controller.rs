use spooky_core::engine::Action;
use spooky_core::movement_controller::MovementController;
use spooky_core::demo_movement_controller::DemoMovementController;
use crate::rotary_movement_controller::RotaryMovementController;

pub struct LcdKitCompositeController
{
    demo_controller: DemoMovementController,
    rotary_controller: RotaryMovementController,
    active_index: usize, // 0 for demo_controller, 1 for rotary_controller
    last_action: Action,
    last_rotary_action: Action,
}

impl LcdKitCompositeController
{
    pub fn new(demo_controller: DemoMovementController, rotary_controller: RotaryMovementController) -> Self {
        Self {
            demo_controller,
            rotary_controller,
            active_index: 0,
            last_action: Action::None,
            last_rotary_action: Action::None,
        }
    }

    pub fn set_movement(&mut self, action: Action) {
        self.rotary_controller.set_movement(action);
    }
}

impl MovementController for LcdKitCompositeController
{
    fn tick(&mut self) {
        self.last_action = Action::None;
        match self.active_index {
            0 => {
                self.rotary_controller.tick();
                let current_rotary_action = self.rotary_controller.get_movement();
                // Initialization state of rotary controller
                if self.last_rotary_action == Action::None {
                    self.last_rotary_action = current_rotary_action;
                } else if self.last_rotary_action != current_rotary_action {
                    // 2nd change of state, we consider it as signal to start the game mode
                    self.last_action = Action::Start;
                    self.set_active(1);
                }
                self.demo_controller.tick()
            },
            1 => self.rotary_controller.tick(),
            _ => {}
        }
    }

    fn get_movement(&self) -> Action {
        if self.last_action != Action::None {
            return self.last_action;
        }

        match self.active_index {
            0 => self.demo_controller.get_movement(),
            1 => self.rotary_controller.get_movement(),
            _ => Action::None,
        }
    }

    fn set_active(&mut self, index: usize) {
        self.active_index = index;
    }
}
