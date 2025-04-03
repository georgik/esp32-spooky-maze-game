// spooky_core/src/maze.rs

// If you want dynamic maze generation, enable the "dynamic_maze" feature
// and ensure the dependency on `maze_generator` is added to Cargo.toml.
#[cfg(feature = "dynamic_maze")]
use maze_generator::{prelude::*, recursive_backtracking::RbGenerator};

use bevy::prelude::Vec;
use rand::prelude::*;
use rand_chacha::ChaChaRng;

#[derive(Copy, Clone)]
pub struct Coin {
    pub x: i32,
    pub y: i32,
}

impl Coin {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Copy, Clone)]
pub struct Npc {
    pub x: i32,
    pub y: i32,
    pub vector_x: i32,
    pub vector_y: i32,
    pub steps_remaining: i32,
}

#[derive(Clone)]
pub struct Maze {
    pub width: u32,
    pub height: u32,
    pub visible_width: u32,
    pub visible_height: u32,
    pub data: [u8; 64 * 64],
    pub coins: [Coin; 100],
    pub coin_counter: u32,
    pub npcs: [Npc; 5],
    pub walkers: [Coin; 5],
    pub dynamites: [Coin; 1],
    pub offset: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    rng: ChaChaRng,
}

impl Maze {
    pub const MARGIN: i32 = 0;

    /// Create a new maze with the given dimensions and an optional seed.
    pub fn new(width: u32, height: u32, seed: Option<[u8; 32]>) -> Self {
        Self {
            width,
            height,
            visible_width: 21,
            visible_height: 16,
            #[cfg(feature = "dynamic_maze")]
            data: [1; 64 * 64],
            #[cfg(feature = "static_maze")]
            data: crate::static_maze_data::STATIC_MAZE_DATA,
            offset: width + 1,
            tile_width: 16,
            tile_height: 16,
            coins: [Coin { x: -1, y: -1 }; 100],
            coin_counter: 100,
            npcs: [Npc {
                x: -1,
                y: -1,
                vector_x: 0,
                vector_y: 0,
                steps_remaining: 0,
            }; 5],
            walkers: [Coin { x: -1, y: -1 }; 5],
            dynamites: [Coin { x: -1, y: -1 }; 1],
            rng: match seed {
                None => ChaChaRng::from_seed([42; 32]),
                Some(s) => ChaChaRng::from_seed(s),
            },
        }
    }

    /// Return a random number in the range 0..255.
    pub fn get_rand(&mut self) -> i32 {
        self.rng.gen_range(0..255)
    }

    /// Check if a given pixel coordinate is outside the maze boundaries.
    pub fn check_boundary_collision(&self, x: i32, y: i32) -> bool {
        x < 0
            || y < 0
            || x >= (self.width * self.tile_width) as i32
            || y >= (self.height * self.tile_height) as i32
    }

    /// Check if the pixel coordinate (x, y) collides with a wall.
    /// Coordinates are assumed to be in world space.
    pub fn check_wall_collision(&self, x: i32, y: i32) -> bool {
        let (left, bottom, right, top) = self.playable_bounds();
        // If outside playable bounds, treat as collision.
        if x < left || x >= right || y < bottom || y >= top {
            return true;
        }
        // Compute the tile coordinates relative to the playable area.
        let tile_x = (x - left) / self.tile_width as i32;
        // TODO: Replace 63 with proper value
        let tile_y = 63 - (y - bottom) / self.tile_height as i32;
        // Our maze data array is laid out with row 0 at the top.
        let maze_row = (self.height as i32 - 1) - tile_y;
        if tile_x < 0
            || maze_row < 0
            || tile_x >= self.width as i32
            || maze_row >= self.height as i32
        {
            return true;
        }
        let tile_index = (maze_row * self.width as i32 + tile_x) as usize;
        self.data[tile_index] == 1
    }

