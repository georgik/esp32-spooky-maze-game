use embedded_graphics::{
    image::Image,
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor},
    text::Text,
    Drawable,
};

use crate::{assets::Assets, maze::Maze};
use heapless::String;
use tinybmp::Bmp;

pub enum GameState {
    Start,
    Playing,
    GameOver,
    Outro,
    Demo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Up,
    Down,
    Left,
    Right,
    Teleport,
    PlaceDynamite,
    Start,
    Stop,
}

pub struct Engine<D> {
    pub start_time: u64,
    pub ghost_x: i32,
    pub ghost_y: i32,
    display: D,
    assets: Option<Assets<'static>>,
    step_size_x: u32,
    step_size_y: u32,
    maze: Maze,
    camera_x: i32,
    camera_y: i32,
    animation_step: u32,
    teleport_counter: u32,
    walker_counter: u32,
    dynamite_counter: u32,
    game_state: GameState,
    outro_counter: u32,
}

impl<D: embedded_graphics::draw_target::DrawTarget<Color = Rgb565>> Engine<D> {
    pub fn new(display: D, seed: Option<[u8; 32]>) -> Engine<D> {
        Engine {
            start_time: 0,
            ghost_x: 9 * 16,
            ghost_y: 7 * 16,
            display,
            assets: None,
            step_size_x: 16,
            step_size_y: 16,
            maze: Maze::new(64, 64, seed),
            camera_x: 0,
            camera_y: 0,
            // #[cfg(any(feature = "imu_controls"))]
            animation_step: 0,
            teleport_counter: 100,
            walker_counter: 0,
            dynamite_counter: 1,
            game_state: GameState::Playing,
            outro_counter: 0,
        }
    }

    fn check_object_collisions(&mut self) {
        let x = self.camera_x + self.ghost_x;
        let y = self.camera_y + self.ghost_y;

        // Coin collisions
        match self.maze.get_coin_at(x, y) {
            Some(coin) => {
                self.maze.remove_coin(coin);
                if self.maze.coin_counter == 0 {
                    self.game_state = GameState::Outro;
                }
            }
            None => {}
        }

        // Walker collisions
        match self.maze.get_walker_at(x, y) {
            Some(walker) => {
                self.maze.relocate_walker(walker);
                if self.walker_counter < 10000 {
                    self.walker_counter += 100;
                }
            }
            None => {}
        }

        // Dynamite collisions
        match self.maze.get_dynamite_at(x, y) {
            Some(dynamite) => {
                self.maze.relocate_dynamite(dynamite);
                if self.dynamite_counter < 10000 {
                    self.dynamite_counter += 1;
                }
            }
            None => {}
        }
    }

    fn relocate_avatar(&mut self) {
        let (new_camera_x, new_camera_y) = self.maze.get_random_coordinates();
        (self.camera_x, self.camera_y) = (new_camera_x - self.ghost_x, new_camera_y - self.ghost_y);
    }

    fn relocate_coins(&mut self, amount: u32) {
        self.maze.relocate_coins(amount);
    }

    fn check_npc_collision(&mut self) {
        let x = self.camera_x + self.ghost_x;
        let y = self.camera_y + self.ghost_y;

        match self.maze.get_npc_at(x, y) {
            Some(_npc) => {
                self.relocate_coins(5);
                self.relocate_avatar();
            }
            None => {}
        }
    }

    fn is_walkable(&self, x: i32, y: i32) -> bool {
        // Walk through walls
        if self.walker_counter > 0 {
            !self.maze.check_boundary_collision(x, y)
        } else {
            !self.maze.check_wall_collision(x, y)
        }
    }

