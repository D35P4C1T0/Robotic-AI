use robotics_lib::interface::{robot_map, where_am_i};
use robotics_lib::utils::LibError;
use robotics_lib::world::tile::Content;
use robotics_lib::world::World;

use crate::bot::{BotAction, Scrapbot, MAX_BACKPACK_ITEMS};

enum RoutineResult {
    Success,
    FilledBackpack,
    Failure,
    NoChanges,
    NewResourcesNotFound,
    FoundFullBin,
    EmptyBackpack,
    EmptyTrashFound,
    Wandering,
}

impl Scrapbot {
    pub fn routine_collect_trash(&mut self, world: &mut World) -> Result<RoutineResult, LibError> {
        self.full_recharge();
        let lssf_result = self.lssf_update(world, None);
        let mut trash_gathered = 0usize;
        return match lssf_result {
            Ok(_) => {
                match self.lssf_search_trash(world) {
                    Ok(trash_found) => {
                        if trash_found {
                            // iterate over trash location, populate the action vec to
                            // reach that location and call
                            // collect_new_trash
                            let trash_coords = self.trash_coords.take().unwrap();
                            for coords in trash_coords {
                                if self.get_remaining_backpack_space() == 0 {
                                    // totally filled the backpack
                                    return Ok(RoutineResult::FilledBackpack);
                                }

                                // this creates the actions to do in order to reach point
                                self.populate_action_vec_given_point(coords);

                                // now I should be near some trash
                                let walk_result =
                                    self.run_action_vec_and_then(world, BotAction::Walk);
                                if walk_result.is_ok() {
                                    let collect_result =
                                        self.collect_new_trash_fill_backpack(world);
                                    match collect_result {
                                        Ok(q) => {
                                            if q == 0 {
                                                println!("Got no trash, sadly");
                                                return Ok(RoutineResult::EmptyTrashFound);
                                            }
                                            trash_gathered += q;
                                            println!("Collected {} trash", q);
                                            // Ok(RoutineResult::Success)
                                        }
                                        Err(err) => {
                                            println!("Error collecting trash: {:?}", err);
                                            return Err(err);
                                        }
                                    };
                                }
                            }
                            if trash_gathered == 0 {
                                return Ok(RoutineResult::NoChanges);
                            }
                            println!("Collected a total of {} trash", trash_gathered);
                            Ok(RoutineResult::Success)
                        } else {
                            println!("No trash found");
                            Ok(RoutineResult::NewResourcesNotFound)
                        }
                    }
                    Err(err) => {
                        println!("Error searching trash: {:?}", err);
                        Err(err)
                    }
                }
            }
            Err(err) => {
                println!("Error finding garbage: {:?}", err);
                Err(err)
            }
        };
    }

    pub fn routine_empty_trash(&mut self, world: &mut World) -> Result<RoutineResult, LibError> {
        self.full_recharge();
        let lssf_result = self.lssf_update(world, None);

        if self.get_content_quantity(&Content::Garbage(0)) == 0 {
            println!("No trash to drop");
            return Ok(RoutineResult::EmptyBackpack);
        }

        match lssf_result {
            Ok(_) => {
                match self.lssf_search_bins(world) {
                    Ok(bin_found) => {
                        if bin_found {
                            // iterate over bin location, populate the action vec to
                            // reach that location and call
                            // drop_trash_into_bin
                            let bin_coords = self.bin_coords.take().unwrap();
                            let mut bad_bins_vec = vec![];
                            if !bin_coords.is_empty() {
                                for coords in &bin_coords {
                                    // this creates the actions to do in order to reach point
                                    self.populate_action_vec_given_point(coords.clone());
                                    let put_direction = self.get_last_move_direction();

                                    // now I should be near some bin
                                    let walk_result =
                                        self.run_action_vec_and_then(world, BotAction::Walk);
                                    if walk_result.is_ok() {
                                        let drop_result = self
                                            .drop_trash_into_bin_in_front_of(world, put_direction);
                                        match drop_result {
                                            Ok(0) => {
                                                println!("Dropped no trash, sadly found bin full");
                                                println!(
                                                    "Removing bin at {}{} from list",
                                                    coords.0, coords.1
                                                );
                                                // here I should remove the bin from the list
                                                // skipp this for loop iteration
                                                bad_bins_vec.push(coords.clone());
                                            }
                                            Ok(999) => {
                                                println!("Tried to drop 0 trash");
                                                return Ok(RoutineResult::EmptyBackpack);
                                            }
                                            Ok(q) => {
                                                println!("Dropped {} trash", q);
                                                return Ok(RoutineResult::Success);
                                            }
                                            Err(err) => {
                                                println!("Error dropping trash: {:?}", err);
                                                return Err(err);
                                            }
                                        };
                                    }
                                }
                                // remove bad bins from list using filter
                                self.bin_coords = Some(
                                    bin_coords
                                        .into_iter()
                                        .filter(|x| !bad_bins_vec.contains(x))
                                        .collect(),
                                );
                            } else {
                                println!("No bin found");
                                return Ok(RoutineResult::NewResourcesNotFound);
                            }
                        } else {
                            println!("No bin found");
                            return Ok(RoutineResult::NewResourcesNotFound);
                        }
                    }
                    Err(err) => {
                        println!("Error searching bins: {:?}", err);
                        return Err(err);
                    }
                }
            }
            Err(err) => {
                println!("Error finding garbage: {:?}", err);
                return Err(err);
            }
        }
        Ok(RoutineResult::NewResourcesNotFound)
    }

