use robotics_lib::utils::LibError;
use robotics_lib::world::tile::Content;
use robotics_lib::world::World;

use crate::robot::{BotAction, Scrapbot, MAX_BACKPACK_ITEMS};

pub(crate) enum RoutineResult {
    Success,
    FilledBackpack,
    PartiallyFilledBackpack,
    EmptyBackpack,
    NoChanges,
    NewResourcesNotFound,
    FoundFullBin,
    EmptyTrashFound,
    Wandering,
}

impl Scrapbot {
    pub(crate) fn routine_collect_trash(
        &mut self,
        world: &mut World,
    ) -> Result<RoutineResult, LibError> {
        self.full_recharge();
        self.lssf_update(world, None);

        if !self.lssf_search_trash(world)? {
            return Ok(RoutineResult::NewResourcesNotFound);
        }

        let mut trash_gathered = 0;
        let mut bad_trash_coords = vec![];
        let trash_coords = self.trash_coords.take().unwrap();

        for coords in &trash_coords {
            if self.get_remaining_backpack_space() == 0 {
                return Ok(RoutineResult::FilledBackpack);
            }

            self.populate_action_vec_given_point(world, *coords);
            self.run_action_vec_and_then(world, BotAction::Walk)?;

            match self.collect_new_trash_fill_backpack(world) {
                Ok(q) => {
                    if q == 0 {
                        bad_trash_coords.push(*coords);
                        continue;
                    }
                    trash_gathered += q;
                }
                Err(err) => {
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

        if self.get_remaining_backpack_space() < (MAX_BACKPACK_ITEMS as f32 / 6f32).floor() as usize
        {
            return Ok(RoutineResult::PartiallyFilledBackpack);
        }

        Ok(RoutineResult::Success)
    }

    pub(crate) fn routine_empty_trash(
        &mut self,
        world: &mut World,
    ) -> Result<RoutineResult, LibError> {
        self.full_recharge();
        self.lssf_update(world, None);

        if self.get_content_quantity(&Content::Garbage(0)) == 0 {
            return Ok(RoutineResult::EmptyBackpack);
        }

        if !self.lssf_search_bins(world)? {
            return Ok(RoutineResult::NewResourcesNotFound);
        }

        let bin_coords = self.bin_coords.take().unwrap_or_default();
        let mut bad_bins_vec = vec![];

        for coords in &bin_coords {
            self.populate_action_vec_given_point(world, *coords);
            self.run_action_vec_and_then(world, BotAction::Put)?;

            let last_move_direction = self.get_last_move_direction();
            if last_move_direction.is_none() {
                return Err(LibError::OperationNotAllowed);
            }

            match self.drop_trash_into_bin_in_front_of(world, last_move_direction.unwrap()) {
                Ok(0) => {
                    bad_bins_vec.push(*coords);
                }
                Ok(999) => {
                    return Ok(RoutineResult::EmptyBackpack);
                }
                Ok(q) => {
                    return Ok(RoutineResult::Success);
                }
                Err(err) => {
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

        Ok(RoutineResult::NewResourcesNotFound)
    }

    pub(crate) fn routine_reach_closest_undiscovered_tile(
        &mut self,
        world: &mut World,
    ) -> Result<RoutineResult, LibError> {
        self.full_recharge();

        if self.move_away_from_border(world) {
            self.move_to_center(world);
            return Ok(RoutineResult::Success);
        }

        self.lssf_update(world, None);
        let next_location = self.find_closest_undiscovered_tile(world);
        match next_location {
            Some(location) => {
                self.populate_action_vec_given_point(world, location);
                self.run_action_vec_and_then(world, BotAction::Walk)
                    .map(|_| RoutineResult::Success)
            }
            None => Ok(RoutineResult::NewResourcesNotFound),
        }
    }

    pub(crate) fn routine(&mut self, world: &mut World) {
        // Initialize vectors if they are not set
        self.actions_vec.get_or_insert_with(Vec::new);
        self.bin_coords.get_or_insert_with(Vec::new);
        self.trash_coords.get_or_insert_with(Vec::new);

        if let BotAction::Start = self.bot_action {
            self.bot_action = BotAction::Walk;
            self.move_away_from_border(world);
        }

        if self.get_remaining_backpack_space()
            >= (MAX_BACKPACK_ITEMS as f32 * (0.6f32)).floor() as usize
        {
            if let Ok(result) = self.routine_collect_trash(world) {
                match result {
                    RoutineResult::Success => self.handle_full_backpack(world),
                    RoutineResult::NewResourcesNotFound => self.handle_wandering(world),
                    RoutineResult::FilledBackpack => self.handle_full_backpack(world),
                    _ => {}
                }
            }
        } else {
            self.handle_full_backpack(world);
        }
    }

    fn handle_wandering(&mut self, world: &mut World) {
        if let Ok(RoutineResult::Success) = self.routine_reach_closest_undiscovered_tile(world) {}
    }

    fn handle_full_backpack(&mut self, world: &mut World) {
        match self.routine_empty_trash(world) {
            Ok(RoutineResult::Success) => self.handle_wandering(world),
            Ok(RoutineResult::EmptyBackpack) => self.handle_wandering(world),
            Ok(RoutineResult::NewResourcesNotFound) => self.handle_wandering(world),
            _ => {}
        }
    }
}
