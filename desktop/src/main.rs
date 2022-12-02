
// #![no_std]
// #![no_main]

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
};
use embedded_graphics_simulator::{
    sdl2::Keycode, OutputSettings, SimulatorDisplay, SimulatorEvent, Window,
};


use spooky_core::{ engine::Engine };

pub struct Universe {
    engine: Engine<SimulatorDisplay<Rgb565>>,
}

fn get_seed_buffer() -> Option<[u8; 32]> {
    let mut seed_buffer = [0u8; 32];
    getrandom::getrandom(&mut seed_buffer).unwrap();
    Some(seed_buffer)
}

impl Universe {

    pub fn new() -> Universe {
        Universe {
            engine: {
                let display = SimulatorDisplay::new(Size::new(320, 200));
                // let mut data = [Rgb565::BLACK ; 320*240];
                // let fbuf = FrameBuf::new(&mut data, 320, 240);
                // let spritebuf = SpriteBuf::new(fbuf);
                Engine::new(display, get_seed_buffer())
            }
        }
    }

    pub fn move_up(&mut self) {
        self.engine.move_up();
    }

    pub fn move_down(&mut self) {
        self.engine.move_down();
    }

    pub fn move_left(&mut self) {
        self.engine.move_left();
    }

    pub fn move_right(&mut self) {
        self.engine.move_right();
    }

    pub fn initialize(&mut self) {
        self.engine.initialize();
    }

    pub fn render_frame(&mut self) -> &mut SimulatorDisplay<Rgb565> {
        self.engine.tick();
        self.engine.draw()
        // display.flush().unwrap();

    }
}



fn main() -> Result<(), core::convert::Infallible> {
    // let mut display: SimulatorDisplay<Rgb888> = SimulatorDisplay::new(Size::new(800, 480));
    let mut window = Window::new("Click to move circle", &OutputSettings::default());

    let mut universe = Universe::new();
    universe.initialize();


    window.update(universe.render_frame());
    'running: loop {
        // window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => {
                    match keycode {
                        Keycode::Left => universe.move_left(),
                        Keycode::Right => universe.move_right(),
                        Keycode::Up => universe.move_up(),
                        Keycode::Down => universe.move_down(),
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
        window.update(universe.render_frame());
    }

    Ok(())
}

