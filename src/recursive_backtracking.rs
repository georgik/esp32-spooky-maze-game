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
use rand::prelude::*;
use rand_chacha::ChaChaRng;

// use crate::prelude::*;

/// [`Generator`] implementation which uses the recursive-backtracking algorithm.
#[derive(Debug, Clone)]
pub struct RbGenerator {
    rng: ChaChaRng,
}

impl RbGenerator {
    /// Create a new instance.
    ///
    /// Optionally a 32 bit seed can be provided to seed the internal random generator.
    /// Giving a seed results in identical mazes being generated which omitting it sources the
    /// random generator from entropy.
    pub fn new(seed: Option<[u8; 32]>) -> RbGenerator {
        // RbGenerator {
        //     rng: match seed {
        //         None => ChaChaRng::from_entropy(),
        //         Some(seed) => ChaChaRng::from_seed(seed),
        //     },
        // }
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
        for i_dir in Direction::gen_random_order(&mut self.rng).iter() {
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