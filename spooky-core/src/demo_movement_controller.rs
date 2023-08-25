use crate::engine::Action;
use crate::movement_controller::MovementController;
use rand::prelude::*;
use rand_chacha::ChaChaRng;

pub struct DemoMovementController {
    rng: ChaChaRng,
    last_action: Action,
}

impl DemoMovementController {
    pub fn new(seed: [u8; 32]) -> Self {
        DemoMovementController {
            rng: ChaChaRng::from_seed(seed),
            last_action: Action::None,
        }
    }

    fn get_rand(&mut self) -> i32 {
        self.rng.gen_range(0..4)
    }

}

impl MovementController for DemoMovementController {

    fn set_active(&mut self, _index:usize) {
    }

    fn get_movement(&self) -> Action {
        self.last_action
    }

    fn tick(&mut self) {
        let random_number = self.get_rand();

        self.last_action = match random_number {
            0 => Action::Up,
            1 => Action::Down,
            2 => Action::Left,
            3 => Action::Right,
            _ => Action::None,
        };
    }


}
