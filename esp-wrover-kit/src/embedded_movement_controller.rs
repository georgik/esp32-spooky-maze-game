use spooky_core::movement_controller::MovementController;
use spooky_core::engine::Action;
use spooky_core::demo_movement_controller::DemoMovementController;
use crate::button_movement_controller::ButtonMovementController;
use crate::button_keyboard::ButtonEvent;
use crate::wrover_keyboard::WroverButtonKeyboard;
use embedded_hal::digital::v2::InputPin;

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
    button_movement_controller: ButtonMovementController,
    wrover_button_keyboard: WroverButtonKeyboard<Up, Down, Left, Right, Dyn, Tel>,
    active_index: usize, // 0: demo_movement_controller, 1: button_movement_controller
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
        button_movement_controller: ButtonMovementController,
        wrover_button_keyboard:  WroverButtonKeyboard<Up, Down, Left, Right, Dyn, Tel>,
    ) -> Self {
        Self {
            demo_movement_controller,
            button_movement_controller,
            wrover_button_keyboard,
            active_index: 0, // Initially, demo is active
            last_action: Action::None
        }
    }

    pub fn get_active_index(&self) -> usize {
        self.active_index
    }


}

impl<Up, Down, Left, Right, Dyn, Tel> MovementController for EmbeddedMovementController<Up, Down, Left, Right, Dyn, Tel>
where
    Up: InputPin,
    Down: InputPin,
    Left: InputPin,
    Right: InputPin,
    Dyn: InputPin,
    Tel: InputPin,
{
    fn set_active(&mut self, index: usize) {
        self.active_index = index;
    }

    fn tick(&mut self) {
        self.last_action = Action::None;
        match self.active_index {
            0 => {
                // self.wrover_button_keyboard.poll();
                if self.wrover_button_keyboard.poll() != ButtonEvent::NoEvent {
                    self.active_index = 1;
                    self.last_action = Action::Start;
                }
                self.demo_movement_controller.tick()
            },
            1 => {
                self.button_movement_controller.react_to_event(self.wrover_button_keyboard.poll());
            }
            _ => {},
        }
    }

    fn get_movement(&self) -> Action {
        if self.last_action != Action::None {
            return self.last_action;
        }

        match self.active_index {
            0 => self.demo_movement_controller.get_movement(),
            1 => self.button_movement_controller.get_movement(),
            _ => Action::None,
        }
    }
}
