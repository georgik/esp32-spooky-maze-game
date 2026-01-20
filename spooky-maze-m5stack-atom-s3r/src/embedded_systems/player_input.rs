use bevy_ecs::prelude::*;
use bmi2::interface::I2cInterface;
use spooky_core::events::player::PlayerInputMessage;
use spooky_core::resources::MazeResource;

pub struct AccelerometerResource<I2C> {
    pub sensor: bmi2::Bmi2<I2cInterface<I2C>>,
}

pub fn dispatch_accelerometer_input<I2C>(
    mut accel_res: NonSendMut<AccelerometerResource<I2C>>,
    maze_res: Res<MazeResource>,
    mut event_writer: MessageWriter<PlayerInputMessage>,
) where
    I2C: embedded_hal::i2c::I2c + embedded_hal::i2c::ErrorType,
{
    if let Ok(accel) = accel_res.sensor.get_acc_data() {
        let step = maze_res.maze.tile_width as f32;
        let threshold = 1200; // Note: the BMI270 returns raw values, adjust threshold accordingly.
        let mut dx = 0.0;
        let mut dy = 0.0;

        if accel.x.abs() > threshold {
            dy = if accel.x > 0 { -step } else { step };
        }
        if accel.y.abs() > threshold {
            dx = if accel.y > 0 { -step } else { step };
        }
        if dx.abs() > f32::EPSILON || dy.abs() > f32::EPSILON {
            event_writer.write(PlayerInputMessage { dx, dy });
        }
    }
}
