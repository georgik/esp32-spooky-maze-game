use crate::types::{UninitializedPins, ConfiguredPins, ConfiguredSystemPins};
use embedded_hal::digital::v2::OutputPin;
use hal::gpio::{self, Pins};

pub fn setup_pins(pins: Pins) -> (UninitializedPins<gpio::Unknown>, ConfiguredPins, ConfiguredSystemPins<impl OutputPin, impl OutputPin, impl OutputPin>) {
    let unconfigured_pins = UninitializedPins {
        sclk: pins.gpio15,
        mosi: pins.gpio9,
        miso: pins.gpio8,
        cs: pins.gpio11,
    };

    let configured_pins = ConfiguredPins {
        adc_pin: pins.gpio6.into_analog(),
    };

    let configured_system_pins = ConfiguredSystemPins {
        dc: pins.gpio13.into_push_pull_output(),
        backlight: pins.gpio5.into_push_pull_output(),
        reset: pins.gpio16.into_push_pull_output(),
    };

    (unconfigured_pins, configured_pins, configured_system_pins)
}
