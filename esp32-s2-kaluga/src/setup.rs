use crate::types::{UnconfiguredPins, ConfiguredPins, ConfiguredSystemPins};
use embedded_hal::digital::v2::OutputPin;
use hal::gpio::{self, Pins};

pub fn setup_pins(pins: Pins) -> (UnconfiguredPins<gpio::Unknown>, ConfiguredPins, ConfiguredSystemPins<impl OutputPin, impl OutputPin, impl OutputPin>) {
    let unconfigured_pins = UnconfiguredPins {
        sclk: pins.gpio15,
        mosi: pins.gpio9,
    };

    let configured_pins = ConfiguredPins {
        adc_pin: pins.gpio6.into_analog(),
    };

    let configured_system_pins = ConfiguredSystemPins {
        dc: pins.gpio13.into_push_pull_output(),
        backlight: pins.gpio5.into_push_pull_output(),
        reset: pins.gpio18.into_push_pull_output(),
    };

    (unconfigured_pins, configured_pins, configured_system_pins)
}



// pub fn setup_adc() -> Adc<ADC1> {
//     let mut adc1 = Adc::new(ADC1, AdcConfig::default());
//     adc1.set_attenuation(Attenuation::DB0);
//     adc1
// }

// pub fn setup_button_keyboard<Up: InputPin, Down: InputPin, Left: InputPin, Right: InputPin, Dyn: InputPin, Tel: InputPin>(
//     configured_pins: ConfiguredPins<Up, Down, Left, Right, Dyn, Tel>
// ) -> ButtonKeyboard<Up, Down, Left, Right, Dyn, Tel> {
//     ButtonKeyboard::new(
//         configured_pins.up_button,
//         configured_pins.down_button,
//         configured_pins.left_button,
//         configured_pins.right_button,
//         configured_pins.dynamite_button,
//         configured_pins.teleport_button,
//     )
// }