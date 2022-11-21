// #![no_std]
// #![no_main]

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Sector},
    image::Image,
};
use embedded_graphics_web_simulator::{
    display::{WebSimulatorDisplay, self}, output_settings::OutputSettingsBuilder,
};

use wasm_bindgen::prelude::*;
use web_sys::{console, Performance};

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;

use embedded_graphics::{
    prelude::RgbColor,
    mono_font::{
        ascii::{FONT_8X13, FONT_9X18_BOLD},
        MonoTextStyle,
    },
    prelude::Point,
    text::{Alignment, Text},
    Drawable,
};
use gloo_timers::callback::Timeout;

use tinybmp::Bmp;

use maze_generator::prelude::*;
use maze_generator::recursive_backtracking::{RbGenerator};

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn perf_to_system(amt: f64) -> SystemTime {
    let secs = (amt as u64) / 1_000;
    let nanos = (((amt as u64) % 1_000) as u32) * 1_000_000;
    UNIX_EPOCH + Duration::new(secs, nanos)
}

struct Assets<'a> {
    ground: Option<Bmp<'a, Rgb565>>,
    wall: Option<Bmp<'a, Rgb565>>,
    ghost1: Option<Bmp<'a, Rgb565>>,
    ghost2: Option<Bmp<'a, Rgb565>>,
}

impl Assets<'static> {
    pub fn new() -> Assets<'static> {
        Assets {
            ground: None,
            wall: None,
            ghost1: None,
            ghost2: None,
        }
    }
}

#[wasm_bindgen]
pub struct Universe {
    pub start_time: u64,
    pub ghost_x: u32,
    pub ghost_y: u32,
    old_ghost_x: u32,
    old_ghost_y: u32,
    display: Option<WebSimulatorDisplay<Rgb565>>,
    assets: Option<Assets<'static>>,
    step_size_x: u32,
    step_size_y: u32,
}



