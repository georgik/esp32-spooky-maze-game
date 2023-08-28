use spooky_embedded::button_keyboard::{ButtonKeyboard, ButtonEvent};
use embedded_hal::digital::v2::InputPin;
use crate::ConfiguredPins;

pub struct WroverButtonKeyboard<Up, Down, Left, Right, Dyn, Tel>
where
    Up: InputPin,
    Down: InputPin,
    Left: InputPin,
    Right: InputPin,
    Dyn: InputPin,
    Tel: InputPin,
{
    inner: ButtonKeyboard<Up, Down, Left, Right, Dyn, Tel>,
}

impl<Up, Down, Left, Right, Dyn, Tel> WroverButtonKeyboard<Up, Down, Left, Right, Dyn, Tel>
where
    Up: InputPin,
    Down: InputPin,
    Left: InputPin,
    Right: InputPin,
    Dyn: InputPin,
    Tel: InputPin,
{
    pub fn new(configured_pins: ConfiguredPins<Up, Down, Left, Right, Dyn, Tel>) -> Self {
        let button_keyboard = ButtonKeyboard::new(
            configured_pins.up_button,
            configured_pins.down_button,
            configured_pins.left_button,
            configured_pins.right_button,
            configured_pins.dynamite_button,
            configured_pins.teleport_button,
        );
        Self { inner: button_keyboard }
    }

    pub fn poll(&self) -> ButtonEvent {
        self.inner.poll()
    }
}
