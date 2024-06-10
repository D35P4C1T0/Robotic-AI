use std::collections::VecDeque;

use robotics_lib::interface::{go, robot_map, teleport, Direction};
use robotics_lib::runner::Runnable;
use robotics_lib::utils::LibError;
use robotics_lib::world::tile::{Tile, TileType};
use robotics_lib::world::World;
use sense_and_find_by_rustafariani::Action;

use crate::robot::{BotAction, Scrapbot};

fn is_undiscovered_tile(known_map: &[Vec<Option<Tile>>], x: usize, y: usize) -> bool {
    known_map[x][y].is_none()
}

fn valid_coords(x: i32, y: i32, map_size: i32) -> bool {
    x >= 0 && y >= 0 && x < map_size && y < map_size
}

impl Scrapbot {
    pub(crate) fn go_to_map_center_and_update_lssf(
        &mut self,
        world: &mut World,
    ) -> Result<(), LibError> {
        println!("Going to map center");
        let map_size = robot_map(world).unwrap().len();
        let center = map_size / 2;

        // Get the current coordinates of the robot
        let (robot_x, robot_y) = (
            self.get_coordinate().get_col(),
            self.get_coordinate().get_row(),
        );

        // Calculate the number of moves needed in each direction
        let (horizontal_moves, horizontal_direction) = if robot_x < center {
            (center - robot_x, Direction::Right)
        } else {
            (robot_x - center, Direction::Left)
        };

        let (vertical_moves, vertical_direction) = if robot_y < center {
            (center - robot_y, Direction::Down)
        } else {
            (robot_y - center, Direction::Up)
        };

        // Move horizontally
        for _ in 0..horizontal_moves {
            go(self, world, horizontal_direction.clone())?;
        }

        // Move vertically
        for _ in 0..vertical_moves {
            go(self, world, vertical_direction.clone())?;
        }

        // Uncomment the line below to update LSSF
        self.lssf_update(world, Some(robot_map(world).unwrap().len()))
    }

    pub(crate) fn nearest_border_distance(&self, world: &World) -> usize {
        let robot_pos = self.get_coordinate();
        // Assumiamo che robot_map(world) restituisca una griglia quadrata,
        // quindi prendiamo la lunghezza di uno dei lati per calcolare world_size.

        let world_size = robot_map(world).unwrap().len();

        let row = robot_pos.get_row();
        let col = robot_pos.get_col();

        // Calcola le distanze dai quattro bordi della mappa.
        let dist_top = row; // Distanza dal bordo superiore.
        let dist_bottom = world_size - row - 1; // Distanza dal bordo inferiore.
        let dist_left = col; // Distanza dal bordo sinistro.
        let dist_right = world_size - col - 1; // Distanza dal bordo destro.

        // Restituisce la distanza minima tra quelle calcolate.
        *[dist_top, dist_bottom, dist_left, dist_right]
            .iter()
            .min()
            .unwrap()
    }

