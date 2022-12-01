// #![no_std]
// #![no_main]

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    image::Image,
};
use embedded_graphics_web_simulator::{
    display::{WebSimulatorDisplay}, output_settings::OutputSettingsBuilder,
};

use wasm_bindgen::prelude::*;
use web_sys::{console};

use embedded_graphics::{
    prelude::RgbColor,
    mono_font::{
        ascii::{FONT_8X13},
        MonoTextStyle,
    },
    prelude::Point,
    text::{Text},
    Drawable,
};

use spooky_core::{ assets::Assets, maze::Maze, engine::Engine, spritebuf::SpriteBuf };
use embedded_graphics_framebuf::{FrameBuf, backends::FrameBufferBackend};

use tinybmp::Bmp;
use heapless::String;

#[wasm_bindgen]
pub struct Universe {
    engine: Option<Engine<SpriteBuf<FrameBufferBackend<Color = Rgb565>>>>,
    display: Option<WebSimulatorDisplay<Rgb565>>,
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
            engine: Engine,
            display: None,
        }
    }


    pub fn initialize(&mut self) {
        
        let document = web_sys::window().unwrap().document().unwrap();
        let output_settings = OutputSettingsBuilder::new()
            .scale(1)
            .pixel_spacing(1)
            .build();
        let mut display = WebSimulatorDisplay::new(
                (320, 240),
                &output_settings,
                document.get_element_by_id("graphics").as_ref(),
        );

        display.clear(Rgb565::BLACK).unwrap();
        display.flush().unwrap();
        self.display = Some(display);
        let mut data = [Rgb565::BLACK ; 320*240];
        let fbuf = FrameBuf::new(&mut data, 320, 240);
        let spritebuf = SpriteBuf::new(fbuf);
        let mut engine = Engine::new(spritebuf, get_seed_buffer());
        engine.initialize();
        self.engine = Some(engine);

    }

    pub fn render_frame(&mut self) {

        console::log_1(&"tick".into());

        match &mut self.engine {
            Some(engine) => {
                engine.tick();
                let buf = engine.draw();
                match self.display {
                    Some(ref mut display) => {
                        display.draw_iter(buf.into_iter()).unwrap();
                    },
                    None => {},
                }
                // display.flush().unwrap();
            },
            None => {
                console::log_1(&"no engine".into());
            }
        }

    }
}