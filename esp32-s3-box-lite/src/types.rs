use hal::gpio;
use embedded_hal::digital::v2::{ InputPin, OutputPin };

// Generic type for unconfigured pins
pub struct UnconfiguredPins<MODE> {
    pub sclk: gpio::Gpio7<MODE>,
    pub mosi: gpio::Gpio6<MODE>,
    pub miso: gpio::Gpio19<MODE>,
    pub cs: gpio::Gpio5<MODE>,
    pub sda: gpio::Gpio8<MODE>,
    pub scl: gpio::Gpio18<MODE>,
}

pub struct ConfiguredPins<Up: InputPin, Down: InputPin, Left: InputPin, Right: InputPin, Dyn: InputPin, Tel: InputPin> {
    pub up_button: Up,
    pub down_button: Down,
    pub left_button: Left,
    pub right_button: Right,
    pub dynamite_button: Dyn,
    pub teleport_button: Tel,
}

pub struct ConfiguredSystemPins<Dc: OutputPin, Bckl: OutputPin, Reset: OutputPin> {
    pub dc: Dc,
    pub backlight: Bckl,
    pub reset: Reset,
}
