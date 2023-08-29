use crate::types::{UnconfiguredPins, ConfiguredPins, ConfiguredSystemPins};
use embedded_hal::digital::v2::{OutputPin, InputPin};
use hal::gpio::{self, Pins};
use spooky_embedded::{ button_keyboard::ButtonKeyboard, embedded_movement_controller::EmbeddedMovementController };
use spooky_core;

pub fn setup_pins(pins: Pins) -> (UnconfiguredPins<gpio::Unknown>, ConfiguredPins<impl InputPin, impl InputPin, impl InputPin, impl InputPin, impl InputPin,
    impl InputPin>, ConfiguredSystemPins<impl OutputPin, impl OutputPin, impl OutputPin>) {
            let unconfigured_pins = UnconfiguredPins {
        sclk: pins.gpio19,
        mosi: pins.gpio23,
        miso: pins.gpio25,
        cs: pins.gpio22,
    };

    let configured_pins = ConfiguredPins {
        up_button: pins.gpio14.into_pull_up_input(),
        down_button: pins.gpio12.into_pull_up_input(),
        left_button: pins.gpio13.into_pull_up_input(),
        right_button: pins.gpio15.into_pull_up_input(),
        dynamite_button: pins.gpio26.into_pull_up_input(),
        teleport_button: pins.gpio27.into_pull_up_input(),
    };

    let configured_system_pins = ConfiguredSystemPins {
        dc: pins.gpio21.into_push_pull_output(),
        backlight: pins.gpio5.into_push_pull_output(),
        reset: pins.gpio18.into_push_pull_output(),
    };

    (unconfigured_pins, configured_pins, configured_system_pins)
}

pub fn setup_button_keyboard<Up: InputPin, Down: InputPin, Left: InputPin, Right: InputPin, Dyn: InputPin, Tel: InputPin>(
    configured_pins: ConfiguredPins<Up, Down, Left, Right, Dyn, Tel>
) -> ButtonKeyboard<Up, Down, Left, Right, Dyn, Tel> {
    ButtonKeyboard::new(
        configured_pins.up_button,
        configured_pins.down_button,
        configured_pins.left_button,
        configured_pins.right_button,
        configured_pins.dynamite_button,
        configured_pins.teleport_button,
    )
}

pub fn setup_movement_controller<Up, Down, Left, Right, Dyn, Tel>(
    seed_buffer: [u8; 32],
    button_keyboard: ButtonKeyboard<Up, Down, Left, Right, Dyn, Tel>
) -> EmbeddedMovementController<Up, Down, Left, Right, Dyn, Tel>
where
    Up: InputPin,
    Down: InputPin,
    Left: InputPin,
    Right: InputPin,
    Dyn: InputPin,
    Tel: InputPin,
{
    let demo_movement_controller = spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    EmbeddedMovementController::new(demo_movement_controller, button_keyboard)
}