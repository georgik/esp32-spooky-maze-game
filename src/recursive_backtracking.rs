// Source: https://github.com/ftsell/maze_generator/blob/master/src/recursive_backtracking.rs

//! Recursive-Backtracking algorithm implementation
//!
//! Recursive backtracking is fast, easy to understand and straightforward.
//! Its downsides are relatively large memory and stack size requirements.
//!
//! The algorithm works as follows:
//!
//! 1. Choose a starting point in the field (in this implementation 0,0) and make it the current cell
//! 2. Randomly choose a direction, check if the field in that direction has not yet been visited.
//!     If that is the case, make the cell in that direction the new current cell and carve a passage between the two.
//! 3. If all adjacent fields have been visited, back up to the last field with unvisited neighbors.
//! 4. The algorithm terminates when it has backed up all the way to the starting point.

// use anyhow::Result;
use petgraph::Undirected;
use petgraph::lib::Vec;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Direction {
    /// *North* or *up* direction
    North,
    /// *East* or *right* direction
    East,
    /// *South* or *down* direction
    South,
    /// *West* or *left* direction
    West,
}

impl Direction {
    /// Return the opposite direction of self
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }

    /// Generate a list of all collections but in random order
    pub fn gen_random_order(rng: &mut impl Rng) -> [Direction; 4] {
        let mut directions = Self::all();
        directions.shuffle(rng);
        directions
    }

    /// Return all directions as array
    pub fn all() -> [Direction; 4] {
        [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
    }
}

#[derive(Debug, Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Coordinates {
    /// X component
    pub x: i32,
    /// Y component
    pub y: i32,
}

impl Coordinates {
    /// Create a new instance from the specified coordinate components
    pub fn new(x: i32, y: i32) -> Self {
        Coordinates { x, y }
    }

    /// Returns the next neighboring coordinates in a specific direction
    pub fn next(&self, direction: &Direction) -> Self {
        Self {
            x: self.x
                + match direction {
                    Direction::East => 1,
                    Direction::West => -1,
                    _ => 0,
                },
            y: self.y
                + match direction {
                    Direction::North => -1,
                    Direction::South => 1,
                    _ => 0,
                },
        }
    }
}

impl From<Coordinates> for (i32, i32) {
    fn from(c: Coordinates) -> Self {
        (c.x, c.y)
    }
}

impl From<(i32, i32)> for Coordinates {
    fn from(source: (i32, i32)) -> Self {
        Self {
            x: source.0,
            y: source.1,
        }
    }
}



pub(crate) type MazeGraph = GraphMap<Coordinates, (), Undirected>;
#[derive(Clone)]
pub struct Maze {
    pub(crate) graph: MazeGraph,
    /// At which coordinates the start field lies
    pub start: Coordinates,
    /// At which coordinates the goal field lies
    pub goal: Coordinates,
    /// How large the maze is in (width, height) format
    pub size: (i32, i32),
}

// Maze part https://github.com/ftsell/maze_generator/blob/master/src/prelude/maze.rs

use petgraph::graphmap::GraphMap;

/// Defines the possible types of fields that exist in a maze
#[derive(Debug, Copy, Clone)]
pub enum FieldType {
    /// Start field from which a potential user should start exploring
    Start,
    /// Goal field which a potential user tries to reach
    Goal,
    /// Standard field with no special meaning
    Normal,
}

#[derive(Clone)]
pub struct Field {
    passages: Vec<Direction>,
    /// Role which this field position serves in the maze
    pub field_type: FieldType,
    /// Where this field is located in the maze
    pub coordinates: Coordinates,
}

impl Field {
    pub(crate) fn new(
        field_type: FieldType,
        coordinates: Coordinates,
        passages: Vec<Direction>,
    ) -> Self {
        Field {
            passages,
            field_type,
            coordinates,
        }
    }

    /// Whether or not a passage (a way) exists from this field to another one which lies in the
    /// specified direction.
    pub fn has_passage(&self, direction: &Direction) -> bool {
        self.passages.contains(direction)
    }
}


