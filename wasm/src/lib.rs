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
    pub ground: Option<Bmp<'a, Rgb565>>,
    pub wall: Option<Bmp<'a, Rgb565>>,
    pub empty: Option<Bmp<'a, Rgb565>>,
    pub ghost1: Option<Bmp<'a, Rgb565>>,
    pub ghost2: Option<Bmp<'a, Rgb565>>,
}

impl Assets<'static> {
    pub fn new() -> Assets<'static> {
        Assets {
            ground: None,
            wall: None,
            empty: None,
            ghost1: None,
            ghost2: None,
        }
    }
}

pub struct Maze {
    pub width: u32,
    pub height: u32,
    pub visible_width: u32,
    pub visible_height: u32,
    pub data: [u8; 64*64],
    // Tile map should have small border top line and left column
    pub offset: u32,
    pub tile_width: u32,
    pub tile_height: u32,
}

impl Maze {
    pub fn new(width: u32, height: u32) -> Maze {
        Maze {
            width,
            height,
            visible_width: 12,
            visible_height: 10,
            data: [1; 64*64],
            offset: width+1,
            tile_width: 16,
            tile_height: 16,
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
    maze: Maze,
    camera_x: i32,
    camera_y: i32,
    old_camera_x: i32,
    old_camera_y: i32,
}



#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        Universe {
            start_time: 0,
            ghost_x: 5*16,
            ghost_y: 5*16,
            old_ghost_x: 5*16,
            old_ghost_y: 5*16,
            display: None,
            assets: None,
            step_size_x: 16,
            step_size_y: 16,
            maze: Maze::new(64, 64),
            camera_x: 0,
            camera_y: 0,
            old_camera_x: 0,
            old_camera_y: 0,
        }
    }

    pub fn tick(&mut self) {
        self.ghost_x += 1;
        self.ghost_y += 1;
    }

    pub fn move_right(&mut self) {
        self.camera_x += self.step_size_x as i32;
        console::log_1(&"key_right".into());
    }

    pub fn move_left(&mut self) {
        self.camera_x -= self.step_size_x as i32;
        console::log_1(&"key_left".into());
    }

    pub fn move_up(&mut self) {
        self.camera_y -= self.step_size_y as i32;
        console::log_1(&"key_up".into());
    }

    pub fn move_down(&mut self) {
        self.camera_y += self.step_size_y as i32;
        console::log_1(&"key_down".into());
    }

    pub fn ghost_x(&self) -> u32 {
        self.ghost_x
    }

    pub fn ghost_y(&self) -> u32 {
        self.ghost_y
    }

    pub fn generate_maze(&mut self) {
        println!("Rendering maze");

        // Dimensions of maze graph produced by algorithm
        // #[cfg(any(feature = "esp32s3_box"))]
        const MAZE_GRAPH_WIDTH:usize = 10;
        // #[cfg(not(feature = "esp32s3_box"))]
        // const MAZE_GRAPH_WIDTH:usize = 8;
        const MAZE_GRAPH_HEIGHT:usize = 8;

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
                let tile_index = (x-1)*2+(y-1)*2*(self.maze.width as usize)+(self.maze.offset as usize);

                self.maze.data[tile_index] = 0;

                if field.has_passage(&Direction::West) {
                    self.maze.data[tile_index + 1] = 0;
                }

                if field.has_passage(&Direction::South) {
                    self.maze.data[tile_index + (self.maze.width as usize)] = 0;
                }
            }
        }

    }

    pub fn draw_maze(&mut self, camera_x: i32, camera_y: i32) {
        println!("Rendering the maze to display");
        #[cfg(feature = "system_timer")]
        let start_timestamp = SystemTimer::now();


        match self.display {
            Some(ref mut display) => {
                let assets = self.assets.as_ref().unwrap();
                let ground = assets.ground.as_ref().unwrap();
                let wall = assets.wall.as_ref().unwrap();
                let empty = assets.empty.as_ref().unwrap();

                let camera_tile_x = camera_x / self.maze.tile_width as i32;
                let camera_tile_y = camera_y / self.maze.tile_height as i32;
                for x in camera_tile_x..(camera_tile_x + (self.maze.visible_width as i32)-1) {
                    for y in camera_tile_y..(camera_tile_y + (self.maze.visible_height as i32)-1) {
                        let position_x = (x as i32 * self.maze.tile_width as i32) - camera_x;
                        let position_y = (y as i32 * self.maze.tile_height as i32) - camera_y;
                        let position = Point::new(position_x.try_into().unwrap(), position_y.try_into().unwrap());
                        if position_x < 0 || position_y < 0 {
                            let tile = Image::new(empty, position);
                            tile.draw(display).unwrap();
                        } else if self.maze.data[(x+y*(self.maze.width as i32)) as usize] == 0 {
                            let tile = Image::new(ground, position);
                            tile.draw(display).unwrap();
                        } else {
                            let tile = Image::new(wall, position);
                            tile.draw(display).unwrap();
                        }
                    }
                }
            },
            None => {}
        }


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

        let empty_data = include_bytes!("../../assets/img/empty.bmp");
        let empty_bmp = Bmp::<Rgb565>::from_slice(empty_data).unwrap();

        let ghost1_data = include_bytes!("../../assets/img/ghost1.bmp");
        let ghost1_bmp = Bmp::<Rgb565>::from_slice(ghost1_data).unwrap();

        let ghost2_data = include_bytes!("../../assets/img/ghost2.bmp");
        let ghost2_bmp = Bmp::<Rgb565>::from_slice(ghost2_data).unwrap();

        assets.ground = Some(ground_bmp);
        assets.wall = Some(wall_bmp);
        assets.empty = Some(empty_bmp);
        assets.ghost1 = Some(ghost1_bmp);
        assets.ghost2 = Some(ghost2_bmp);

        self.assets = Some(assets);

        self.generate_maze();
        self.draw_maze(self.camera_x,self.camera_y);

        let mut old_x = self.ghost_x;
        let mut old_y = self.ghost_y;

    }

    pub fn render_frame(&mut self) {

        console::log_1(&"tick".into());

        if self.old_camera_x != self.camera_x || self.old_camera_y != self.camera_y {
            self.draw_maze(self.camera_x,self.camera_y);
            self.old_camera_x = self.camera_x;
            self.old_camera_y = self.camera_y;
        }


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