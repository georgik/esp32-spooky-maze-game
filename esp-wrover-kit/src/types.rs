use embedded_hal::digital::v2::InputPin;

pub struct ConfiguredPins<Up: InputPin, Down: InputPin, Left: InputPin, Right: InputPin, Dyn: InputPin, Tel: InputPin> {
    pub up_button: Up,
    pub down_button: Down,
    pub left_button: Left,
    pub right_button: Right,
    pub dynamite_button: Dyn,
    pub teleport_button: Tel,
}