impl Maze {
    pub(crate) fn new(width: i32, height: i32, start: Coordinates, goal: Coordinates) -> Self {
        debug_assert!(width > 0, "maze width should be >0");
        debug_assert!(height > 0, "maze height should be >0");

        Maze {
            graph: GraphMap::with_capacity((width * height) as usize, 0),
            size: (width, height),
            start,
            goal,
        }
    }

    /// Retrieve the [`Field`] which is located at `coordinates`
    pub fn get_field(&self, coordinates: &Coordinates) -> Option<Field> {
        if self.are_coordinates_inside(coordinates) {
            // figure out in which directions passages exist
            let passages: Vec<_> = Direction::all()
                .iter()
                .filter(|dir| {
                    self.graph
                        .contains_edge(*coordinates, coordinates.next(dir))
                })
                .copied()
                .collect();

            let field_type = if &self.start == coordinates {
                FieldType::Start
            } else if &self.goal == coordinates {
                FieldType::Goal
            } else {
                FieldType::Normal
            };

            Some(Field::new(field_type, *coordinates, passages))
        } else {
            None
        }
    }

    pub(crate) fn are_coordinates_inside(&self, coordinates: &Coordinates) -> bool {
        coordinates.x >= 0
            && coordinates.x < self.size.0
            && coordinates.y >= 0
            && coordinates.y < self.size.1
    }
}

use anyhow::Result;

use rand::prelude::*;
use rand_chacha::ChaChaRng;

// use crate::prelude::*;

/// [`Generator`] implementation which uses the recursive-backtracking algorithm.
#[derive(Debug, Clone)]
pub struct RbGenerator {
    rng: ChaChaRng,
}

use esp_println::{println, print};
/// Generic generator Api implemented by all algorithms to generate a maze
pub trait Generator {
    /// Key function to generate a maze
    ///
    /// The returned [`Maze`] will have the provided width and height.
    /// It can be any rectangular shape.
    fn generate(&mut self, width: i32, height: i32) -> Result<Maze>;
}
impl RbGenerator {
    /// Create a new instance.
    ///
    /// Optionally a 32 bit seed can be provided to seed the internal random generator.
    /// Giving a seed results in identical mazes being generated which omitting it sources the
    /// random generator from entropy.
    pub fn new(seed: Option<[u8; 32]>) -> RbGenerator {
        RbGenerator {
            rng: match seed {
                None => ChaChaRng::from_seed([42; 32]),
                Some(seed) => ChaChaRng::from_seed(seed),
            },
        }
    }

    /// Core algorithm implementation
    ///
    /// Carves passages in all directions in random order from the current coordinates but only
    /// if the field in that direction has not yet been processed.
    ///
    /// Returns coordinates of the goal field
    fn carve_passages_from(
        &mut self,
        maze: &mut Maze,
        current_coordinates: Coordinates,
    ) -> Coordinates {
        let mut goal_coords = maze.start;
        print!("+");
        for i_dir in Direction::gen_random_order(&mut self.rng).iter() {
            print!(".");
            let next_coords = current_coordinates.next(i_dir);

            if maze.are_coordinates_inside(&next_coords)
                && maze.graph.neighbors(next_coords).count() == 0
            {
                maze.graph.add_edge(current_coordinates, next_coords, ());
                if goal_coords == maze.start {
                    goal_coords = self.carve_passages_from(maze, next_coords);
                } else {
                    self.carve_passages_from(maze, next_coords);
                }
            }
        }
        print!("-");

        if goal_coords == maze.start {
            current_coordinates
        } else {
            goal_coords
        }
    }
}

impl Generator for RbGenerator {
    fn generate(&mut self, width: i32, height: i32) -> Result<Maze> {
        let start = (0, 0).into();
        let mut maze = Maze::new(width, height, start, (0, 0).into());
        maze.graph.add_node(start);

        let goal = self.carve_passages_from(&mut maze, start);
        maze.goal = goal;
        println!(" Ok");

        Ok(maze)
    }
}

#[cfg(test)]
mod test {
    test_all_coordinates_have_fields!(super::RbGenerator);
    test_route_from_start_to_goal_exists!(super::RbGenerator);
    test_all_fields_connected!(super::RbGenerator);
    test_generation_is_deterministic!(super::RbGenerator);
}