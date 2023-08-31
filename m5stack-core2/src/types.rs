use hal::gpio;
use embedded_hal::digital::v2::{ InputPin, OutputPin };

// Generic type for unconfigured pins
pub struct UnconfiguredPins<MODE> {
    pub sclk: gpio::Gpio18<MODE>, // SPI Clock
    pub mosi: gpio::Gpio23<MODE>, // SPI Master Out Slave In
    pub sda: gpio::Gpio21<MODE>,  // I2C Data
    pub scl: gpio::Gpio22<MODE>,  // I2C Clock
}

pub struct ConfiguredPins<Up: InputPin, Down: InputPin, Left: InputPin, Right: InputPin, Dyn: InputPin, Tel: InputPin> {
    pub up_button: Up,        // Button A
    pub down_button: Down,    // Button B
    pub left_button: Left,    // Button C
    pub right_button: Right,  // Button D (if applicable)
    pub dynamite_button: Dyn, // Additional Custom Button (if applicable)
    pub teleport_button: Tel, // Additional Custom Button (if applicable)
}

pub struct ConfiguredSystemPins<Dc: OutputPin, Bckl: OutputPin, Reset: OutputPin> {
    pub dc: Dc,        // Data/Command for Display
    pub backlight: Bckl,  // LCD backlight control
    pub reset: Reset,   // Reset line for any additional modules or for LCD
}
