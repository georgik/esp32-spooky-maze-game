use crate::types::ConfiguredPins;
use embedded_hal::digital::v2::InputPin;
use spooky_embedded::{ button_keyboard::ButtonKeyboard, controllers::embedded::EmbeddedMovementController };
use spooky_core;

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