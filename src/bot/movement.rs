use robotics_lib::interface::{go, robot_map, teleport, Direction};
use robotics_lib::runner::Runnable;
use robotics_lib::utils::LibError;
use robotics_lib::world::tile::Content;
use robotics_lib::world::World;
use sense_and_find_by_rustafariani::Action;

use crate::bot::{BotAction, Scrapbot};

impl Scrapbot {
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
    pub fn run_action_vec_and_then(
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

    pub fn get_last_move_direction(&self) -> Direction {
        match self.actions_vec.as_ref().unwrap().last().unwrap() {
            Action::North => Direction::Up,
            Action::South => Direction::Down,
            Action::East => Direction::Right,
            Action::West => Direction::Left,
            _ => Direction::Up, // hope it doesnt get here
        }
    }

    pub fn next_quadrant_clockwise(&mut self, world: &mut World) -> (usize, usize) {
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
