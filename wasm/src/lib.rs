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

mod assets;
use crate::assets::Assets;

mod maze;
use crate::maze::Maze;

use tinybmp::Bmp;


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
                        let position = Point::new(position_x, position_y);

                        if x < 0 || y < 0 || x > (self.maze.width-1) as i32 || y > (self.maze.height-1) as i32 {
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
        assets.load();
        self.assets = Some(assets);

        self.maze.generate_maze(32, 32);
        self.maze.generate_coins();
        self.maze.generate_npcs();
        self.draw_maze(self.camera_x,self.camera_y);

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