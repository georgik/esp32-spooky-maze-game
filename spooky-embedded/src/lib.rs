#![cfg_attr(not(feature = "std"), no_std)]
pub mod button_keyboard;
pub mod controllers {
    pub mod accel;
    pub mod button;
    pub mod composites {
        pub mod s3box;
    }
    pub mod embedded;
}
pub mod embedded_display;
