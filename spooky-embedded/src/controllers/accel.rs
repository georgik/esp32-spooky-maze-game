use spooky_core::engine::Action;
use spooky_core::movement_controller::MovementController;
use icm42670::accelerometer::Accelerometer;
use log::debug;

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
    fn set_active(&mut self, _index:usize) {
    }

    fn get_movement(&self) -> Action {
        self.last_action
    }

    fn tick(&mut self) {
        if let Ok(accel_norm) = self.icm.accel_norm() {
            debug!("X: {}, Y: {}", accel_norm.x, accel_norm.y);
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
        } else {
            debug!("Error reading accelerometer");
        }
    }
}

#[cfg(feature = "mpu6886")]
use embedded_hal::blocking::i2c::{Write, WriteRead};

#[cfg(feature = "mpu6886")]
use mpu6886::{Mpu6886, Mpu6886Error};

// Wrapper for Mpu6886
#[cfg(feature = "mpu6886")]
pub struct Mpu6886Wrapper<I>(Mpu6886<I>);

// Existing impl block for AccelDevice
#[cfg(feature = "mpu6886")]
impl<I, E> Accelerometer for Mpu6886Wrapper<I>
where
    I: WriteRead<Error = E> + Write<Error = E>, E: core::fmt::Debug
{
    type Error = Mpu6886Error<E>;

    fn accel_norm(&mut self) -> Result<icm42670::accelerometer::vector::F32x3, icm42670::accelerometer::Error<Self::Error>> {
        let measurement = self.0.get_acc()?;
        Ok(icm42670::accelerometer::vector::F32x3 {
            x: measurement.y as f32, // X and Y are swapped on the MPU6886
            y: measurement.x as f32,
            z: measurement.z as f32,
        })
    }

    fn sample_rate(&mut self) -> Result<f32, icm42670::accelerometer::Error<Self::Error>> {
        todo!()
    }
}

// Separate impl block for initialization
#[cfg(feature = "mpu6886")]
impl<I, E> Mpu6886Wrapper<I>
where
    I: WriteRead<Error = E> + Write<Error = E>,
{
    pub fn new(inner: Mpu6886<I>) -> Self {
        Self(inner)
    }
}
