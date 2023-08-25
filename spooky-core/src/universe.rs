use crate::engine::{Engine, Action};
use embedded_graphics::{
    prelude::DrawTarget,
    pixelcolor::Rgb565,
};

pub trait MovementController {
    fn get_movement(&self) -> crate::engine::Action;
}

pub struct NoMovementController;
impl MovementController for NoMovementController {
    fn get_movement(&self) -> crate::engine::Action {
        Action::None
    }
}

pub struct Universe<D, M = NoMovementController>
where
    M: MovementController,
{
    pub engine: Engine<D>,
    movement_controller: Option<M>,
}

impl<D: DrawTarget<Color = Rgb565>> Universe<D, NoMovementController> {
    pub fn new_without_movement_controller(engine: Engine<D>) -> Universe<D, NoMovementController> {
        Universe {
            engine,
            movement_controller: None,
        }
    }
}

impl<D: DrawTarget<Color = Rgb565>, M> Universe<D, M>
where
    M: MovementController,
{
    pub fn new_with_movement_controller(engine: Engine<D>, movement_controller: M) -> Universe<D, M> {
        Universe {
            engine,
            movement_controller: Some(movement_controller),
        }
    }

    pub fn initialize(&mut self) {
        self.engine.initialize();
        self.engine.start();
    }

    pub fn render_frame(&mut self) -> &D {
        if let Some(controller) = &self.movement_controller {
            let movement = controller.get_movement();
            match movement {
                Action::Up => self.engine.action(Action::Up),
                Action::Down => self.engine.action(Action::Down),
                Action::Left => self.engine.action(Action::Left),
                Action::Right => self.engine.action(Action::Right),
                Action::Teleport => self.engine.action(Action::Teleport),
                Action::PlaceDynamite => self.engine.action(Action::PlaceDynamite),
                Action::None => {}
            }
        }

        self.engine.tick();
        self.engine.draw()
    }
}