    pub fn action(&mut self, action: Action) {
        match self.game_state {
            GameState::Demo => match action {
                Action::None => {}
                Action::Up => self.move_up(),
                Action::Down => self.move_down(),
                Action::Left => self.move_left(),
                Action::Right => self.move_right(),
                Action::Teleport => self.teleport(),
                Action::PlaceDynamite => self.place_dynamite(),
                Action::Start => self.switch_game_state(GameState::Playing),
                _ => {}
            },
            GameState::Playing => match action {
                Action::None => {}
                Action::Up => self.move_up(),
                Action::Down => self.move_down(),
                Action::Left => self.move_left(),
                Action::Right => self.move_right(),
                Action::Teleport => self.teleport(),
                Action::PlaceDynamite => self.place_dynamite(),
                Action::Stop => self.switch_game_state(GameState::Demo),
                _ => {}
            },
            GameState::Outro => {
                if self.outro_counter > 30 {
                    self.game_state = GameState::Start;
                    self.outro_counter = 0;
                    self.start();
                }
            }
            _ => {}
        }
    }

    fn move_right(&mut self) {
        let new_camera_x = self.camera_x + self.step_size_x as i32;
        if self.is_walkable(new_camera_x + self.ghost_x, self.camera_y + self.ghost_y) {
            self.camera_x = new_camera_x;
            self.check_object_collisions();
        }
    }

    fn move_left(&mut self) {
        let new_camera_x = self.camera_x - self.step_size_x as i32;
        if self.is_walkable(new_camera_x + self.ghost_x, self.camera_y + self.ghost_y) {
            self.camera_x = new_camera_x;
            self.check_object_collisions();
        }
    }

    fn move_up(&mut self) {
        let new_camera_y = self.camera_y - self.step_size_y as i32;
        if self.is_walkable(self.camera_x + self.ghost_x, new_camera_y + self.ghost_y) {
            self.camera_y = new_camera_y;
            self.check_object_collisions();
        }
    }

    fn move_down(&mut self) {
        let new_camera_y = self.camera_y + self.step_size_y as i32;
        if self.is_walkable(self.camera_x + self.ghost_x, new_camera_y + self.ghost_y) {
            self.camera_y = new_camera_y;
            self.check_object_collisions();
        }
    }

    fn teleport(&mut self) {
        if self.teleport_counter == 100 {
            self.relocate_avatar();
            self.teleport_counter = 0;
        }
    }

    fn place_dynamite(&mut self) {
        if self.dynamite_counter == 0 {
            return;
        }

        self.maze
            .place_dynamite(self.camera_x + self.ghost_x, self.camera_y + self.ghost_y);
        self.dynamite_counter -= 1;
    }

    pub fn draw_maze(&mut self, camera_x: i32, camera_y: i32) {
        #[cfg(feature = "system_timer")]
        let start_timestamp = SystemTimer::now();

        let assets = self.assets.as_ref().unwrap();
        let ground = assets.ground.as_ref().unwrap();
        let wall = assets.wall.as_ref().unwrap();
        let scorched = assets.scorched.as_ref().unwrap();
        let empty = assets.empty.as_ref().unwrap();

        let camera_tile_x = camera_x / self.maze.tile_width as i32;
        let camera_tile_y = camera_y / self.maze.tile_height as i32;
        for x in camera_tile_x..(camera_tile_x + (self.maze.visible_width as i32) - 1) {
            for y in camera_tile_y..(camera_tile_y + (self.maze.visible_height as i32) - 1) {
                let position_x = (x as i32 * self.maze.tile_width as i32) - camera_x;
                let position_y = (y as i32 * self.maze.tile_height as i32) - camera_y;
                let position = Point::new(position_x, position_y);

                if x < 0
                    || y < 0
                    || x > (self.maze.width - 1) as i32
                    || y > (self.maze.height - 1) as i32
                {
                    let tile = Image::new(empty, position);
                    let _ = tile.draw(&mut self.display);
                } else {
                    let tile_index = self.maze.data[(x + y * (self.maze.width as i32)) as usize];
                    match tile_index {
                        0 => {
                            let tile = Image::new(ground, position);
                            let _ = tile.draw(&mut self.display);
                        }
                        1 => {
                            let tile = Image::new(wall, position);
                            let _ = tile.draw(&mut self.display);
                        }
                        _ => {
                            let tile = Image::new(scorched, position);
                            let _ = tile.draw(&mut self.display);
                        }
                    }
                }
            }
        }
    }