    // They call me the wanderer
    // Yeah, the wanderer
    // I roam around, around, around
    pub fn routine_wander(&mut self, world: &mut World) -> Result<RoutineResult, LibError> {
        // this routine is called when the bot has no more trash to collect
        self.full_recharge();
        let new_location = self.next_quadrant_clockwise(world);

        self.populate_action_vec_given_point(new_location);
        let walk_result = self.run_action_vec_and_then(world, BotAction::Walk);

        return match walk_result {
            Ok(_) => Ok(RoutineResult::Success),
            Err(err) => {
                println!("Error wandering: {:?}", err);
                Err(err)
            }
        };
        // then you should call the trash routine or bin routine accordingly
    }

    // TODO: rework routine method
    pub fn routine(&mut self, world: &mut World) {
        if self.quadrants_visited.values().all(|&v| v) {
            println!("All quadrants visited, stopping");
            return;
        }

        // first time calling routine
        if self.actions_vec.is_none() {
            self.actions_vec = Some(vec![]);
        }
        if self.bin_coords.is_none() {
            self.bin_coords = Some(vec![]);
        }
        if self.trash_coords.is_none() {
            self.trash_coords = Some(vec![]);
        }

        if self.get_remaining_backpack_space() == MAX_BACKPACK_ITEMS {
            let trash_collect_routine_result = self.routine_collect_trash(world);
            match trash_collect_routine_result {
                Ok(RoutineResult::Success) => {
                    println!("Trash collected");
                    return;
                    // I'll wait next cycle to empty the trash
                }
                Ok(RoutineResult::NewResourcesNotFound) => {
                    // wander here
                    match self.routine_wander(world) {
                        Ok(RoutineResult::Success) => {
                            println!("Went to next quadrant");
                            return;
                        }
                        _ => {
                            println!("Error wandering");
                            return;
                        }
                    }
                }
                Ok(RoutineResult::FilledBackpack) => {
                    println!("Backpack is full");
                    let trash_drop_routine_result = self.routine_empty_trash(world);
                    match trash_drop_routine_result {
                        Ok(RoutineResult::Success) => {
                            println!("Trash dropped");
                            return;
                        }
                        Ok(RoutineResult::EmptyBackpack) => {
                            println!("Backpack is empty");
                            // Will need to get new trash
                            return;
                        }
                        Ok(RoutineResult::NewResourcesNotFound) => {
                            // wander here
                            let wander_result = self.routine_wander(world);
                            match wander_result {
                                Ok(RoutineResult::Success) => {
                                    println!("Went to next quadrant");
                                    return;
                                }
                                _ => {
                                    println!("Error wandering");
                                    return;
                                }
                            }
                        }
                        _ => {
                            println!("Error planning next task");
                            return;
                        }
                    }
                }
                _ => {
                    println!("Error planning next task");
                    return;
                }
            }
        }

        if self.get_remaining_backpack_space() == MAX_BACKPACK_ITEMS {
            // empty backpack
            println!("Backpack is full");
            let trash_drop_routine_result = self.routine_empty_trash(world);
            match trash_drop_routine_result {
                Ok(RoutineResult::Success) => {
                    println!("Trash dropped");
                    return;
                }
                Ok(RoutineResult::EmptyBackpack) => {
                    println!("Backpack is empty");
                    // Will need to get new trash
                    return;
                }
                Ok(RoutineResult::NewResourcesNotFound) => {
                    // wander here
                    let wander_result = self.routine_wander(world);
                    match wander_result {
                        Ok(RoutineResult::Success) => {
                            println!("Went to next quadrant");
                            return;
                        }
                        _ => {
                            println!("Error wandering");
                            return;
                        }
                    }
                }
                _ => {
                    println!("Error planning next task");
                    return;
                }
            }
        }
    }
}
