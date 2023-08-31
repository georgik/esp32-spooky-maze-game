use crate::engine::{Engine, Action};
use crate::movement_controller::MovementController;
use embedded_graphics::{
    prelude::DrawTarget,
    pixelcolor::Rgb565,
};

pub struct Universe<D, M>
where
    D: DrawTarget<Color = Rgb565>,
    M: MovementController,
{
    pub engine: Engine<D>,
    movement_controller: M,
}

impl<D: DrawTarget<Color = Rgb565>, M: MovementController> Universe<D, M> {
    pub fn new_with_movement_controller(engine: Engine<D>, movement_controller: M) -> Self {
        Universe {
            engine,
            movement_controller,
        }
    }

    pub fn set_active(&mut self, index:usize) {
        self.movement_controller.set_active(index);
    }

    pub fn get_movement_controller_mut(&mut self) -> &mut M {
        &mut self.movement_controller
    }

    pub fn initialize(&mut self) {
        self.engine.initialize();
        self.engine.start();
    }

    pub fn render_frame(&mut self) -> &D {
        self.movement_controller.tick();
        let movement = self.movement_controller.get_movement();
        match movement {
            Action::Up => self.engine.action(Action::Up),
            Action::Down => self.engine.action(Action::Down),
            Action::Left => self.engine.action(Action::Left),
            Action::Right => self.engine.action(Action::Right),
            Action::Teleport => self.engine.action(Action::Teleport),
            Action::PlaceDynamite => self.engine.action(Action::PlaceDynamite),
            Action::Start => self.engine.action(Action::Start),
            Action::Stop => self.engine.action(Action::Stop),
            Action::None => {}
        }

        self.engine.tick();
        self.engine.draw()
    }
}
