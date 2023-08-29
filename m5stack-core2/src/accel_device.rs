// accel_device.rs
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
