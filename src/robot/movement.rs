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

fn valid_coords_no_border(x: i32, y: i32, map_size: i32) -> bool {
    x >= 0 && y >= 0 && x < map_size && y < map_size
}

impl Scrapbot {
    pub(crate) fn move_away_from_border(&mut self, world: &mut World) -> bool {
        let map_size = robot_map(world).unwrap().len();
        let robot_pos = self.get_coordinate();
        let (robot_x, robot_y) = (robot_pos.get_col(), robot_pos.get_row());

        let min_distance = 4;
        let mut moved = true;

        let mut moves_stack: Vec<Direction> = vec![];

        match (
            robot_x < min_distance,
            robot_y < min_distance,
            robot_y >= map_size - min_distance,
            robot_x >= map_size - min_distance,
        ) {
            (true, false, false, false) => {
                moves_stack.extend(std::iter::repeat(Direction::Right).take(min_distance))
            }
            (false, true, false, false) => {
                moves_stack.extend(std::iter::repeat(Direction::Down).take(min_distance))
            }
            (false, false, true, false) => {
                moves_stack.extend(std::iter::repeat(Direction::Up).take(min_distance))
            }
            (false, false, false, true) => {
                moves_stack.extend(std::iter::repeat(Direction::Left).take(min_distance))
            }
            _ => moved = false,
        }

        for direction in &moves_stack {
            go(self, world, direction.clone()).ok();
            print!("Moved away from border, {:?} | ", direction);
        }

        self.lssf_update(world, Some(min_distance * 2));

        moved
    }

