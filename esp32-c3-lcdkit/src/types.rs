use embedded_hal::digital::v2::{InputPin, OutputPin};
use hal::gpio::{self, Input, PullUp};

// Generic type for unconfigured pins
pub struct UnconfiguredPins<MODE> {
    pub sclk: gpio::Gpio1<MODE>,
    pub mosi: gpio::Gpio0<MODE>,
    pub miso: gpio::Gpio4<MODE>,
    pub sda: gpio::Gpio3<MODE>,
    // pub scl: gpio::Gpio6<MODE>,
    pub cs: gpio::Gpio7<MODE>,
}

pub struct ConfiguredPins<
    Up: InputPin,
    Down: InputPin,
    Left: InputPin,
    Right: InputPin,
    Dyn: InputPin,
    Tel: InputPin,
> {
    pub up_button: Up,
    pub down_button: Down,
    pub left_button: Left,
    pub right_button: Right,
    pub dynamite_button: Dyn,
    pub teleport_button: Tel,
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
