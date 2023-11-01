use crate::types::{UnconfiguredPins, ConfiguredSystemPins};
use embedded_hal::digital::v2::OutputPin;
use hal::gpio::{self, Pins};

pub fn setup_pins(pins: Pins) -> (UnconfiguredPins<gpio::Unknown>, /*ConfiguredPins<impl InputPin, impl InputPin, impl InputPin, impl InputPin, impl InputPin,
    impl InputPin>, */ConfiguredSystemPins<impl OutputPin, impl OutputPin, impl OutputPin>) {
            let unconfigured_pins = UnconfiguredPins {
        sclk: pins.gpio0,
        mosi: pins.gpio6,
        miso: pins.gpio11,
        cs: pins.gpio5,
        sda: pins.gpio10,
        scl: pins.gpio8,
    };

    // let configured_pins = ConfiguredPins {
    //     up_button: pins.gpio14.into_pull_up_input(),
    //     down_button: pins.gpio12.into_pull_up_input(),
    //     left_button: pins.gpio13.into_pull_up_input(),
    //     right_button: pins.gpio15.into_pull_up_input(),
    //     dynamite_button: pins.gpio26.into_pull_up_input(),
    //     teleport_button: pins.gpio27.into_pull_up_input(),
    // };

    let configured_system_pins = ConfiguredSystemPins {
        dc: pins.gpio4.into_push_pull_output(),
        backlight: pins.gpio1.into_push_pull_output(),
        reset: pins.gpio3.into_push_pull_output(),
    };

    (unconfigured_pins, /*configured_pins, */configured_system_pins)
}
