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
        }
    }

    pub fn react_to_event(&mut self) {
        let button_event = self.wrover_button_keyboard.poll();

        match button_event {
            ButtonEvent::TeleportPressed => self.active_index = 0, // Switch to demo controller
            _ => {
                if self.active_index == 1 {
                    self.button_movement_controller.react_to_event(button_event);
                }
            },
        }
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
        match self.active_index {
            0 => self.demo_movement_controller.tick(),
            1 => {}, // No action required, events are processed in react_to_event
            _ => {},
        }
    }

    fn get_movement(&self) -> Action {
        match self.active_index {
            0 => self.demo_movement_controller.get_movement(),
            1 => self.button_movement_controller.get_movement(),
            _ => Action::None,
        }
    }
}