#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        Universe {
            start_time: 0,
            ghost_x: 16,
            ghost_y: 16,
            old_ghost_x: 16,
            old_ghost_y: 16,
            display: None,
            assets: None,
            step_size_x: 16,
            step_size_y: 16,
        }
    }

    pub fn tick(&mut self) {
        self.ghost_x += 1;
        self.ghost_y += 1;
    }

    pub fn move_right(&mut self) {
        self.ghost_x += self.step_size_x;
        console::log_1(&"key_right".into());
    }

    pub fn move_left(&mut self) {
        if self.ghost_x > 0 {
            self.ghost_x -= self.step_size_x;
        }
        console::log_1(&"key_left".into());
    }

    pub fn move_up(&mut self) {
        if self.ghost_y > 0 {
            self.ghost_y -= self.step_size_y;
        }
        console::log_1(&"key_up".into());
    }

    pub fn move_down(&mut self) {
        self.ghost_y += self.step_size_y;
        console::log_1(&"key_down".into());
    }

    pub fn ghost_x(&self) -> u32 {
        self.ghost_x
    }

    pub fn ghost_y(&self) -> u32 {
        self.ghost_y
    }


    pub fn initialize(&mut self) {
        let document = web_sys::window().unwrap().document().unwrap();
        let output_settings = OutputSettingsBuilder::new()
            .scale(1)
            .pixel_spacing(1)
            .build();
        self.display = Some(WebSimulatorDisplay::new(
                (320, 240),
                &output_settings,
                document.get_element_by_id("graphics").as_ref(),
        ));

        match self.display {
            Some(ref mut display) => {
                display.clear(Rgb565::BLACK).unwrap();
                display.flush().unwrap();
            },
            None => {}
        }

        println!("Loading image");

        let mut assets = Assets::new();
        let ground_data = include_bytes!("../../assets/img/ground.bmp");
        let ground_bmp = Bmp::<Rgb565>::from_slice(ground_data).unwrap();

        let wall_data = include_bytes!("../../assets/img/wall.bmp");
        let wall_bmp = Bmp::<Rgb565>::from_slice(wall_data).unwrap();

        let ghost1_data = include_bytes!("../../assets/img/ghost1.bmp");
        let ghost1_bmp = Bmp::<Rgb565>::from_slice(ghost1_data).unwrap();

        let ghost2_data = include_bytes!("../../assets/img/ghost2.bmp");
        let ghost2_bmp = Bmp::<Rgb565>::from_slice(ghost2_data).unwrap();

        assets.ground = Some(ground_bmp);
        assets.wall = Some(wall_bmp);
        assets.ghost1 = Some(ghost1_bmp);
        assets.ghost2 = Some(ghost2_bmp);

        self.assets = Some(assets);

        println!("Rendering maze");

        // Dimension of tiles
        const TILE_WIDTH:usize = 16;
        const TILE_HEIGHT:usize = 16;

        // Simplified maze map in memory for tile mapping
        // #[cfg(any(feature = "esp32s3_box"))]
        const MAZE_WIDTH:usize = 21;
        // #[cfg(not(feature = "esp32s3_box"))]
        // const MAZE_WIDTH:usize = 16;
        const MAZE_HEIGHT:usize = 16;

        // Tile map should have small border top line and left column
        const MAZE_OFFSET:usize = MAZE_WIDTH + 1;

        // Dimension of Playground
        const PLAYGROUND_WIDTH:usize = MAZE_WIDTH * TILE_WIDTH;
        const PLAYGROUND_HEIGHT:usize = MAZE_HEIGHT * MAZE_HEIGHT;

        // Dimensions of maze graph produced by algorithm
        // #[cfg(any(feature = "esp32s3_box"))]
        const MAZE_GRAPH_WIDTH:usize = 10;
        // #[cfg(not(feature = "esp32s3_box"))]
        // const MAZE_GRAPH_WIDTH:usize = 8;
        const MAZE_GRAPH_HEIGHT:usize = 8;

        let mut maze: [u8; MAZE_WIDTH*MAZE_HEIGHT] = [1; MAZE_WIDTH*MAZE_HEIGHT];

        println!("Initializing Random Number Generator Seed");
        // let mut rng = Rng::new(peripherals.RNG);
        // let mut rng = Rng::new( 0x12345678 );
        let mut seed_buffer = [0u8;32];
        // rng.read(&mut seed_buffer).unwrap();

        println!("Acquiring maze generator");
        let mut generator = RbGenerator::new(Some(seed_buffer));
        println!("Generating maze");
        let maze_graph = generator.generate(MAZE_GRAPH_WIDTH as i32, MAZE_GRAPH_HEIGHT as i32).unwrap();

        println!("Converting to tile maze");
        for y in 1usize..MAZE_GRAPH_HEIGHT {
            for x in 1usize..MAZE_GRAPH_WIDTH {
                let field = maze_graph.get_field(&(x.try_into().unwrap(),y.try_into().unwrap()).into()).unwrap();
                let tile_index = (x-1)*2+(y-1)*2*MAZE_WIDTH+MAZE_OFFSET;

                maze[tile_index] = 0;

                if field.has_passage(&Direction::West) {
                    maze[tile_index + 1] = 0;
                }

                if field.has_passage(&Direction::South) {
                    maze[tile_index + MAZE_WIDTH] = 0;
                }
            }
        }

        println!("Rendering the maze to display");
        #[cfg(feature = "system_timer")]
        let start_timestamp = SystemTimer::now();


        match self.display {
            Some(ref mut display) => {
                for x in 0..(MAZE_WIDTH-1) {
                    for y in 0..(MAZE_HEIGHT-1) {
                        let position = Point::new((x*TILE_WIDTH).try_into().unwrap(), (y*TILE_HEIGHT).try_into().unwrap());
                        if maze[x+y*MAZE_WIDTH] == 0 {
                            let tile = Image::new(&ground_bmp, position);
                            tile.draw(display).unwrap();
                        } else {
                            let tile = Image::new(&wall_bmp, position);
                            tile.draw(display).unwrap();
                        }
                    }
                }
            },
            None => {}
        }

        let mut old_x = self.ghost_x;
        let mut old_y = self.ghost_y;

    }

    pub fn render_frame(&mut self) {

        console::log_1(&"tick".into());
        match self.display {
            Some(ref mut display) => {
                match self.assets {
                    Some(ref mut assets) => {
                        let bmp:Bmp<Rgb565> = assets.ghost1.unwrap();
                        let ghost1 = Image::new(&bmp, Point::new(self.ghost_x.try_into().unwrap(), self.ghost_y.try_into().unwrap()));
                        ghost1.draw(display).unwrap();
                        display.flush().unwrap();
                    },
                    None => {}
                }

                display.flush().unwrap();
            },
            None => {}
        }

    }
}