    /// Return a random valid coordinate (in pixel space) where the tile is walkable.
    /// Instead of building an entire list of valid coordinates, we repeatedly generate
    /// random tile indices (which are in the range 0..width and 0..height) and check if
    /// that tile is walkable. We try at most 10 times before returning a default coordinate.
    pub fn get_random_coordinates(&mut self) -> (i32, i32) {
        const MAX_ATTEMPTS: usize = 10;
        for _ in 0..MAX_ATTEMPTS {
            let tx = self.rng.gen_range(0..self.width as i32);
            let ty = self.rng.gen_range(0..self.height as i32);
            // Compute pixel coordinate for the tile (using the top-left corner).
            let x = tx * self.tile_width as i32;
            let y = ty * self.tile_height as i32;
            // Our data array is laid out row‑major with row 0 at the top.
            // Here we assume the tile at (tx, ty) in the data array corresponds directly.
            let index = (ty * self.width as i32 + tx) as usize;
            if self.data[index] == 0 {
                return (x, y);
            }
        }
        // Fallback coordinate if no valid tile was found in MAX_ATTEMPTS.
        (1, 1)
    }

    // The following methods remain unchanged (coin generation, NPC movement, etc.)

    pub fn generate_coins(&mut self) {
        for index in 0..100 {
            let (new_x, new_y) = self.get_random_coordinates();
            self.coins[index].x = new_x;
            self.coins[index].y = new_y;
        }
        self.coin_counter = 100;
    }

    pub fn relocate_coins(&mut self, amount: u32) {
        let mut relocate_counter = 0;
        for index in 0..100 {
            if self.coins[index].x == -1 && self.coins[index].y == -1 {
                let (new_x, new_y) = self.get_random_coordinates();
                self.coins[index].x = new_x;
                self.coins[index].y = new_y;
                relocate_counter += 1;
                self.coin_counter += 1;
                if relocate_counter == amount {
                    break;
                }
            }
        }
    }

    pub fn generate_walkers(&mut self) {
        for index in 0..5 {
            let (new_x, new_y) = self.get_random_coordinates();
            self.walkers[index].x = new_x;
            self.walkers[index].y = new_y;
        }
    }

    pub fn generate_dynamites(&mut self) {
        for index in 0..1 {
            let (new_x, new_y) = self.get_random_coordinates();
            self.dynamites[index].x = new_x;
            self.dynamites[index].y = new_y;
        }
    }

    pub fn generate_npcs(&mut self) {
        for index in 0..5 {
            let (new_x, new_y) = self.get_random_coordinates();
            self.npcs[index].x = new_x;
            self.npcs[index].y = new_y;
            // Choose a random direction from the four cardinal directions.
            let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
            let idx = self.get_rand() as usize % 4;
            let (vx, vy) = directions[idx];
            self.npcs[index].vector_x = vx;
            self.npcs[index].vector_y = vy;
            // Random steps between 1 and 4.
            self.npcs[index].steps_remaining = (self.get_rand() % 4) + 1;
        }
    }

    pub fn get_coin_at(&self, x: i32, y: i32) -> Option<Coin> {
        self.coins
            .iter()
            .copied()
            .find(|coin| coin.x == x && coin.y == y)
    }

    pub fn get_npc_at(&self, x: i32, y: i32) -> Option<Npc> {
        self.npcs
            .iter()
            .copied()
            .find(|npc| npc.x == x && npc.y == y)
    }

    pub fn get_walker_at(&self, x: i32, y: i32) -> Option<Coin> {
        self.walkers
            .iter()
            .copied()
            .find(|walker| walker.x == x && walker.y == y)
    }

    pub fn get_dynamite_at(&self, x: i32, y: i32) -> Option<Coin> {
        self.dynamites
            .iter()
            .copied()
            .find(|d| d.x == x && d.y == y)
    }

    pub fn remove_coin(&mut self, coin: Coin) {
        for index in 0..100 {
            if self.coins[index].x == coin.x && self.coins[index].y == coin.y {
                self.coins[index].x = -1;
                self.coins[index].y = -1;
                if self.coin_counter > 0 {
                    self.coin_counter -= 1;
                }
            }
        }
    }

