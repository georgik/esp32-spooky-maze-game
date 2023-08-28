use embedded_hal::digital::v2::InputPin;

pub enum ButtonEvent {
    UpPressed,
    DownPressed,
    LeftPressed,
    RightPressed,
    DynamitePressed,
    TeleportPressed,
    NoEvent,
}

pub (crate) struct ButtonKeyboard<U, D, L, R, Dy, T>
where
    U: InputPin,
    D: InputPin,
    L: InputPin,
    R: InputPin,
    Dy: InputPin,
    T: InputPin,
{
    up_button: U,
    down_button: D,
    left_button: L,
    right_button: R,
    dynamite_button: Dy,
    teleport_button: T,
}

impl<U, D, L, R, Dy, T> ButtonKeyboard<U, D, L, R, Dy, T>
where
    U: InputPin,
    D: InputPin,
    L: InputPin,
    R: InputPin,
    Dy: InputPin,
    T: InputPin,
{
    // ... Initialization and other methods remain the same

    pub fn poll(&self) -> ButtonEvent {
        // Replace the following example code with your actual polling logic
        if self.up_button.is_low().unwrap() {
            ButtonEvent::UpPressed
        } else if self.down_button.is_low().unwrap() {
            ButtonEvent::DownPressed
        } // ... More conditions here
        else {
            ButtonEvent::NoEvent
        }
    }
}
