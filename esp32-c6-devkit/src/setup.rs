use crate::types::{UninitializedPins, ConfiguredPins, ConfiguredSystemPins};
use embedded_hal::digital::v2::{OutputPin, InputPin};
use hal::gpio::{self, Pins};

pub fn setup_pins(pins: Pins) -> (UninitializedPins<gpio::Unknown>, ConfiguredPins, ConfiguredSystemPins<impl OutputPin, impl OutputPin, impl OutputPin>) {
    let unconfigured_pins = UninitializedPins {
        sclk: pins.gpio6,
        mosi: pins.gpio7,
        miso: pins.gpio0,
        cs: pins.gpio20,
    };

    let configured_pins = ConfiguredPins {
        adc_pin: pins.gpio2.into_analog(),
    };


    let configured_system_pins = ConfiguredSystemPins {
        dc: pins.gpio21.into_push_pull_output(),
        backlight: pins.gpio4.into_push_pull_output(),
        reset: pins.gpio3.into_push_pull_output(),
    };

    (unconfigured_pins, configured_pins, configured_system_pins)
}