    pub fn relocate_walker(&mut self, walker: Coin) {
        for index in 0..5 {
            if self.walkers[index].x == walker.x && self.walkers[index].y == walker.y {
                let (new_x, new_y) = self.get_random_coordinates();
                self.walkers[index].x = new_x;
                self.walkers[index].y = new_y;
            }
        }
    }

    pub fn relocate_dynamite(&mut self, dynamite: Coin) {
        for index in 0..1 {
            if self.dynamites[index].x == dynamite.x && self.dynamites[index].y == dynamite.y {
                let (new_x, new_y) = self.get_random_coordinates();
                self.dynamites[index].x = new_x;
                self.dynamites[index].y = new_y;
            }
        }
    }

    pub fn set_tile_at(&mut self, x: i32, y: i32, tile: u8) {
        let tile_x = x / self.tile_width as i32;
        let tile_y = y / self.tile_height as i32;
        if tile_x < 0 || tile_y < 0 || tile_x >= self.width as i32 || tile_y >= self.height as i32 {
            return;
        }
        let tile_index = (tile_y * self.width as i32 + tile_x) as usize;
        self.data[tile_index] = tile;
    }

    pub fn place_dynamite(&mut self, x: i32, y: i32) {
        let tw = self.tile_width as i32;
        self.set_tile_at(x - tw, y - tw, 2);
        self.set_tile_at(x, y - tw, 2);
        self.set_tile_at(x + tw, y - tw, 2);
        self.set_tile_at(x - tw, y, 2);
        self.set_tile_at(x + tw, y, 2);
        self.set_tile_at(x - tw, y + tw, 2);
        self.set_tile_at(x, y + tw, 2);
        self.set_tile_at(x + tw, y + tw, 2);
    }

    fn get_random_vector(&mut self) -> (i32, i32) {
        let mut x = self.get_rand() % 3 - 1;
        let y = self.get_rand() % 3 - 1;
        if x == 0 && y == 0 {
            x = 1;
        }
        (x, y)
    }

    pub fn move_npcs(&mut self) {
        for index in 0..5 {
            let mut x = self.npcs[index].x;
            let mut y = self.npcs[index].y;
            x += self.npcs[index].vector_x * self.tile_width as i32;
            y += self.npcs[index].vector_y * self.tile_height as i32;
            if self.check_wall_collision(x, y) {
                let (vx, vy) = self.get_random_vector();
                self.npcs[index].vector_x = vx;
                self.npcs[index].vector_y = vy;
            } else {
                self.npcs[index].x = x;
                self.npcs[index].y = y;
            }
        }
    }

    #[cfg(feature = "static_maze")]
    pub fn generate_maze(&mut self, _graph_width: usize, _graph_height: usize) {
        // No dynamic generation in static mode.
    }

    #[cfg(feature = "dynamic_maze")]
    pub fn generate_maze(&mut self, graph_width: usize, graph_height: usize) {
        let seed: [u8; 32] = self.rng.gen();
        let mut generator = RbGenerator::new(Some(seed));
        let maze_graph = generator
            .generate(graph_width as i32, graph_height as i32)
            .unwrap();
        for y in 1..graph_height {
            for x in 1..graph_width {
                let field = maze_graph.get_field(&(x as i32, y as i32).into()).unwrap();
                let tile_index =
                    (x - 1) * 2 + (y - 1) * 2 * (self.width as usize) + (self.offset as usize);
                self.data[tile_index] = 0;
                if field.has_passage(&Direction::West) {
                    self.data[tile_index + 1] = 0;
                }
                if field.has_passage(&Direction::South) {
                    self.data[tile_index + (self.width as usize)] = 0;
                }
            }
        }
    }

    pub fn playable_bounds(&self) -> (i32, i32, i32, i32) {
        let margin = Self::MARGIN;
        let left = margin * self.tile_width as i32;
        let bottom = margin * self.tile_height as i32;
        let right = left + self.width as i32 * self.tile_width as i32;
        let top = bottom + self.height as i32 * self.tile_height as i32;
        (left, bottom, right, top)
    }
}
