use spooky_core::engine::Action;
use embedded_hal::digital::v2::InputPin;
use spooky_core::movement_controller::MovementController;

pub struct ButtonMovementController<U, D, L, R, Dy, T>
where
    U: InputPin,
    D: InputPin,
    L: InputPin,
    R: InputPin,
    Dy: InputPin,
    T: InputPin,
{
    up: U,
    down: D,
    left: L,
    right: R,
    dynamite: Dy,
    teleport: T,
    last_action: Action,
}

impl<U, D, L, R, Dy, T> ButtonMovementController<U, D, L, R, Dy, T>
where
    U: InputPin,
    D: InputPin,
    L: InputPin,
    R: InputPin,
    Dy: InputPin,
    T: InputPin,
{
    pub fn new(
        up: U,
        down: D,
        left: L,
        right: R,
        dynamite: Dy,
        teleport: T,
    ) -> Self {
        Self {
            up,
            down,
            left,
            right,
            dynamite,
            teleport,
            last_action: Action::None,
        }
    }

    fn move_up(&self) -> bool {
        match self.up.is_low() {
            Ok(value) => value,
            Err(_) => false,
        }
    }

    fn move_down(&self) -> bool {
        match self.down.is_low() {
            Ok(value) => value,
            Err(_) => false,
        }
    }

    fn move_left(&self) -> bool {
        match self.left.is_low() {
            Ok(value) => value,
            Err(_) => false,
        }
    }

    fn move_right(&self) -> bool {
        match self.right.is_low() {
            Ok(value) => value,
            Err(_) => false,
        }
    }

    fn place_dynamite(&self) -> bool {
        match self.dynamite.is_low() {
            Ok(value) => value,
            Err(_) => false,
        }
    }

    fn perform_teleport(&self) -> bool {
        match self.teleport.is_low() {
            Ok(value) => value,
            Err(_) => false,
        }
    }
}

impl<U, D, L, R, Dy, T> MovementController for ButtonMovementController<U, D, L, R, Dy, T>
where
    U: InputPin,
    D: InputPin,
    L: InputPin,
    R: InputPin,
    Dy: InputPin,
    T: InputPin,
{
    fn set_active(&mut self, _index: usize) {
    }

    fn get_movement(&self) -> Action {
        self.last_action
    }

    fn tick(&mut self) {
        if self.move_up() {
            self.last_action = Action::Up;
        } else if self.move_down() {
            self.last_action = Action::Down;
        } else if self.move_left() {
            self.last_action = Action::Left;
        } else if self.move_right() {
            self.last_action = Action::Right;
        } else if self.place_dynamite() {
            self.last_action = Action::PlaceDynamite;
        } else if self.perform_teleport() {
            self.last_action = Action::Teleport;
        } else {
            self.last_action = Action::None;
        }
    }
}
