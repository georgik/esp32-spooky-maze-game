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

        // Threshold for accelerometer control
        let threshold = 600.0;
        let mut dx = 0.0;
        let mut dy = 0.0;

        // accel.x → dx (horizontal), accel.y → dy (vertical)
        // Fixed direction: positive accel should move in positive direction
        // X axis inverted for left/right
        if accel.x.abs() > threshold {
            dx = if accel.x > 0.0 { -step } else { step };
        }
        if accel.y.abs() > threshold {
            dy = if accel.y > 0.0 { step } else { -step };
        }

        // Send movement if threshold exceeded
        if dx.abs() > f32::EPSILON || dy.abs() > f32::EPSILON {
            event_writer.write(PlayerInputMessage { dx, dy });
        }
    }
}
