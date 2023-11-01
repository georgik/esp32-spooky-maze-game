use crate::types::{ConfiguredSystemPins, RotaryPins, UnconfiguredPins};
use embedded_hal::digital::v2::OutputPin;
use hal::gpio::{self, Input, Pins, PullUp};

pub fn setup_pins(
    pins: Pins,
) -> (
    UnconfiguredPins<gpio::Unknown>,
    ConfiguredSystemPins<impl OutputPin, impl OutputPin, impl OutputPin>,
    RotaryPins<
        hal::gpio::GpioPin<Input<PullUp>, 10>,
        hal::gpio::GpioPin<Input<PullUp>, 6>,
        hal::gpio::GpioPin<Input<PullUp>, 9>,
    >,
) {
    let unconfigured_pins = UnconfiguredPins {
        sclk: pins.gpio1,
        mosi: pins.gpio0,
        miso: pins.gpio4,
        sda: pins.gpio3,
        // scl: pins.gpio6,
        cs: pins.gpio7,
    };

    let configured_system_pins = ConfiguredSystemPins {
        dc: pins.gpio2.into_push_pull_output(),
        backlight: pins.gpio5.into_push_pull_output(),
        reset: pins.gpio8.into_push_pull_output(),
    };

    let rotary_pins = RotaryPins {
        dt: pins.gpio10.into_pull_up_input(),
        clk: pins.gpio6.into_pull_up_input(),
        switch: pins.gpio9.into_pull_up_input(),
    };

    (
        unconfigured_pins,
        /*configured_pins, */ configured_system_pins,
        rotary_pins,
    )
}
