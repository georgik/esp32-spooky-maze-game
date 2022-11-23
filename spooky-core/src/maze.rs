use maze_generator::prelude::*;
use maze_generator::recursive_backtracking::{RbGenerator};


#[derive(Copy, Clone)]
pub struct Coin {
    pub x: i32,
    pub y: i32,
}

impl Coin {
    pub fn new(x: i32, y: i32) -> Coin {
        Coin { x, y }
    }
}

#[derive(Copy, Clone)]
pub struct Npc {
    pub x: i32,
    pub y: i32,
    pub vector_x: i32,
    pub vector_y: i32,
}


pub struct Maze {
    pub width: u32,
    pub height: u32,
    pub visible_width: u32,
    pub visible_height: u32,
    pub data: [u8; 64*64],
    pub coins: [Coin; 100],
    pub coin_counter: u32,
    pub npcs: [Npc; 5],
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
            visible_width: 21,
            visible_height: 16,
            data: [1; 64*64],
            offset: width+1,
            tile_width: 16,
            tile_height: 16,
            coins: [Coin {x: -1, y: -1}; 100],
            coin_counter: 100,
            npcs: [Npc {x: -1, y: -1, vector_x: 0, vector_y: 0}; 5],
        }
    }

    fn get_rand(&self) -> i32 {
        let mut seed_buffer = [0u8;2];
        #[cfg(feature = "getrandom")]
        getrandom::getrandom(&mut seed_buffer).unwrap();
        seed_buffer[0].try_into().unwrap()
    }

    pub fn check_wall_collision(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 {
            return true;
        }
        let tile_x = x / self.tile_width as i32;
        let tile_y = y / self.tile_height as i32;
        let tile_index = (tile_y * self.width as i32 + tile_x) as usize;
        self.data[tile_index] == 1
    }

    pub fn get_random_coordinates(&self) -> (i32, i32) {
        let mut x = (self.get_rand() % (self.width as i32 - 2) + 1) * self.tile_width as i32;
        let mut y = (self.get_rand() % (self.height as i32 - 2) + 1) * self.tile_height as i32;
        while self.check_wall_collision(x, y) {
            x = (self.get_rand() % (self.width as i32 - 2) + 1) * self.tile_width as i32;
            y = (self.get_rand() % (self.height as i32 - 2) + 1) * self.tile_height as i32;
        }
        (x, y)
    }

    pub fn generate_coins(&mut self) {

        for index in 0..100 {
            (self.coins[index].x, self.coins[index].y) = self.get_random_coordinates();
        }
        self.coin_counter = 100;
    }

    pub fn generate_npcs(&mut self) {
        for index in 0..5 {
            (self.npcs[index].x, self.npcs[index].y) = self.get_random_coordinates();
            self.npcs[index].vector_x = 1;
            self.npcs[index].vector_y = 1;
        }
    }

    pub fn get_coin_at(&self, x: i32, y: i32) -> Option<Coin> {
        for coin in self.coins.iter() {
            if coin.x == x && coin.y == y {
                return Some(*coin);
            }
        }
        None
    }

    pub fn get_npc_at(&self, x: i32, y: i32) -> Option<Npc> {
        for npc in self.npcs.iter() {
            if npc.x == x && npc.y == y {
                return Some(*npc);
            }
        }
        None
    }

    pub fn remove_coin(&mut self, coin: Coin) {
        for index in 0..100 {
            if self.coins[index].x == coin.x && self.coins[index].y == coin.y {
                self.coins[index].x = -1;
                self.coins[index].y = -1;
                if self.coin_counter > 0 {
                    self.coin_counter -= 1;
                } else {
                    self.generate_coins();
                }
            }
        }
    }

    fn get_random_vector(&self) -> (i32, i32) {
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
            x += self.npcs[index].vector_x * 16;
            y += self.npcs[index].vector_y * 16;
            if self.check_wall_collision(x, y) {
                (self.npcs[index].vector_x, self.npcs[index].vector_y) = self.get_random_vector();
            } else {
                self.npcs[index].x = x;
                self.npcs[index].y = y;
            }
        }
    }

    pub fn generate_maze(&mut self, graph_width: usize, graph_height: usize) {
        // let mut rng = Rng::new(peripherals.RNG);
        // let mut rng = Rng::new( 0x12345678 );
        let mut seed_buffer = [0u8;32];
        #[cfg(feature = "getrandom")]
        getrandom::getrandom(&mut seed_buffer).unwrap();

        let mut generator = RbGenerator::new(Some(seed_buffer));
        let maze_graph = generator.generate(graph_width as i32, graph_height as i32).unwrap();

        for y in 1usize..graph_height {
            for x in 1usize..graph_width {
                let field = maze_graph.get_field(&(x.try_into().unwrap(),y.try_into().unwrap()).into()).unwrap();
                let tile_index = (x-1)*2+(y-1)*2*(self.width as usize)+(self.offset as usize);

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
}