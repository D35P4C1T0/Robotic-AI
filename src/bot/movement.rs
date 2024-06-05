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
        // used to run the actions vector
        if self.actions_vec.is_some() {
            // last_move is the last element of the actions vector
            // remove the last element of the actions vector
            // and put it in last_move
            let last_move_direction = match self.actions_vec.as_mut().unwrap().remove(0) {
                Action::North => Direction::Up,
                Action::South => Direction::Down,
                Action::East => Direction::Right,
                Action::West => Direction::Left,
                _ => Direction::Up, // hope it doesnt get here
            };

            // iterate over the moves and execute them, moving the robot
            if let Some(mut actions) = self.actions_vec.take() {
                self.full_recharge(); // because why not
                for action in &mut actions {
                    match action {
                        Action::North => if go(self, world, Direction::Up).is_ok() {},
                        Action::South => if go(self, world, Direction::Down).is_ok() {},
                        Action::East => if go(self, world, Direction::Right).is_ok() {},
                        Action::West => if go(self, world, Direction::Left).is_ok() {},
                        Action::Teleport(row, col) => {
                            if teleport(self, world, (*row, *col)).is_ok() {}
                        }
                    }
                }
                // Put the modified vector back into the option
                self.actions_vec = Some(actions);
            }

            // at this point, the bot should be in front of the desired content
            self.full_recharge(); // because why not

            match action {
                BotAction::Destroy => {
                    return match self.collect_trash_in_front_of(world, last_move_direction.clone())
                    {
                        Ok(q) => {
                            if q == 0 {
                                self.must_find_new_trash = true;
                            }
                            println!("Collected {} trash", q);
                            Ok(q)
                        }
                        Err(err) => {
                            println!("Error collecting trash: {:?}", err);
                            Err(err)
                        }
                    };
                }
                BotAction::Put => {
                    let quantity = self.get_content_quantity(&Content::Garbage(0));
                    return match self.drop_trash_into_bin(world, last_move_direction, quantity) {
                        Ok(q) => {
                            if q == 0 {
                                self.must_find_new_trash = true;
                            }
                            println!("Dropped {} trash", q);
                            Ok(q)
                        }
                        Err(err) => {
                            println!("Error dropping trash: {:?}", err);
                            Err(err)
                        }
                    };
                }
                BotAction::Start => {
                    self.actions_vec.as_mut().unwrap().clear(); // clear the actions vector
                }
                BotAction::Walk => {
                    // do nothing, just reach the spot
                    return Ok(0);
                }
            }
            self.actions_vec.as_mut().unwrap().clear(); // clear the actions vector
        }
        Ok(0)
    }

    // needed??
    pub fn move_to_coords_and_do_action(&mut self, coords: (usize, usize), world: &mut World) {
        let result = self
            .lssf
            .take()
            .unwrap()
            .smart_sensing_centered(5, world, self, 1);

        match result {
            Ok(_) => {
                self.actions_vec = Some(
                    self.lssf
                        .take()
                        .unwrap()
                        .get_action_vec(coords.0, coords.1)
                        .unwrap(),
                );
            }
            Err(err) => {
                println!("Error moving to coords: {:?}", err);
            }
        }

        // need to call methods to go and collect trash
        // or dispose it in the bin right after this method
    }

    pub fn next_quadrant_clockwise(&self, world: &mut World) -> (usize, usize) {
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

        let bot_location_quadrant = match (bot_row <= map_side / 2, bot_col <= map_side / 2) {
            (true, true) => 1,
            (true, false) => 2,
            (false, false) => 3,
            (false, true) => 4,
        };

        let next_quadrant = match bot_location_quadrant {
            1 => 2,
            2 => 3,
            3 => 4,
            4 => 1,
            _ => 1,
        };

        quadrants_centers[next_quadrant - 1]
    }

    // They call me the wanderer
    // Yeah, the wanderer
    // I roam around, around, around
    pub fn routine_wander(&mut self, world: &mut World) {
        // TODO: set the next quadrant center as the target
    }
}
