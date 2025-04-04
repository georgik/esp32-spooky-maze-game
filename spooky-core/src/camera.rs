#[cfg(feature = "std")]
pub type SpookyCamera = bevy::prelude::Camera2d;

#[cfg(not(feature = "std"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct SpookyCamera;
