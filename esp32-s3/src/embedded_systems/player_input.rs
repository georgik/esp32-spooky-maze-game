use spooky_core::events::player::PlayerInputEvent;
use embedded_hal::i2c::I2c;
use bevy_ecs::prelude::*;
use core::fmt::Debug;
use icm42670::prelude::*;
use icm42670::Icm42670;
use spooky_core::resources::MazeResource;
use spooky_core::resources::PlayerPosition;

/// A resource wrapping the accelerometer sensor.
/// (This resource is non‑Send because the sensor’s driver isn’t Sync.)
pub struct AccelerometerResource<I2C> {
    pub sensor: Icm42670<I2C>,
}

/// Reads the accelerometer data and dispatches a PlayerInputEvent
/// if the reading exceeds a threshold. Movement is in one-tile steps.
pub fn dispatch_accelerometer_input<I2C, E>(
    mut accel_res: NonSendMut<AccelerometerResource<I2C>>,
    maze_res: Res<MazeResource>,
    mut event_writer: EventWriter<PlayerInputEvent>,
) where
    I2C: I2c<Error = E>,
    E: Debug,
{
    if let Ok(accel) = accel_res.sensor.accel_norm() {
        let step = maze_res.maze.tile_width as f32;
        let threshold = 0.2;
        let mut dx = 0.0;
        let mut dy = 0.0;

        if accel.x.abs() > threshold {
            dx = if accel.x > 0.0 { step } else { -step };
        }
        if accel.y.abs() > threshold {
            dy = if accel.y > 0.0 { step } else { -step };
        }
        if dx.abs() > f32::EPSILON || dy.abs() > f32::EPSILON {
            event_writer.send(PlayerInputEvent { dx, dy });
        }
    }
}
