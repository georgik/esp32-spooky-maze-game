// #![no_std]
// #![no_main]

use embedded_graphics::{
    pixelcolor::Rgb565,
};
use embedded_graphics_web_simulator::{
    display::{WebSimulatorDisplay}, output_settings::OutputSettingsBuilder,
};

use wasm_bindgen::prelude::*;
use web_sys::{console};

use spooky_core::{ engine::Engine };

#[wasm_bindgen]
pub struct Universe {
    engine: Engine<WebSimulatorDisplay<Rgb565>>,
}

fn get_seed_buffer() -> Option<[u8; 32]> {
    let mut seed_buffer = [0u8; 32];
    getrandom::getrandom(&mut seed_buffer).unwrap();
    Some(seed_buffer)
}


#[wasm_bindgen]
impl Universe {

    pub fn new() -> Universe {
        Universe {
            engine: {
                let document = web_sys::window().unwrap().document().unwrap();
                let output_settings = OutputSettingsBuilder::new()
                    .scale(2)
                    // .pixel_spacing(1)
                    .build();
                let display = WebSimulatorDisplay::new(
                        (320, 240),
                        &output_settings,
                        document.get_element_by_id("graphics").as_ref(),
                );
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

    pub fn teleport(&mut self) {
        self.engine.teleport();
    }

    pub fn place_dynamite(&mut self) {
        self.engine.place_dynamite();
    }

    pub fn initialize(&mut self) {
        self.engine.initialize();
    }

    pub fn render_frame(&mut self) {

        console::log_1(&"tick".into());
        self.engine.tick();
        let display:&mut WebSimulatorDisplay<Rgb565> = self.engine.draw();
        display.flush().unwrap();

    }
}