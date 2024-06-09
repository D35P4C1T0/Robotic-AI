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
    pub(crate) fn bfs_find_closest_undiscovered_tile(
        &mut self,
        world: &mut World,
    ) -> Option<(usize, usize)> {
        let robot_x = self.get_coordinate().get_row();
        let robot_y = self.get_coordinate().get_col();
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
            Ok(actions) => {
                self.actions_vec = Some(actions);
            }
            Err(err) => {
                println!("Error planning next move: {:?}", err);
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
            (map_side / 4, (map_side / 4) * 3),
            ((map_side / 4) * 3, (map_side / 4) * 3),
            ((map_side / 4) * 3, map_side / 4),
        ];

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