    pub(crate) fn move_to_center(&mut self, world: &mut World) {
        let map_size = robot_map(world).unwrap().len();
        let robot_pos = self.get_coordinate();
        let (robot_x, robot_y) = (robot_pos.get_col(), robot_pos.get_row());
        let center = map_size / 2;

        let mut moves_stack: Vec<Direction> = vec![];

        match (robot_x < center, robot_y < center) {
            (true, true) => {
                moves_stack.extend(std::iter::repeat(Direction::Right).take(center - robot_x));
                moves_stack.extend(std::iter::repeat(Direction::Down).take(center - robot_y));
            }
            (true, false) => {
                moves_stack.extend(std::iter::repeat(Direction::Right).take(center - robot_x));
                moves_stack.extend(std::iter::repeat(Direction::Up).take(robot_y - center));
            }
            (false, true) => {
                moves_stack.extend(std::iter::repeat(Direction::Left).take(robot_x - center));
                moves_stack.extend(std::iter::repeat(Direction::Down).take(center - robot_y));
            }
            (false, false) => {
                moves_stack.extend(std::iter::repeat(Direction::Left).take(robot_x - center));
                moves_stack.extend(std::iter::repeat(Direction::Up).take(robot_y - center));
            }
        }

        for direction in &moves_stack {
            go(self, world, direction.clone()).ok();
            print!("Moved to center, {:?} | ", direction);
        }

        self.lssf_update(world, Some((center * 2) - 1));
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

    fn opposite_border_coords(x: i32, y: i32, map_size: i32) -> (i32, i32) {
        let new_x = if x < 4 {
            map_size - 5
        } else if x >= map_size - 4 {
            4
        } else {
            x
        };
        let new_y = if y < 4 {
            map_size - 5
        } else if y >= map_size - 4 {
            4
        } else {
            y
        };
        (new_x, new_y)
    }

    pub fn find_closest_undiscovered_tile(&mut self, world: &mut World) -> Option<(usize, usize)> {
        let robot_x = self.get_coordinate().get_row();
        let robot_y = self.get_coordinate().get_col();
        let known_map = robot_map(world).unwrap();
        let map_size = known_map.len() as i32;
        let border_limit = 4; // Distance from the border to be avoided
        let mut visited = vec![vec![false; map_size as usize]; map_size as usize];
        let mut queue = VecDeque::new();

        // Mark the border and impassable tiles as visited
        for i in 0..map_size {
            for j in 0..map_size {
                if i < border_limit
                    || i >= map_size - border_limit
                    || j < border_limit
                    || j >= map_size - border_limit
                {
                    visited[i as usize][j as usize] = true;
                } else if let Some(tile) = &known_map[i as usize][j as usize] {
                    match tile.tile_type {
                        TileType::Lava | TileType::DeepWater | TileType::Wall => {
                            visited[i as usize][j as usize] = true
                        }
                        _ => {}
                    }
                }
            }
        }

        // Start BFS from the robot's position
        queue.push_back((robot_x, robot_y));
        visited[robot_x][robot_y] = true;

        while let Some((x, y)) = queue.pop_front() {
            let x_i = x as i32;
            let y_i = y as i32;

            for (dx, dy) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x_i + dx, y_i + dy);
                if Self::valid_coords(nx, ny, map_size, border_limit) {
                    let (nx_usize, ny_usize) = (nx as usize, ny as usize);
                    if !visited[nx_usize][ny_usize] {
                        if known_map[nx_usize][ny_usize].is_none() {
                            // Found an undiscovered tile, now return any adjacent discovered tile
                            for (adx, ady) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                                let (ax, ay) = (nx + adx, ny + ady);
                                if Self::valid_coords(ax, ay, map_size, border_limit) {
                                    let (ax_usize, ay_usize) = (ax as usize, ay as usize);
                                    if known_map[ax_usize][ay_usize].is_some()
                                        && self.valid_lssf_coords(ax_usize, ay_usize)
                                    {
                                        return Some((ax_usize, ay_usize));
                                    }
                                }
                            }
                        }
                        queue.push_back((nx_usize, ny_usize));
                        visited[nx_usize][ny_usize] = true;
                    }
                } else {
                    // Handle the case where the robot is at the border
                    let (opposite_x, opposite_y) = Self::opposite_border_coords(nx, ny, map_size);
                    if Self::valid_coords(opposite_x, opposite_y, map_size, border_limit) {
                        let (opposite_usize_x, opposite_usize_y) =
                            (opposite_x as usize, opposite_y as usize);
                        if !visited[opposite_usize_x][opposite_usize_y] {
                            if known_map[opposite_usize_x][opposite_usize_y].is_some()
                                && self.valid_lssf_coords(opposite_usize_x, opposite_usize_y)
                            {
                                return Some((opposite_usize_x, opposite_usize_y));
                            }
                            queue.push_back((opposite_usize_x, opposite_usize_y));
                            visited[opposite_usize_x][opposite_usize_y] = true;
                        }
                    }
                }
            }
        }

        // If no undiscovered tile is found, return a coordinate near the center
        Some((map_size as usize / 2, map_size as usize / 2))
    }

    fn valid_coords(x: i32, y: i32, map_size: i32, border_limit: i32) -> bool {
        x >= border_limit
            && x < map_size - border_limit
            && y >= border_limit
            && y < map_size - border_limit
    }

    pub fn populate_action_vec_given_point(&mut self, world: &World, coordinate: (usize, usize)) {
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

            let mut last_move_direction = None;

            // Get the last move direction from the first element of the actions vector
            match action {
                BotAction::Destroy | BotAction::Put => {
                    last_move_direction = match actions.remove(0) {
                        Action::North => Some(Direction::Up),
                        Action::South => Some(Direction::Down),
                        Action::East => Some(Direction::Right),
                        Action::West => Some(Direction::Left),
                        _ => None, // Default to Up, though ideally should never hit this case
                    };
                }
                _ => (),
            }

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
                BotAction::Destroy => {
                    self.collect_trash_in_front_of(world, last_move_direction.unwrap())
                }
                BotAction::Put => {
                    self.drop_trash_into_bin_in_front_of(world, last_move_direction.unwrap())
                }
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

    pub(crate) fn get_last_move_direction(&self) -> Option<Direction> {
        if self.actions_vec.as_ref().unwrap().is_empty() {
            return None;
        }
        match self.actions_vec.as_ref().unwrap().last().unwrap() {
            Action::North => Some(Direction::Up),
            Action::South => Some(Direction::Down),
            Action::East => Some(Direction::Right),
            Action::West => Some(Direction::Left),
            _ => None, // hope it doesnt get here
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
