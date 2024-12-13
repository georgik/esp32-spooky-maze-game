use crate::controllers::accel::AccelMovementController;
use icm42670::accelerometer::Accelerometer;
use spooky_core::demo_movement_controller::DemoMovementController;
use spooky_core::engine::Action;
use spooky_core::movement_controller::MovementController;

pub struct AccelCompositeController<I>
where
    I: Accelerometer,
{
    demo_controller: DemoMovementController,
    accel_controller: AccelMovementController<I>,
    active_index: usize, // 0 for demo_controller, 1 for accel_controller
    last_action: Action,
    last_accel_action: Action,
}

impl<I> AccelCompositeController<I>
where
    I: Accelerometer,
{
    pub fn new(
        demo_controller: DemoMovementController,
        accel_controller: AccelMovementController<I>,
    ) -> Self {
        Self {
            demo_controller,
            accel_controller,
            active_index: 0,
            last_action: Action::None,
            last_accel_action: Action::None,
        }
    }
}

impl<I> MovementController for AccelCompositeController<I>
where
    I: Accelerometer,
{
    fn tick(&mut self) {
        self.last_action = Action::None;
        match self.active_index {
            0 => {
                self.accel_controller.tick();
                let current_accel_action = self.accel_controller.get_movement();
                // Initialization state of accelerometer
                if self.last_accel_action == Action::None {
                    self.last_accel_action = current_accel_action;
                } else if self.last_accel_action != current_accel_action {
                    // 2nd change of state, we consider it as signal to start the game mode
                    self.last_action = Action::Start;
                    self.set_active(1);
                }
                self.demo_controller.tick()
            }
            1 => self.accel_controller.tick(),
            _ => {}
        }
    }

    fn get_movement(&self) -> Action {
        if self.last_action != Action::None {
            return self.last_action;
        }

        match self.active_index {
            0 => self.demo_controller.get_movement(),
            1 => self.accel_controller.get_movement(),
            _ => Action::None,
        }
    }

    fn set_active(&mut self, index: usize) {
        self.active_index = index;
    }
}
