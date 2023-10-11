use crate::types::{ConfiguredPins, ConfiguredSystemPins, RotaryPins, UnconfiguredPins};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use hal::gpio::{self, Input, Pins, PullUp};
use spooky_core;
use spooky_embedded::{
    button_keyboard::ButtonKeyboard, embedded_movement_controller::EmbeddedMovementController,
};

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

pub fn setup_button_keyboard<
    Up: InputPin,
    Down: InputPin,
    Left: InputPin,
    Right: InputPin,
    Dyn: InputPin,
    Tel: InputPin,
>(
    configured_pins: ConfiguredPins<Up, Down, Left, Right, Dyn, Tel>,
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
    button_keyboard: ButtonKeyboard<Up, Down, Left, Right, Dyn, Tel>,
) -> EmbeddedMovementController<Up, Down, Left, Right, Dyn, Tel>
where
    Up: InputPin,
    Down: InputPin,
    Left: InputPin,
    Right: InputPin,
    Dyn: InputPin,
    Tel: InputPin,
{
    let demo_movement_controller =
        spooky_core::demo_movement_controller::DemoMovementController::new(seed_buffer);
    EmbeddedMovementController::new(demo_movement_controller, button_keyboard)
}
