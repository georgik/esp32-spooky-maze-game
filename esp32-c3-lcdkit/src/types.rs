use embedded_hal::digital::v2::{InputPin, OutputPin};
use hal::gpio;

// Generic type for unconfigured pins
pub struct UnconfiguredPins<MODE> {
    pub sclk: gpio::Gpio1<MODE>,
    pub mosi: gpio::Gpio0<MODE>,
    pub miso: gpio::Gpio4<MODE>,
    pub sda: gpio::Gpio3<MODE>,
    // pub scl: gpio::Gpio6<MODE>,
    pub cs: gpio::Gpio7<MODE>,
}

pub struct RotaryPins<DT: InputPin, CLK: InputPin, SW: InputPin>
{
    pub dt: DT,
    pub clk: CLK,
    pub switch: SW,
}

pub struct ConfiguredSystemPins<Dc: OutputPin, Bckl: OutputPin, Reset: OutputPin> {
    pub dc: Dc,
    pub backlight: Bckl,
    pub reset: Reset,
}
