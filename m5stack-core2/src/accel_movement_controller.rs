use spooky_core::engine::Action;
use spooky_core::movement_controller::MovementController;
#[cfg(feature = "mpu6886")]
use mpu6886::Mpu6886;


pub struct AccelMovementController<A, I>
where
    A: Mpu6886<I>,
{
    icm: A,
    last_action: Action,
    accel_threshold: f32,
}

impl<I> AccelMovementController<I>
where
A: Mpu6886<I>,
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
A: Mpu6886<I>,
{
    fn set_active(&mut self, _index:usize) {
    }

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
