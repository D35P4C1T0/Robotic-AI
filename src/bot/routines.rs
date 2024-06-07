use robotics_lib::utils::LibError;
use robotics_lib::world::tile::Content;
use robotics_lib::world::World;

use crate::bot::{BotAction, Scrapbot, MAX_BACKPACK_ITEMS};

enum RoutineResult {
    Success,
    FilledBackpack,
    PartiallyFilledBackpack,
    EmptyBackpack,
    Failure,
    NoChanges,
    NewResourcesNotFound,
    FoundFullBin,
    EmptyTrashFound,
    Wandering,
}

impl Scrapbot {
    pub fn routine_collect_trash(&mut self, world: &mut World) -> Result<RoutineResult, LibError> {
        self.full_recharge();
        self.lssf_update(world, None)?;

        if !self.lssf_search_trash(world)? {
            println!("No trash found");
            return Ok(RoutineResult::NewResourcesNotFound);
        }

        let mut trash_gathered = 0;
        let mut bad_trash_coords = vec![];
        let trash_coords = self.trash_coords.take().unwrap();

        for coords in &trash_coords {
            if self.get_remaining_backpack_space() == 0 {
                return Ok(RoutineResult::FilledBackpack);
            }

            self.populate_action_vec_given_point(coords.clone());
            self.run_action_vec_and_then(world, BotAction::Walk)?;

            match self.collect_new_trash_fill_backpack(world) {
                Ok(q) => {
                    if q == 0 {
                        println!("Got no trash, sadly");
                        bad_trash_coords.push(coords.clone());
                        continue;
                    }
                    trash_gathered += q;
                    println!("Collected {} trash", q);
                }
                Err(err) => {
                    println!("Error collecting trash: {:?}", err);
                    return Err(err);
                }
            }
        }

        // Remove bad trash locations
        self.trash_coords = Some(
            trash_coords
                .into_iter()
                .filter(|coords| !bad_trash_coords.contains(coords))
                .collect(),
        );

        if trash_gathered == 0 {
            return Ok(RoutineResult::NoChanges);
        }

        if self.get_remaining_backpack_space() < MAX_BACKPACK_ITEMS / 6 {
            return Ok(RoutineResult::PartiallyFilledBackpack);
        }

        println!("Collected a total of {} trash", trash_gathered);
        Ok(RoutineResult::Success)
    }

    pub fn routine_empty_trash(&mut self, world: &mut World) -> Result<RoutineResult, LibError> {
        self.full_recharge();
        self.lssf_update(world, None)?;

        if self.get_content_quantity(&Content::Garbage(0)) == 0 {
            println!("No trash to drop");
            return Ok(RoutineResult::EmptyBackpack);
        }

        if !self.lssf_search_bins(world)? {
            println!("No bin found");
            return Ok(RoutineResult::NewResourcesNotFound);
        }

        let bin_coords = self.bin_coords.take().unwrap_or_default();
        let mut bad_bins_vec = vec![];

        for coords in &bin_coords {
            self.populate_action_vec_given_point(coords.clone());
            self.run_action_vec_and_then(world, BotAction::Walk)?;

            match self.drop_trash_into_bin_in_front_of(world, self.get_last_move_direction()) {
                Ok(0) => {
                    println!("Dropped no trash, sadly found bin full");
                    println!("Removing bin at {}{} from list", coords.0, coords.1);
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
            }
        }

        self.bin_coords = Some(
            bin_coords
                .into_iter()
                .filter(|coords| !bad_bins_vec.contains(coords))
                .collect(),
        );

        println!("No bin found");
        Ok(RoutineResult::NewResourcesNotFound)
    }

    // They call me the wanderer
    // Yeah, the wanderer
    // I roam around, around, around
    pub fn routine_wander(&mut self, world: &mut World) -> Result<RoutineResult, LibError> {
        // This routine is called when the bot has no more trash to collect
        self.full_recharge();
        let new_location = self.next_quadrant_clockwise(world);

        self.populate_action_vec_given_point(new_location);
        self.run_action_vec_and_then(world, BotAction::Walk)
            .map(|_| RoutineResult::Success)
            .map_err(|err| {
                println!("Error wandering: {:?}", err);
                err
            })
    }

    // TODO: rework routine method
    pub fn routine(&mut self, world: &mut World) {
        if self.quadrants_visited.values().all(|&v| v) {
            println!("All quadrants visited, stopping");
            return;
        }

        // Initialize vectors if they are not set
        self.actions_vec.get_or_insert_with(Vec::new);
        self.bin_coords.get_or_insert_with(Vec::new);
        self.trash_coords.get_or_insert_with(Vec::new);

        if self.get_remaining_backpack_space() >= MAX_BACKPACK_ITEMS * (3 / 5) {
            if let Ok(result) = self.routine_collect_trash(world) {
                match result {
                    RoutineResult::Success => println!("Trash collected"),
                    RoutineResult::NewResourcesNotFound => self.handle_wandering(world),
                    RoutineResult::FilledBackpack => self.handle_full_backpack(world),
                    _ => println!("Error planning next task"),
                }
            } else {
                println!("Error planning next task");
            }
        } else if self.get_remaining_backpack_space() == 0 {
            println!("Backpack is full");
            self.handle_full_backpack(world);
        }
    }

    fn handle_wandering(&mut self, world: &mut World) {
        match self.routine_wander(world) {
            Ok(RoutineResult::Success) => println!("Went to next quadrant"),
            _ => println!("Error wandering"),
        }
    }

    fn handle_full_backpack(&mut self, world: &mut World) {
        match self.routine_empty_trash(world) {
            Ok(RoutineResult::Success) => println!("Trash dropped"),
            Ok(RoutineResult::EmptyBackpack) => println!("Backpack is empty"),
            Ok(RoutineResult::NewResourcesNotFound) => self.handle_wandering(world),
            _ => println!("Error planning next task"),
        }
    }
}