    pub fn tick(&mut self) {
        match self.game_state {
            GameState::Playing | GameState::Demo => {
                self.animation_step += 1;
                if self.animation_step > 1 {
                    self.animation_step = 0;
                }

                // Recharge teleport
                if self.teleport_counter < 100 {
                    self.teleport_counter += 1;
                }

                // Decrement remaining time when Walker is active
                if self.walker_counter > 0 {
                    self.walker_counter -= 1;
                }

                self.maze.move_npcs();
                self.check_npc_collision();
            }
            GameState::Outro => {
                if self.outro_counter < 1000 {
                    self.outro_counter += 1;
                }
            }
            _ => {}
        }
    }

    pub fn initialize(&mut self) {
        let mut assets = Assets::new();
        assets.load();
        self.assets = Some(assets);
    }

    pub fn start(&mut self) {
        self.maze.generate_maze(32, 32);
        self.relocate_avatar();
        self.maze.generate_coins();
        self.maze.generate_npcs();
        self.maze.generate_walkers();
        self.maze.generate_dynamites();
        self.draw_maze(self.camera_x, self.camera_y);
        self.game_state = GameState::Demo;
    }

    pub fn switch_game_state(&mut self, new_state: GameState) {
        self.game_state = new_state;
    }

    fn draw_status_number(&mut self, value: u32, x: i32, y: i32) {
        let value_message: String<5> = String::from(value);
        let _ = Text::new(
            &value_message,
            Point::new(x, y),
            MonoTextStyle::new(&FONT_8X13, Rgb565::WHITE),
        )
        .draw(&mut self.display);
    }

