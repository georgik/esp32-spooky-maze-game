use spooky_core::engine::Action;
use spooky_core::movement_controller::MovementController;
use crate::accel_device::AccelDevice; // Replace this with the actual path to your AccelDevice trait
use crate::accel_device::AccelNorm; // Replace this with the actual path to your AccelNorm struct

use log::debug;

pub struct AccelMovementController<A>
where
    A: AccelDevice,
{
    icm: A,
    last_action: Action,
    accel_threshold: f32,
}

impl<A> AccelMovementController<A>
where
    A: AccelDevice,
{
    pub fn new(icm: A, accel_threshold: f32) -> Self {
        Self {
            icm,
            last_action: Action::None,
            accel_threshold,
        }
    }

    // This function is no longer in the impl block for MovementController
    fn update_last_action(&mut self, accel_norm: AccelNorm) {
        debug!("Accel values: {}, {}", accel_norm.x, accel_norm.y);
        if accel_norm.x > self.accel_threshold {
            self.last_action = Action::Left;
        } else if accel_norm.x < -self.accel_threshold {
            self.last_action = Action::Right;
        } else if accel_norm.y > self.accel_threshold {
            self.last_action = Action::Down;
        } else if accel_norm.y < -self.accel_threshold {
            self.last_action = Action::Up;
        } else {
            self.last_action = Action::None;
        }
        // Additional actions for Teleport and PlaceDynamite can be added here
    }
}

impl<A> MovementController for AccelMovementController<A>
where
    A: AccelDevice,
{
    fn set_active(&mut self, _index: usize) {
        // Your implementation here
    }

    fn get_movement(&self) -> Action {
        self.last_action
    }

    fn tick(&mut self) {
        match self.icm.accel_norm() {
            Ok(accel_norm) => self.update_last_action(accel_norm),
            Err(_e) => debug!("Error reading accelerometer"),
        }
    }
}
