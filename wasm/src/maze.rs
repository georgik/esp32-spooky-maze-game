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
            npcs: [Npc {x: -1, y: -1, vector_x: 0, vector_y: 0}; 5],
        }
    }

    fn get_rand(&self) -> i32 {
        let mut seed_buffer = [0u8;2];
        getrandom::getrandom(&mut seed_buffer).unwrap();
        seed_buffer[0].try_into().unwrap()
    }

    pub fn generate_coins(&mut self) {

        for index in 0..100 {
            let x:i32 = ((self.get_rand() % 63) + 1) * 16;
            let y:i32 = ((self.get_rand() % 63) + 1) * 16;
            self.coins[index].x = x;
            self.coins[index].y = y;
        }
    }

    pub fn generate_npcs(&mut self) {
        for index in 0..5 {
            let x:i32 = ((self.get_rand() % 63) + 1) * 16;
            let y:i32 = ((self.get_rand() % 63) + 1) * 16;
            self.npcs[index].x = x;
            self.npcs[index].y = y;
            self.npcs[index].vector_x = 1;
            self.npcs[index].vector_y = 1;
        }
    }

    pub fn generate_maze(&mut self, graph_width: usize, graph_height: usize) {
        println!("Rendering maze");

        println!("Initializing Random Number Generator Seed");
        // let mut rng = Rng::new(peripherals.RNG);
        // let mut rng = Rng::new( 0x12345678 );
        let mut seed_buffer = [0u8;32];
        getrandom::getrandom(&mut seed_buffer).unwrap();

        println!("Acquiring maze generator");
        let mut generator = RbGenerator::new(Some(seed_buffer));
        println!("Generating maze");
        let maze_graph = generator.generate(graph_width as i32, graph_height as i32).unwrap();

        println!("Converting to tile maze");
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
