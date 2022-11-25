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

use spooky_core::assets::Assets;
use spooky_core::maze::Maze;

use tinybmp::Bmp;
use heapless::String;

#[wasm_bindgen]
pub struct Universe {
    pub start_time: u64,
    pub ghost_x: i32,
    pub ghost_y: i32,
    display: Option<WebSimulatorDisplay<Rgb565>>,
    assets: Option<Assets<'static>>,
    step_size_x: u32,
    step_size_y: u32,
    maze: Maze,
    camera_x: i32,
    camera_y: i32,
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
            start_time: 0,
            ghost_x: 9*16,
            ghost_y: 7*16,
            display: None,
            assets: None,
            step_size_x: 16,
            step_size_y: 16,
            maze: Maze::new(64, 64, get_seed_buffer()),
            camera_x: 0,
            camera_y: 0,
        }
    }

    fn check_coin_collision(&mut self) {
        let x = self.camera_x + self.ghost_x;
        let y = self.camera_y + self.ghost_y;

        match self.maze.get_coin_at(x, y) {
            Some(coin) => {
                self.maze.remove_coin(coin);
            },
            None => {}
        }
    }

    fn relocate_avatar(&mut self) {
        let (new_camera_x, new_camera_y) = self.maze.get_random_coordinates();
        (self.camera_x, self.camera_y) = (new_camera_x - self.ghost_x, new_camera_y - self.ghost_y);
    }

    fn check_npc_collision(&mut self) {
        let x = self.camera_x + self.ghost_x;
        let y = self.camera_y + self.ghost_y;

        match self.maze.get_npc_at(x, y) {
            Some(_npc) => {
                self.relocate_avatar();
            },
            None => {}
        }
    }

    pub fn move_right(&mut self) {
        let new_camera_x = self.camera_x + self.step_size_x as i32;
        if !self.maze.check_wall_collision(new_camera_x + self.ghost_x, self.camera_y + self.ghost_y) {
            self.camera_x = new_camera_x;
            self.check_coin_collision();
        }
        console::log_1(&"key_right".into());
    }

    pub fn move_left(&mut self) {
        let new_camera_x = self.camera_x - self.step_size_x as i32;
        if !self.maze.check_wall_collision(new_camera_x + self.ghost_x, self.camera_y + self.ghost_y) {
            self.camera_x = new_camera_x;
            self.check_coin_collision();
        }
        console::log_1(&"key_left".into());
    }

    pub fn move_up(&mut self) {
        let new_camera_y = self.camera_y - self.step_size_y as i32;
        if !self.maze.check_wall_collision(self.camera_x + self.ghost_x, new_camera_y + self.ghost_y) {
            self.camera_y = new_camera_y;
            self.check_coin_collision();
        }
        console::log_1(&"key_up".into());
    }

    pub fn move_down(&mut self) {
        let new_camera_y = self.camera_y + self.step_size_y as i32;
        if !self.maze.check_wall_collision(self.camera_x + self.ghost_x, new_camera_y + self.ghost_y) {
            self.camera_y = new_camera_y;
            self.check_coin_collision();
        }
        console::log_1(&"key_down".into());
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
        self.relocate_avatar();
        self.maze.generate_coins();
        self.maze.generate_npcs();
        self.draw_maze(self.camera_x,self.camera_y);

    }

    pub fn render_frame(&mut self) {

        console::log_1(&"tick".into());

        self.maze.move_npcs();
        self.check_npc_collision();
        self.draw_maze(self.camera_x,self.camera_y);

        match self.display {
            Some(ref mut display) => {
                match self.assets {
                    Some(ref mut assets) => {

                        let coin_bmp:Bmp<Rgb565> = assets.coin.unwrap();
                        for index in 0..100 {
                            let coin = self.maze.coins[index];
                            if coin.x < 0 || coin.y < 0 {
                                continue;
                            }

                            let draw_x = coin.x - self.camera_x;
                            let draw_y = coin.y - self.camera_y;
                            if draw_x >= 0 && draw_y >= 0 && draw_x < (self.maze.visible_width*16).try_into().unwrap() && draw_y < (self.maze.visible_height*16).try_into().unwrap() {
                                let position = Point::new(draw_x, draw_y);
                                let tile = Image::new(&coin_bmp, position);
                                tile.draw(display).unwrap();
                            }
                        }

                        let npc_bmp:Bmp<Rgb565> = assets.npc.unwrap();
                        for index in 0..5 {
                            let item = self.maze.npcs[index];
                            if item.x < 0 || item.y < 0 {
                                continue;
                            }

                            let draw_x = item.x - self.camera_x;
                            let draw_y = item.y - self.camera_y;
                            if draw_x >= 0 && draw_y >= 0 && draw_x < (self.maze.visible_width*16).try_into().unwrap() && draw_y < (self.maze.visible_height*16).try_into().unwrap() {
                                let position = Point::new(draw_x, draw_y);
                                let tile = Image::new(&npc_bmp, position);
                                tile.draw(display).unwrap();
                            }
                        }

                        let bmp:Bmp<Rgb565> = assets.ghost1.unwrap();
                        let ghost1 = Image::new(&bmp, Point::new(self.ghost_x.try_into().unwrap(), self.ghost_y.try_into().unwrap()));
                        ghost1.draw(display).unwrap();
                        display.flush().unwrap();
                    },
                    None => {}
                }

                let coin_message: String<5> = String::from(self.maze.coin_counter);
                Text::new(&coin_message, Point::new(10, 10), MonoTextStyle::new(&FONT_8X13, Rgb565::WHITE))
                    .draw(display)
                    .unwrap();

                display.flush().unwrap();
            },
            None => {}
        }

    }
}