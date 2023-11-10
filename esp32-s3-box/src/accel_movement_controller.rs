use icm42670::accelerometer::Accelerometer;
use spooky_core::engine::Action;
use spooky_core::movement_controller::MovementController;
pub struct AccelMovementController<I>
where
    I: Accelerometer,
{
    icm: I,
    last_action: Action,
    accel_threshold: f32,
}

impl<I> AccelMovementController<I>
where
    I: Accelerometer,
{
    pub fn new(icm: I, accel_threshold: f32) -> Self {
        Self {
            icm,
            last_action: Action::None,
            accel_threshold,
        }
    }
}

impl<I> MovementController for AccelMovementController<I>
where
    I: Accelerometer,
{
    fn set_active(&mut self, _index: usize) {}

    fn get_movement(&self) -> Action {
        self.last_action
    }

    fn tick(&mut self) {
        if let Ok(accel_norm) = self.icm.accel_norm() {
            if accel_norm.y > self.accel_threshold {
                self.last_action = Action::Left;
            } else if accel_norm.y < -self.accel_threshold {
                self.last_action = Action::Right;
            } else if accel_norm.x > self.accel_threshold {
                self.last_action = Action::Down;
            } else if accel_norm.x < -self.accel_threshold {
                self.last_action = Action::Up;
            } else {
                self.last_action = Action::None;
            }
            // Additional actions for Teleport and PlaceDynamite can be added here
        }
    }
}