    pub fn draw_main_scene(&mut self) -> &mut D {
        self.draw_maze(self.camera_x, self.camera_y);

        match self.assets {
            Some(ref mut assets) => {
                let coin_bmp: Bmp<Rgb565> = assets.coin.unwrap();
                for index in 0..100 {
                    let coin = self.maze.coins[index];
                    if coin.x < 0 || coin.y < 0 {
                        continue;
                    }

                    let draw_x = coin.x - self.camera_x;
                    let draw_y = coin.y - self.camera_y;
                    if draw_x >= 0
                        && draw_y >= 0
                        && draw_x < (self.maze.visible_width * 16).try_into().unwrap()
                        && draw_y < (self.maze.visible_height * 16).try_into().unwrap()
                    {
                        let position = Point::new(draw_x, draw_y);
                        let tile = Image::new(&coin_bmp, position);
                        let _ = tile.draw(&mut self.display);
                    }
                }

                let npc_bmp: Bmp<Rgb565> = assets.npc.unwrap();
                for index in 0..5 {
                    let item = self.maze.npcs[index];
                    if item.x < 0 || item.y < 0 {
                        continue;
                    }

                    let draw_x = item.x - self.camera_x;
                    let draw_y = item.y - self.camera_y;
                    if draw_x >= 0
                        && draw_y >= 0
                        && draw_x < (self.maze.visible_width * 16).try_into().unwrap()
                        && draw_y < (self.maze.visible_height * 16).try_into().unwrap()
                    {
                        let position = Point::new(draw_x, draw_y);
                        let tile = Image::new(&npc_bmp, position);
                        let _ = tile.draw(&mut self.display);
                    }
                }

                let walker_bmp: Bmp<Rgb565> = assets.walker.unwrap();
                for index in 0..5 {
                    let item = self.maze.walkers[index];
                    if item.x < 0 || item.y < 0 {
                        continue;
                    }

                    let draw_x = item.x - self.camera_x;
                    let draw_y = item.y - self.camera_y;
                    if draw_x >= 0
                        && draw_y >= 0
                        && draw_x < (self.maze.visible_width * 16).try_into().unwrap()
                        && draw_y < (self.maze.visible_height * 16).try_into().unwrap()
                    {
                        let position = Point::new(draw_x, draw_y);
                        let tile = Image::new(&walker_bmp, position);
                        let _ = tile.draw(&mut self.display);
                    }
                }

                let dynamite_bmp: Bmp<Rgb565> = assets.dynamite.unwrap();
                for index in 0..1 {
                    let item = self.maze.dynamites[index];
                    if item.x < 0 || item.y < 0 {
                        continue;
                    }

                    let draw_x = item.x - self.camera_x;
                    let draw_y = item.y - self.camera_y;
                    if draw_x >= 0
                        && draw_y >= 0
                        && draw_x < (self.maze.visible_width * 16).try_into().unwrap()
                        && draw_y < (self.maze.visible_height * 16).try_into().unwrap()
                    {
                        let position = Point::new(draw_x, draw_y);
                        let tile = Image::new(&dynamite_bmp, position);
                        let _ = tile.draw(&mut self.display);
                    }
                }

                match self.animation_step {
                    0 => {
                        let bmp: Bmp<Rgb565> = assets.ghost1.unwrap();
                        let ghost1 = Image::new(
                            &bmp,
                            Point::new(
                                self.ghost_x.try_into().unwrap(),
                                self.ghost_y.try_into().unwrap(),
                            ),
                        );
                        let _ = ghost1.draw(&mut self.display);
                    }
                    _ => {
                        let bmp: Bmp<Rgb565> = assets.ghost2.unwrap();
                        let ghost2 = Image::new(
                            &bmp,
                            Point::new(
                                self.ghost_x.try_into().unwrap(),
                                self.ghost_y.try_into().unwrap(),
                            ),
                        );
                        let _ = ghost2.draw(&mut self.display);
                    }
                }

                // Status bar - coins, teleport, walk time, dynamite
                let position = Point::new(5, 6);
                let tile = Image::new(&coin_bmp, position);
                let _ = tile.draw(&mut self.display);

                let teleport_bmp: Bmp<Rgb565> = assets.teleport.unwrap();
                let position = Point::new(5, 28);
                let tile = Image::new(&teleport_bmp, position);
                let _ = tile.draw(&mut self.display);

                let walker_bmp: Bmp<Rgb565> = assets.walker.unwrap();
                let position = Point::new(5, 50);
                let tile = Image::new(&walker_bmp, position);
                let _ = tile.draw(&mut self.display);

                let dynamite_bmp: Bmp<Rgb565> = assets.dynamite.unwrap();
                let position = Point::new(5, 72);
                let tile = Image::new(&dynamite_bmp, position);
                let _ = tile.draw(&mut self.display);

                // display.flush().unwrap();
            }
            None => {
                panic!("Assets not loaded");
            }
        };

        self.draw_status_number(self.maze.coin_counter, 24, 17);
        self.draw_status_number(self.teleport_counter, 24, 39);
        self.draw_status_number(self.walker_counter, 24, 61);
        self.draw_status_number(self.dynamite_counter, 24, 83);

        &mut self.display
    }

    pub fn draw_demo_text(&mut self) -> &mut D {
        let _ = Text::new(
            "Demo. Press any key...",
            Point::new(80, 150),
            MonoTextStyle::new(&FONT_8X13, Rgb565::WHITE),
        )
        .draw(&mut self.display);
        &mut self.display
    }

    pub fn draw_outro_scene(&mut self) -> &mut D {
        let assets = self.assets.as_ref().unwrap();
        let bmp: Bmp<Rgb565> = assets.smiley.unwrap();
        let (x, y) = (
            self.maze.get_rand() + self.maze.get_rand() % 70,
            self.maze.get_rand() % 240,
        );
        let outro = Image::new(&bmp, Point::new(x, y));
        let _ = outro.draw(&mut self.display);
        &mut self.display
    }

    pub fn draw(&mut self) -> &mut D {
        match self.game_state {
            GameState::Demo => {
                self.draw_main_scene();
                self.draw_demo_text()
            }
            GameState::Playing => self.draw_main_scene(),
            GameState::Outro => self.draw_outro_scene(),
            _ => &mut self.display,
        }
    }
}
