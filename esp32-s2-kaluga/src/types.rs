use hal::{gpio::{self, GpioPin, Analog}, adc::{ADC, AdcPin, ADC1}};
use embedded_hal::digital::v2::{ InputPin, OutputPin };

// Generic type for unconfigured pins
pub struct UnconfiguredPins<MODE> {
    pub sclk: gpio::Gpio15<MODE>,
    pub mosi: gpio::Gpio9<MODE>,
}

pub struct ConfiguredPins {
    pub adc_pin: AdcPin<GpioPin<Analog, 6>, ADC1>,
}

pub struct ConfiguredSystemPins<Dc: OutputPin, Bckl: OutputPin, Reset: OutputPin> {
    pub dc: Dc,
    pub backlight: Bckl,
    pub reset: Reset,
}
