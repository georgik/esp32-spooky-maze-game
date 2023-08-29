use crate::types::{UnconfiguredPins, ConfiguredPins, ConfiguredSystemPins};
use embedded_hal::digital::v2::{OutputPin, InputPin};
use hal::gpio::{self, Pins};
use spooky_embedded::{ button_keyboard::ButtonKeyboard, embedded_movement_controller::EmbeddedMovementController };
use spooky_core;

pub fn setup_pins(pins: Pins) -> (UnconfiguredPins<gpio::Unknown>, ConfiguredSystemPins<impl OutputPin, impl OutputPin, impl OutputPin>) {
    let unconfigured_pins = UnconfiguredPins {
        sclk: pins.gpio7,
        mosi: pins.gpio6,
        sda: pins.gpio21,  // Updated for M5Stack
        scl: pins.gpio22,  // Updated for M5Stack
    };

    let configured_system_pins = ConfiguredSystemPins {
        dc: pins.gpio4.into_push_pull_output(),
        backlight: pins.gpio45.into_push_pull_output(),
        reset: pins.gpio48.into_push_pull_output(),
    };

    (unconfigured_pins, configured_system_pins)
}

pub fn setup_button_keyboard<Up: InputPin, Down: InputPin, Left: InputPin, Right: InputPin, Dyn: InputPin, Tel: InputPin>(
    pins: Pins  // Passing in the Pins here, or you could pass in the ConfiguredPins if you uncomment that code
) -> ButtonKeyboard<Up, Down, Left, Right, Dyn, Tel> {
    let configured_pins = ConfiguredPins {
        up_button: pins.gpio39.into_pull_up_input(),  // Button A
        down_button: pins.gpio38.into_pull_up_input(),  // Button B
        left_button: pins.gpio37.into_pull_up_input(),  // Button C
        right_button: pins.gpio37.into_pull_up_input(),  // Dummy example; replace appropriately
        dynamite_button: pins.gpio37.into_pull_up_input(),  // Dummy example; replace appropriately
        teleport_button: pins.gpio37.into_pull_up_input(),  // Dummy example; replace appropriately
    };

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
