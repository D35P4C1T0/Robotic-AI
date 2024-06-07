use std::collections::HashMap;

use pmp_collect_all::CollectAll;
use robotics_lib::interface::{destroy, put, robot_map, Direction};
use robotics_lib::utils::LibError;
use robotics_lib::world::tile::Content;
use robotics_lib::world::World;

use crate::bot::Scrapbot;

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

    pub fn drop_trash_into_bin_in_front_of(
        &mut self,
        world: &mut World,
        direction: Direction,
    ) -> Result<usize, LibError> {
        // call this if you have the action vector set to drop trash
        let quantity = self.get_content_quantity(&Content::Garbage(0));
        if quantity == 0 {
            return Ok(999);
            // 999 is a special value to indicate that there is no
            // trash to drop from the backpack
        }

        let content = Content::Garbage(0);
        println!("putting content of type: {:?}", content);
        match put(
            self,
            world,
            Content::Garbage(0),
            quantity,
            direction.clone(),
        ) {
            Ok(quantity) => {
                println!("trash dropped");
                Ok(quantity)
            }
            Err(err) => {
                println!("Error dropping trash: {:?}", err);
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
            true => Ok(free_backpack_space - new_backpack_space),
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
        let result = self.lssf_update(world, None);
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
    pub fn lssf_search_bins(&mut self, world: &mut World) -> Result<bool, LibError> {
        let result = self.lssf_update(world, None);
        match result {
            Ok(_) => {
                let bin_found = self
                    .lssf
                    .as_ref()
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
}
