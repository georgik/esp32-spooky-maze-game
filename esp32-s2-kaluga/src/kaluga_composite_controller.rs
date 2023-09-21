use spooky_core::engine::Action;
use spooky_core::movement_controller::MovementController;
use crate::ladder_movement_controller::LadderMovementController;
use spooky_core::demo_movement_controller::DemoMovementController;

pub struct KalugaCompositeController<'a> {
    demo_controller: DemoMovementController,
    ladder_controller: LadderMovementController<'a>,
    active_index: usize, // 0 for demo_controller, 1 for ladder_controller
    last_action: Action,
}

impl<'a> KalugaCompositeController<'a> {
    pub fn new(demo_controller: DemoMovementController, ladder_controller: LadderMovementController<'a>) -> Self {
        Self {
            demo_controller,
            ladder_controller,
            active_index: 0,
            last_action: Action::None,
        }
    }
}

impl MovementController for KalugaCompositeController<'_> {
    fn tick(&mut self) {
        self.last_action = Action::None;

        match self.active_index {
            0 => {
                self.demo_controller.tick();
                self.last_action = self.demo_controller.get_movement();

                self.ladder_controller.tick();
                let ladder_action = self.ladder_controller.get_movement();
                if ladder_action != Action::None {
                    self.set_active(1);
                    self.last_action = Action::Start;
                }
            },
            1 => {
                self.ladder_controller.tick();
                self.last_action = self.ladder_controller.get_movement();
            },
            _ => {}
        }
    }

    fn get_movement(&self) -> Action {
        self.last_action
    }

    fn set_active(&mut self, index: usize) {
        self.active_index = index;
    }
}
