use crate::bot::{BotAction, Scrapbot, MAX_BACKPACK_ITEMS};
use pmp_collect_all::CollectAll;
use robotics_lib::interface::{destroy, robot_map, Direction};
use robotics_lib::utils::LibError;
use robotics_lib::world::tile::Content;
use robotics_lib::world::World;
use std::collections::HashMap;

impl Scrapbot {
    // deprecated
    pub fn collect_trash_in_front_of(
        &mut self,
        world: &mut World,
        direction: Direction,
    ) -> Result<usize, LibError> {
        let result = destroy(self, world, direction);
        match result {
            Ok(quantity) => {
                println!("Collected {} trash", quantity);
                Ok(quantity)
            }
            Err(err) => {
                println!("Error destroying: {:?}", err);
                Err(err)
            }
        }
    }

    // call this when you're relatively near some trash
    pub fn collect_new_trash(
        &mut self,
        world: &mut World,
        range: usize,
        quantity: usize,
    ) -> Result<usize, LibError> {
        self.full_recharge(); // because why not

        let mut requirements = HashMap::new(); // Insert all your requirements in here
        requirements.insert(Content::Garbage(0), quantity);

        let free_backpack_space = self.get_remaining_backpack_space();
        CollectAll::collect_items(self, world, range, requirements);
        // this will try to collect all the items in the requirements
        // but it's likely that it will not be able to collect all of the
        // required quantity. So we need to check how much we collected
        let new_backpack_space = self.get_remaining_backpack_space();
        match new_backpack_space < free_backpack_space {
            true => {
                self.must_find_new_trash = false;
                Ok(free_backpack_space - new_backpack_space)
            }
            false => Ok(0),
        }
    }

    pub fn collect_new_trash_fill_backpack(
        &mut self,
        world: &mut World,
    ) -> Result<usize, LibError> {
        let free_backpack_space = self.get_remaining_backpack_space();
        let portion_of_map = (robot_map(world).unwrap().len() / 100) * 15;
        self.collect_new_trash(world, portion_of_map, free_backpack_space)
    }
    pub fn lssf_search_trash(&mut self, world: &mut World) -> Result<bool, LibError> {
        let result = self.lssf_update(world, 10);

        match result {
            Ok(_) => {
                let old_lssf = self.lssf.take().unwrap();
                let trash_found = old_lssf.get_content_vec(&Content::Garbage(0));
                self.lssf = Some(old_lssf);

                match !trash_found.is_empty() {
                    true => {
                        let pev_trash_vec = self.trash_coords.take();
                        match pev_trash_vec {
                            Some(mut trash) => {
                                trash.extend(trash_found);
                                self.trash_coords = Some(trash);
                            }
                            None => {
                                self.trash_coords = Some(trash_found);
                            }
                        }

                        self.must_find_new_trash = false;
                        self.util_sort_points_from_nearest(Content::Garbage(0));
                        Ok(true)
                    }
                    false => Ok(false),
                }
            }
            Err(err) => {
                println!("Error finding garbage: {:?}", err);
                Err(err)
            }
        }
    }
    pub fn search_bins(&mut self, world: &mut World) -> Result<bool, LibError> {
        let result = self
            .lssf
            .take()
            .unwrap()
            .smart_sensing_centered(5, world, self, 1);

        match result {
            Ok(_) => {
                let bin_found = self
                    .lssf
                    .take()
                    .unwrap()
                    .get_content_vec(&Content::Bin(0..10));

                match !bin_found.is_empty() {
                    true => {
                        let pev_bin_vec = self.bin_coords.take();
                        match pev_bin_vec {
                            Some(mut bins) => {
                                bins.extend(bin_found);
                                self.bin_coords = Some(bins);
                            }
                            None => {
                                self.bin_coords = Some(bin_found);
                            }
                        }

                        self.must_find_new_bin = false;
                        self.util_sort_points_from_nearest(Content::Bin(0..10));
                        Ok(true)
                    }
                    false => Ok(false),
                }
            }
            Err(err) => {
                println!("Error finding bins: {:?}", err);
                Err(err)
            }
        }
    }

    pub fn routine_collect_trash(&mut self, world: &mut World) -> Result<(), LibError> {
        self.full_recharge();
        let backpack_threshold = MAX_BACKPACK_ITEMS / 6;
        let lssf_result = self.lssf_update(world, 10);
        match lssf_result {
            Ok(_) => {
                match self.lssf_search_trash(world) {
                    Ok(trash_found) => {
                        if trash_found {
                            // iterate over trash location, populate the action vec to
                            // reach that location and call
                            // collect_new_trash
                            let trash_coords = self.trash_coords.take().unwrap();
                            for coords in trash_coords {
                                if self.get_remaining_backpack_space() <= backpack_threshold {
                                    break; // fillato abbastanza yeah
                                }

                                // this creates the actions to do in order to reach point
                                self.populate_action_vec_given_point(coords);

                                // now I should be near some trash
                                let walk_result =
                                    self.run_action_vec_and_then(world, BotAction::Walk);
                                if walk_result.is_ok() {
                                    let collect_result =
                                        self.collect_new_trash_fill_backpack(world);
                                    return match collect_result {
                                        Ok(q) => {
                                            if q == 0 {
                                                self.must_find_new_trash = false;
                                                println!("Got no trash, sadly");
                                            }
                                            println!("Collected {} trash", q);
                                            Ok(())
                                        }
                                        Err(err) => {
                                            println!("Error collecting trash: {:?}", err);
                                            Err(err)
                                        }
                                    };
                                }
                            }
                        } else {
                            println!("No trash found");
                            return Ok(());
                        }
                    }
                    Err(err) => {
                        println!("Error searching trash: {:?}", err);
                        return Err(err);
                    }
                }
            }
            Err(err) => {
                println!("Error finding garbage: {:?}", err);
                return Err(err);
            }
        }
        Ok(())
    }
}
