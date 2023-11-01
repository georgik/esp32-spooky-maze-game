use hal::gpio;
use embedded_hal::digital::v2::OutputPin;

// Generic type for unconfigured pins
pub struct UnconfiguredPins<MODE> {
    pub sclk: gpio::Gpio0<MODE>,
    pub mosi: gpio::Gpio6<MODE>,
    pub miso: gpio::Gpio11<MODE>,
    pub cs: gpio::Gpio5<MODE>,
    pub sda: gpio::Gpio10<MODE>,
    pub scl: gpio::Gpio8<MODE>,
}

pub struct ConfiguredSystemPins<Dc: OutputPin, Bckl: OutputPin, Reset: OutputPin> {
    pub dc: Dc,
    pub backlight: Bckl,
    pub reset: Reset,
}
