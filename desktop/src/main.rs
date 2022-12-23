
// #![no_std]
// #![no_main]

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
};
use embedded_graphics_simulator::{
    sdl2::Keycode, SimulatorDisplay, SimulatorEvent, Window, OutputSettingsBuilder,
};
use embedded_graphics_framebuf::{FrameBuf};

use std::time::Duration;

use spooky_core::{ spritebuf::SpriteBuf, engine::Engine, engine::Action::{ Up, Down, Left, Right, Teleport, PlaceDynamite } };

pub struct Universe<D> {
    engine: Engine<D>,
}

fn get_seed_buffer() -> Option<[u8; 32]> {
    let mut seed_buffer = [0u8; 32];
    getrandom::getrandom(&mut seed_buffer).unwrap();
    Some(seed_buffer)
}

impl <D:embedded_graphics::draw_target::DrawTarget<Color = Rgb565>> Universe <D> {

    pub fn new(engine:Engine<D>) -> Universe<D> {
        Universe {
            engine
        }
    }

    pub fn move_up(&mut self) {
        self.engine.action(Up);
    }

    pub fn move_down(&mut self) {
        self.engine.action(Down);
    }

    pub fn move_left(&mut self) {
        self.engine.action(Left);
    }

    pub fn move_right(&mut self) {
        self.engine.action(Right);
    }

    pub fn teleport(&mut self) {
        self.engine.action(Teleport);
    }

    pub fn initialize(&mut self) {
        self.engine.initialize();
        self.engine.start();
    }

    pub fn place_dynamite(&mut self) {
        self.engine.action(PlaceDynamite);
    }

    pub fn render_frame(&mut self) -> &D {
        self.engine.tick();
        self.engine.draw()
    }
}



fn main() -> Result<(), core::convert::Infallible> {
    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("ESP32 Spooky Maze", &output_settings);

    let mut data = [Rgb565::BLACK ; 320*240];
    let fbuf = FrameBuf::new(&mut data, 320, 240);
    let spritebuf = SpriteBuf::new(fbuf);

    let engine = Engine::new(spritebuf, get_seed_buffer());

    let mut universe = Universe::new(engine);
    universe.initialize();
    let mut display = SimulatorDisplay::new(Size::new(320, 200));


    display.draw_iter(universe.render_frame().into_iter()).unwrap();
    window.update(&display);

    // window.update(universe.render_frame());
    'running: loop {
        // window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => {
                    match keycode {
                        Keycode::Left | Keycode::A => universe.move_left(),
                        Keycode::Right | Keycode::D => universe.move_right(),
                        Keycode::Up | Keycode::W => universe.move_up(),
                        Keycode::Down | Keycode::S => universe.move_down(),
                        Keycode::Return => universe.teleport(),
                        Keycode::Space => universe.place_dynamite(),
                        _ => {},
                    };
                }
                // SimulatorEvent::MouseButtonUp { point, .. } => {
                //     move_circle(&mut display, position, point)?;
                //     position = point;
                // }
                _ => {}
            }
        }
        display.draw_iter(universe.render_frame().into_iter()).unwrap();
        window.update(&display);
        std::thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}