    pub(crate) fn bfs_find_closest_undiscovered_tile(
        &mut self,
        world: &mut World,
    ) -> Option<(usize, usize)> {
        let robot_x = self.get_coordinate().get_col();
        let robot_y = self.get_coordinate().get_row();
        let known_map = robot_map(world).unwrap();
        let map_size = known_map.len() as i32;
        let mut visited = vec![vec![false; map_size as usize]; map_size as usize];
        let mut queue = VecDeque::new();

        // Mark impassable tiles as visited
        for i in 0..map_size {
            for j in 0..map_size {
                if let Some(tile) = known_map[i as usize][j as usize].clone() {
                    match tile.tile_type {
                        TileType::Lava | TileType::DeepWater | TileType::Wall => {
                            visited[i as usize][j as usize] = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        queue.push_back((robot_x, robot_y));
        visited[robot_x][robot_y] = true;

        while let Some((x_u, y_u)) = queue.pop_front() {
            if is_undiscovered_tile(&known_map, x_u, y_u) {
                return Some((x_u, y_u));
            }

            for (dx, dy) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (x_next, y_next) = (x_u as i32 + dx, y_u as i32 + dy);
                if valid_coords(x_next, y_next, map_size)
                    && !visited[x_next as usize][y_next as usize]
                {
                    queue.push_back((x_next as usize, y_next as usize));
                    visited[x_next as usize][y_next as usize] = true;
                }
            }
        }

        None
    }
    pub fn populate_action_vec_given_point(&mut self, coordinate: (usize, usize)) {
        let old_lssf = self.lssf.take().unwrap();
        match old_lssf.get_action_vec(coordinate.0, coordinate.1) {
            // col(x), row(y)
            Ok(actions) => {
                println!("Populated action vec!: {:?}", actions);
                self.actions_vec = Some(actions);
            }
            Err(err) => {
                println!("Error planning next move to: {:?} | {:?}", coordinate, err);
            }
        }
        self.lssf = Some(old_lssf);
    }
    pub(crate) fn run_action_vec_and_then(
        &mut self,
        world: &mut World,
        action: BotAction,
    ) -> Result<usize, LibError> {
        // Run the actions vector if it exists
        if let Some(mut actions) = self.actions_vec.take() {
            // check if the action vector is empty
            if actions.is_empty() {
                println!("No actions to perform");
                return Err(LibError::CannotWalk);
            }

            // Get the last move direction from the first element of the actions vector
            let last_move_direction = match actions.remove(0) {
                Action::North => Direction::Up,
                Action::South => Direction::Down,
                Action::East => Direction::Right,
                Action::West => Direction::Left,
                _ => Direction::Up, // Default to Up, though ideally should never hit this case
            };

            // Execute the actions in the vector
            self.full_recharge();
            for action in &actions {
                match action {
                    Action::North => {
                        go(self, world, Direction::Up).ok();
                    }
                    Action::South => {
                        go(self, world, Direction::Down).ok();
                    }
                    Action::East => {
                        go(self, world, Direction::Right).ok();
                    }
                    Action::West => {
                        go(self, world, Direction::Left).ok();
                    }
                    Action::Teleport(row, col) => {
                        teleport(self, world, (*row, *col)).ok();
                    }
                }
            }

            self.actions_vec = Some(actions); // Put the modified vector back

            // Perform the final action
            self.full_recharge();
            let result = match action {
                BotAction::Destroy => self.collect_trash_in_front_of(world, last_move_direction),
                BotAction::Put => self.drop_trash_into_bin_in_front_of(world, last_move_direction),
                BotAction::Start => {
                    self.actions_vec.as_mut().unwrap().clear();
                    return Ok(0);
                }
                BotAction::Walk => return Ok(0),
            };

            match result {
                Ok(q) => {
                    println!("Processed {} trash", q);
                    Ok(q)
                }
                Err(err) => {
                    println!("Error processing action: {:?}", err);
                    Err(err)
                }
            }
        } else {
            Ok(0)
        }
    }

    pub(crate) fn get_last_move_direction(&self) -> Direction {
        match self.actions_vec.as_ref().unwrap().last().unwrap() {
            Action::North => Direction::Up,
            Action::South => Direction::Down,
            Action::East => Direction::Right,
            Action::West => Direction::Left,
            _ => Direction::Up, // hope it doesnt get here
        }
    }

    pub(crate) fn next_quadrant_clockwise(&mut self, world: &mut World) -> (usize, usize) {
        let map_side = robot_map(world).unwrap().len();
        let quadrants_centers = [
            (map_side / 4, map_side / 4),
            ((map_side / 4) * 3, map_side / 4),
            ((map_side / 4) * 3, (map_side / 4) * 3),
            (map_side / 4, (map_side / 4) * 3),
        ]; // col,row

        // 1 | 2
        // -----
        // 4 | 3
        // clockwise

        let (bot_col, bot_row) = (
            self.get_coordinate().get_col(),
            self.get_coordinate().get_row(),
        );

        let bot_location_quadrant: usize = match (bot_row <= map_side / 2, bot_col <= map_side / 2)
        {
            (true, true) => 1,
            (true, false) => 2,
            (false, false) => 3,
            (false, true) => 4,
        };

        // mark current quadrant as visited
        self.quadrants_visited
            .entry(bot_location_quadrant)
            .and_modify(|e| *e = true)
            .or_insert(true);

        let next_quadrant = match bot_location_quadrant {
            1 => 2,
            2 => 3,
            3 => 4,
            4 => 1,
            _ => 1,
        };

        quadrants_centers[next_quadrant - 1]
    }
}
