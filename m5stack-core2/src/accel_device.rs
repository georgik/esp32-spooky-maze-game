// External crate imports
use embedded_hal::blocking::i2c::{Write, WriteRead};

// Local imports
#[cfg(feature = "mpu6886")]
use mpu6886::{Mpu6886, Mpu6886Error};

pub trait AccelDevice {
    type Error;

    fn accel_norm(&mut self) -> Result<AccelNorm, Self::Error>;
}

#[derive(Debug, Clone, Copy)]
pub struct AccelNorm {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// Wrapper for Mpu6886
#[cfg(feature = "mpu6886")]
pub struct Mpu6886Wrapper<I>(Mpu6886<I>);

// Existing impl block for AccelDevice
#[cfg(feature = "mpu6886")]
impl<I, E> AccelDevice for Mpu6886Wrapper<I>
where
    I: WriteRead<Error = E> + Write<Error = E>,
{
    type Error = Mpu6886Error<E>;

    fn accel_norm(&mut self) -> Result<AccelNorm, Self::Error> {
        let measurement = self.0.get_acc()?;
        Ok(AccelNorm {
            x: measurement.x as f32,
            y: measurement.y as f32,
            z: measurement.z as f32,
        })
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

    pub fn init<D>(&mut self, delay: &mut D) -> Result<(), Mpu6886Error<E>>
    where
        D: embedded_hal::blocking::delay::DelayMs<u8>,
    {
        self.0.init(delay)
    }
}
