#![cfg_attr(not(feature = "std"), no_std)]
pub mod app;
pub mod button_keyboard;
pub mod controllers {
    #[cfg(any(feature = "icm42670", feature = "mpu6886"))]
    pub mod accel;
    pub mod button;
    pub mod composites {
        #[cfg(any(feature = "esp32s2", feature = "esp32c6"))]
        pub mod ladder_composite;
        #[cfg(any(feature = "icm42670", feature = "mpu6886"))]
        pub mod accel_composite;
    }
    pub mod embedded;
    #[cfg(any(feature = "esp32s2", feature = "esp32c6"))]
    pub mod ladder;
}
pub mod embedded_display;
