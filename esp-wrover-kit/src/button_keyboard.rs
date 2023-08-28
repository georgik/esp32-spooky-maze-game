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

pub (crate) struct ButtonKeyboard<Up: InputPin, Down: InputPin, Left: InputPin, Right: InputPin, Dyn: InputPin, Tel: InputPin> {
    pub up_button: Up,
    pub down_button: Down,
    pub left_button: Left,
    pub right_button: Right,
    pub dynamite_button: Dyn,
    pub teleport_button: Tel,
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
    pub fn new(up_button: U, down_button: D, left_button: L, right_button: R, dynamite_button: Dy, teleport_button: T) -> Self {
        Self {
            up_button,
            down_button,
            left_button,
            right_button,
            dynamite_button,
            teleport_button,
        }
    }

    pub fn poll(&self) -> ButtonEvent {
        // Replace the following example code with your actual polling logic
        if self.up_button.is_low().unwrap_or(false) {
            ButtonEvent::UpPressed
        } else if self.down_button.is_low().unwrap_or(false) {
            ButtonEvent::DownPressed
        } // ... More conditions here
        else {
            ButtonEvent::NoEvent
        }
    }
}
