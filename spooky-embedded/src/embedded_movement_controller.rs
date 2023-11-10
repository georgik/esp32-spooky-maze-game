use crate::button_keyboard::{ButtonEvent, ButtonKeyboard};
use embedded_hal::digital::v2::InputPin;
use spooky_core::demo_movement_controller::DemoMovementController;
use spooky_core::engine::Action;
use spooky_core::movement_controller::MovementController;

pub struct EmbeddedMovementController<Up, Down, Left, Right, Dyn, Tel>
where
    Up: InputPin,
    Down: InputPin,
    Left: InputPin,
    Right: InputPin,
    Dyn: InputPin,
    Tel: InputPin,
{
    demo_movement_controller: DemoMovementController,
    keyboard: ButtonKeyboard<Up, Down, Left, Right, Dyn, Tel>,
    active_index: usize, // 0: demo_movement_controller, 1: keyboard
    last_action: Action,
}

impl<Up, Down, Left, Right, Dyn, Tel> EmbeddedMovementController<Up, Down, Left, Right, Dyn, Tel>
where
    Up: InputPin,
    Down: InputPin,
    Left: InputPin,
    Right: InputPin,
    Dyn: InputPin,
    Tel: InputPin,
{
    pub fn new(
        demo_movement_controller: DemoMovementController,
        keyboard: ButtonKeyboard<Up, Down, Left, Right, Dyn, Tel>,
    ) -> Self {
        Self {
            demo_movement_controller,
            keyboard,
            active_index: 0,
            last_action: Action::None,
        }
    }

    fn react_to_event(&mut self, event: ButtonEvent) {
        match event {
            ButtonEvent::UpPressed => self.last_action = Action::Up,
            ButtonEvent::DownPressed => self.last_action = Action::Down,
            ButtonEvent::LeftPressed => self.last_action = Action::Left,
            ButtonEvent::RightPressed => self.last_action = Action::Right,
            ButtonEvent::DynamitePressed => self.last_action = Action::PlaceDynamite,
            ButtonEvent::TeleportPressed => self.last_action = Action::Teleport,
            ButtonEvent::NoEvent => self.last_action = Action::None,
        }
    }

    fn poll_keyboard(&mut self) {
        let event = self.keyboard.poll();
        self.react_to_event(event);
    }
}

impl<Up, Down, Left, Right, Dyn, Tel> MovementController
    for EmbeddedMovementController<Up, Down, Left, Right, Dyn, Tel>
where
    Up: InputPin,
    Down: InputPin,
    Left: InputPin,
    Right: InputPin,
    Dyn: InputPin,
    Tel: InputPin,
{
    fn tick(&mut self) {
        self.last_action = Action::None;
        match self.active_index {
            0 => {
                if self.keyboard.poll() != ButtonEvent::NoEvent {
                    self.active_index = 1;
                    self.last_action = Action::Start;
                }
                self.demo_movement_controller.tick();
            }
            1 => self.poll_keyboard(),
            _ => {}
        }
    }

    fn get_movement(&self) -> Action {
        if self.last_action != Action::None {
            return self.last_action;
        }

        match self.active_index {
            0 => self.demo_movement_controller.get_movement(),
            1 => self.last_action,
            _ => Action::None,
        }
    }

    fn set_active(&mut self, index: usize) {
        self.active_index = index;
    }
}
