use bevy_ecs::prelude::*;
use mpu6886::Mpu6886;
use spooky_core::events::player::PlayerInputMessage;
use spooky_core::resources::MazeResource;

pub struct AccelerometerResource<I2C> {
    pub sensor: Mpu6886<I2C>,
}

pub fn dispatch_accelerometer_input<I2C>(
    mut accel_res: NonSendMut<AccelerometerResource<I2C>>,
    maze_res: Res<MazeResource>,
    mut event_writer: MessageWriter<PlayerInputMessage>,
) where
    I2C: embedded_hal::i2c::I2c + embedded_hal::i2c::ErrorType,
{
    if let Ok(accel) = accel_res.sensor.get_acc() {
        let step = maze_res.maze.tile_width as f32;

        // Threshold for accelerometer control (lowered for easier control)
        // MPU6886 values are normalized (in g units), not raw values
        let threshold = 0.15;
        let mut dx = 0.0;
        let mut dy = 0.0;

        // MPU6886 has X and Y swapped compared to the device orientation
        // Based on original Mpu6886Wrapper in spooky-embedded:
        //   x: measurement.y, y: measurement.x
        //
        // Original mapping (with swapped axes):
        //   accel_norm.y (measurement.x) > threshold → Left
        //   accel_norm.y (measurement.x) < -threshold → Right
        //   accel_norm.x (measurement.y) > threshold → Down
        //   accel_norm.x (measurement.y) < -threshold → Up

        // measurement.x controls left/right
        if accel.x.abs() > threshold {
            dx = if accel.x > 0.0 { -step } else { step };
        }
        // measurement.y controls up/down
        if accel.y.abs() > threshold {
            dy = if accel.y > 0.0 { step } else { -step };
        }

        // Send movement if threshold exceeded
        if dx.abs() > f32::EPSILON || dy.abs() > f32::EPSILON {
            event_writer.write(PlayerInputMessage { dx, dy });
        }
    }
}
