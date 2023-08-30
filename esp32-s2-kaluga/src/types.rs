use hal::gpio::{self, GpioPin, Analog};
use embedded_hal::digital::v2::OutputPin;

// Generic type for unconfigured pins
pub struct UnconfiguredPins<MODE> {
    pub sclk: gpio::Gpio15<MODE>,
    pub mosi: gpio::Gpio9<MODE>,
}

pub struct ConfiguredPins {
    pub adc_pin: GpioPin<Analog, 6>,
}

pub struct ConfiguredSystemPins<Dc: OutputPin, Bckl: OutputPin, Reset: OutputPin> {
    pub dc: Dc,
    pub backlight: Bckl,
    pub reset: Reset,
}
