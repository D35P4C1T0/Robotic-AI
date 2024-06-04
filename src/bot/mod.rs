use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface::{debug, destroy, get_score, go, put, robot_map, teleport, Direction};
use robotics_lib::runner::backpack::BackPack;
use robotics_lib::runner::{Robot, Runnable};
use robotics_lib::utils::LibError;
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::world::tile::Content;
use robotics_lib::world::tile::Content::Garbage;
use robotics_lib::world::World;
use sense_and_find_by_rustafariani::{Action, Lssf};
use spyglass::spyglass::Spyglass;

use crate::utils::spytrash::get_nearby_content;
use crate::utils::{nearest_border_distance, render_world, world_dim};
use crate::{backpack_content, energy, events, points, positions, robot_view};

mod print;

// Each bin can handle max 10 of garbage.

#[derive(Debug)]
pub(crate) enum ThumbotState {
    Start,
    SearchingTrash,
    SearchingBin,
    GotTrash,
    FoundBin,
    Done,
}

pub(crate) struct Thumbot {
    pub(crate) robot: Robot,
    // robot
    pub(crate) state: ThumbotState,
    // stato del Thumbot
    pub(crate) bins_locations: Vec<(usize, usize)>,
    // posizioni dei bidoni
    pub(crate) garbage_locations: Vec<(usize, usize)>,
    // posizioni dei rifiuti
    // pub(crate) lssf: RefCell<Lssf>,
    pub(crate) lssf: Option<Lssf>,
}

impl Thumbot {
    pub(crate) fn new() -> Self {
        Thumbot {
            robot: Robot::new(),
            state: ThumbotState::Start,
            bins_locations: vec![],
            garbage_locations: vec![],
            lssf: Some(Lssf::new()),
        }
    }

    pub(crate) fn update_lssf_map(&mut self, world: &mut World) -> Result<(), LibError> {
        const LARGEST_SQUARE_SIDE: usize = 69;
        let nearest_border = nearest_border_distance(self, world);

        let _square_side = if nearest_border < LARGEST_SQUARE_SIDE {
            nearest_border
        } else {
            LARGEST_SQUARE_SIDE
        };

        println!("bot location: {:?}", self.get_coordinate());

        let mut temp_lssf = self.lssf.take().unwrap();
        let result = temp_lssf.smart_sensing_centered(5, world, self, 1);
        self.lssf = Some(temp_lssf);
        result

        // let mut temp_lssf = Lssf::new();
        // mem::swap(&mut temp_lssf, &mut *self.lssf.borrow_mut());
        // let result = temp_lssf.smart_sensing_centered(square_side, world, self, 50);
        // mem::swap(&mut temp_lssf, &mut *self.lssf.borrow_mut());
        // result
    }

    // pub(crate) fn new() -> Self {
    //     Thumbot(Robot::new(), ThumbotState::Start,)
    // }

    // fn state_machine_next_state(&mut self, world: &mut World) {
    //     match self.1 {
    //         ThumbotState::Start => {
    //             // do the big scan for trash and bins
    //             // and remember the locations
    //
    //             // Transition to the next state
    //             self.1 = ThumbotState::SearchingTrash;
    //         }
    //         ThumbotState::SearchingTrash => {
    //             // backpack not full, search for trash
    //             // if backpack full, search for bin
    //
    //             if self.get_backpack() {
    //                 self.1 = ThumbotState::SearchingBin;
    //             } else {
    //                 self.1 = ThumbotState::GotTrash;
    //             }
    //         }
    //         ThumbotState::SearchingBin => {
    //             // search for bin
    //             // if bin found, go to it
    //             // once reached, change state to GotTrash
    //
    //             if let Some(bin_location) = self.find_nearest_bin() {
    //                 self.go_to_location(bin_location);
    //                 self.1 = ThumbotState::GotTrash;
    //             }
    //         }
    //         ThumbotState::GotTrash => {
    //             // collect trash,
    //             // if backpack full, search for bin
    //
    //             self.collect_trash();
    //
    //             if self.backpack_full() {
    //                 self.1 = ThumbotState::SearchingBin;
    //             }
    //         }
    //         ThumbotState::FoundBin => {
    //             // dump trash until bin full or backpack empty
    //             // if backpack empty, search for trash
    //
    //             self.dump_trash();
    //
    //             if self.backpack_empty() {
    //                 self.1 = ThumbotState::SearchingTrash;
    //             }
    //         }
    //         _ => {}
    //     }
    // }
}

impl Runnable for Thumbot {
    fn process_tick(&mut self, world: &mut World) {
        let garbage_locations =
            get_nearby_content(self, world, world_dim(world), Content::Garbage(0));
        match garbage_locations {
            Ok(garbage_vec) => {
                self.garbage_locations.clone_from(&garbage_vec);
                println!("Garbage locations: {:?}", garbage_vec);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        let bins_locations = get_nearby_content(self, world, world_dim(world), Content::Bin(0..1));
        match bins_locations {
            Ok(bins_vec) => {
                self.bins_locations.clone_from(&bins_vec);
                println!("Bins locations: {:?}", bins_vec);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        println!("nearest border: {}", nearest_border_distance(self, world));

        let map_update_res = self.update_lssf_map(world); // index out of bounds
                                                          // panic!("Map update result: {:?}", map_update_res);
        println!("Map update result: {:?}", map_update_res);

        while self.get_energy().get_energy_level() > 0 {
            let debug_view = debug(self, world);
            let map = debug_view.0;
            let robot_pos = debug_view.2;
            render_world(robot_pos, map.clone());

            // let robot_view = one_direction_view(self, world, Direction::Left, 6);
            // println!("Robot view: {:?}", robot_view);

            // if destroy(self, world, Direction::Right).is_ok() {
            //     println!("Destroyed something");
            //     println!("{:?}", self.get_backpack());
            // } else {
            //     println!("Nothing to destroy");
            // }

            println!("Energy level: {}", self.get_energy().get_energy_level());
            println!("bot pos: {:?}", self.get_coordinate());
        }
    }

    fn handle_event(&mut self, _event: Event) {
        // react to this event in your GUI
    }
    fn get_energy(&self) -> &Energy {
        &self.robot.energy
    }
    fn get_energy_mut(&mut self) -> &mut Energy {
        &mut self.robot.energy
    }
    fn get_coordinate(&self) -> &Coordinate {
        &self.robot.coordinate
    }
    fn get_coordinate_mut(&mut self) -> &mut Coordinate {
        &mut self.robot.coordinate
    }
    fn get_backpack(&self) -> &BackPack {
        &self.robot.backpack
    }
    fn get_backpack_mut(&mut self) -> &mut BackPack {
        &mut self.robot.backpack
    }
}

//noinspection ALL
impl Default for Thumbot {
    fn default() -> Self {
        Thumbot::new()
    }
}

// Reborn from the ashes

pub enum BotAction {
    Destroy,
    Put,
    Start,
}

pub struct Scrapbot {
    pub robot: Robot,
    pub bin_coords: Option<Vec<(usize, usize)>>,
    pub trash_coords: Option<Vec<(usize, usize)>>,
    pub ticks: usize,
    pub must_empty: bool,
    pub must_find_new_trash: bool,
    pub must_find_new_bin: bool,
    pub lssf: Option<Lssf>,
    pub actions_vec: Option<Vec<Action>>,
    pub target: Content,
    pub bot_action: BotAction,
}

impl Scrapbot {
    pub fn new() -> Scrapbot {
        Scrapbot {
            robot: Robot::new(),
            bin_coords: None,
            trash_coords: None,
            ticks: 0,
            must_empty: false,
            must_find_new_trash: true,
            must_find_new_bin: true,
            lssf: Some(Lssf::new()),
            actions_vec: None,
            target: Content::Garbage(0),
            bot_action: BotAction::Start,
        }
    }

    // backpack methods
    pub fn get_remaining_backpack_space(&mut self) -> usize {
        let backpack_size = self.robot.backpack.get_size();
        let mut space_left = backpack_size;
        let backpack = self.robot.backpack.get_contents();

        for (_, quantity) in backpack.iter() {
            space_left -= quantity;
        }

        if space_left < backpack_size / 5 {
            self.must_empty = true;
        }

        space_left
    }

    pub fn get_content_quantity(&mut self, content: &Content) -> usize {
        *self.robot.backpack.get_contents().get(content).unwrap()
    }

    // energy
    pub fn full_recharge(&mut self) {
        *self.get_energy_mut() = Robot::new().energy;
        self.handle_event(Event::EnergyRecharged(1000));
    }

    pub fn collect_near_trash(
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

    // bin methods
    pub fn drop_trash_into_bin(
        &mut self,
        world: &mut World,
        direction: Direction,
        quantity: usize,
    ) -> Result<usize, LibError> {
        // call this if you have the action vector set to drop trash
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

    // routine methods
    pub fn plan_next_moves_given_coord(&mut self, coordinate: Coordinate) {
        match self
            .lssf
            .take()
            .unwrap()
            .get_action_vec(coordinate.get_col(), coordinate.get_row())
        {
            Ok(actions) => {
                self.actions_vec = Some(actions);
            }
            Err(err) => {
                println!("Error planning next move: {:?}", err);
            }
        }
    }

    pub fn execute_actions(
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
                    return match self.collect_near_trash(world, last_move_direction.clone()) {
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
            }
            self.actions_vec.as_mut().unwrap().clear(); // clear the actions vector
        }
        Ok(0)
    }

    pub fn work_done(&mut self, world: &mut World) -> bool {
        let mut is_work_done = false;
        //number of unexplored tiles
        let mut none_num = 0;
        let threshold = 0.20;
        if let Some(known_map) = robot_map(world) {
            let size = known_map.len();

            known_map.iter().for_each(|row| {
                row.iter().for_each(|tile| {
                    if tile.is_none() {
                        none_num += 1;
                    }
                });
            });

            if (none_num as f64) / ((size * size) as f64) < threshold {
                //checks if there are still trash in the world if not it returns that the
                //job of the robot is done

                let search_result = self
                    .lssf
                    .take()
                    .unwrap()
                    .smart_sensing_centered(10, world, self, 1);

                match search_result {
                    Ok(_) => {
                        let next_trash_location =
                            self.lssf.take().unwrap().get_content_vec(&Garbage(0));
                        match !next_trash_location.is_empty() {
                            true => {
                                is_work_done = false;
                            }
                            false => {
                                is_work_done = true;
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error finding garbage: {:?}", err);
                    }
                }

                if self.trash_coords.is_none() {
                    // prima inizializzazione
                    self.trash_coords =
                        Some(self.lssf.take().unwrap().get_content_vec(&Garbage(0)));
                } else if self.trash_coords.take().unwrap().is_empty() {
                    // trash_points esauriti
                    self.must_find_new_trash = true;
                    if self.bin_coords.is_some() && self.bin_coords.take().unwrap().is_empty() {
                        // finiti i bin
                        is_work_done = true;
                    } else {
                        let _c = self.bin_coords.clone().unwrap();
                        // println!("bin cord now: {:?}", known_map[c.0][c.1].clone().unwrap());
                    }
                }
            }
            //println!("percentuale di mondo non scoperta: {}", (none_num as f64) / ((size*size) as f64))
        }
        is_work_done
    }

    // map exploration methods
    pub fn spyglass_explore(&mut self, world: &mut World) {
        //println!("spyglass exploration");
        let map = robot_map(world).unwrap();
        let map_size = map.len();
        let distance = if map_size < 64 { map_size / 4 } else { 30 };
        let mut spy_glass = Spyglass::new(
            self.get_coordinate().get_row(),
            self.get_coordinate().get_col(),
            distance,
            map_size,
            Some(self.get_energy().get_energy_level()),
            true,
            1.0,
            |_| false,
        );
        spy_glass.new_discover(self, world);
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

    pub fn search_trash(&mut self, world: &mut World) -> Result<bool, LibError> {
        let result = self
            .lssf
            .take()
            .unwrap()
            .smart_sensing_centered(6, world, self, 1);

        match result {
            Ok(_) => {
                let trash_found = self
                    .lssf
                    .take()
                    .unwrap()
                    .get_content_vec(&Content::Garbage(0));

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

    pub fn util_sort_points_from_nearest(&mut self, content: Content) {
        // Take the coordinates vector to be ordered based on the content type
        let mut coords_vec_to_be_ordered = if content == Content::Garbage(0) {
            self.trash_coords.take()
        } else {
            self.bin_coords.take()
        };

        // If the vector is not empty, sort it by distance from the robot's current position
        if let Some(coords_vec) = &mut coords_vec_to_be_ordered {
            coords_vec.sort_by_key(|(row, col)| {
                // Calculate the absolute differences in row and column
                let row_diff = (*row as i32 - self.get_coordinate().get_row() as i32).abs();
                let col_diff = (*col as i32 - self.get_coordinate().get_col() as i32).abs();
                // Sort by the sum of the absolute differences
                row_diff + col_diff
            });
        }

        // Put back the ordered coordinates vector based on the content type
        if content == Content::Garbage(0) {
            self.trash_coords = coords_vec_to_be_ordered;
        } else {
            self.bin_coords = coords_vec_to_be_ordered;
        }
    }

    pub fn plan(&mut self, world: &mut World) {
        //println!("planning");
        if self.must_find_new_trash {
            //println!("searching trash");
            match self.search_trash(world) {
                Ok(result) => {
                    match result {
                        true => {
                            self.must_find_new_trash = false;
                        }
                        false => {
                            self.must_find_new_trash = true;
                            self.spyglass_explore(world);
                            // TODO not the right tool but ok
                        }
                    }
                }
                Err(err) => {
                    println!("Error finding trash: {:?}", err);
                }
            }
        }

        if self.must_find_new_bin {
            //println!("searching bin");
            match self.search_bins(world) {
                Ok(result) => {
                    match result {
                        true => {
                            self.must_find_new_bin = false;
                        }
                        false => {
                            self.must_find_new_bin = true;
                            self.spyglass_explore(world);
                            // TODO not the right tool but ok
                        }
                    }
                }
                Err(err) => {
                    println!("Error finding bins: {:?}", err);
                }
            }
        }
    }

    // TODO: rework routine method
    pub fn routine(&mut self, world: &mut World) {
        //println!("routine");
        // if self.work_done(world) {
        //     self.must_empty = true;
        // }

        if self.must_empty {
            //println!("emptying");
            if self.get_content_quantity(&Content::Garbage(0)) > 0 {
                //println!("emptying trash");
                let result = self.execute_actions(world, BotAction::Put);
                match result {
                    Ok(_) => {
                        self.must_empty = false;
                    }
                    Err(err) => {
                        println!("Error emptying trash: {:?}", err);
                    }
                }
            }
            // else {
            //     //println!("emptying bin");
            //     let result = self.execute_actions(world, BotAction::Destroy);
            //     match result {
            //         Ok(_) => {
            //             self.must_empty = false;
            //         }
            //         Err(err) => {
            //             println!("Error emptying bin: {:?}", err);
            //         }
            //     }
            // }
        } else {
            //println!("planning");
            self.plan(world);
            //println!("planning done");
            //println!("executing");
            // let result = self.execute_actions(world, false, false);
            // match result {
            //     Ok(_) => {
            //         //println!("executing done");
            //     }
            //     Err(err) => {
            //         println!("Error executing actions: {:?}", err);
            //     }
            // }
        }
    }
}

impl Runnable for Scrapbot {
    fn process_tick(&mut self, world: &mut World) {
        // self.routine(world);

        let mut update_points = points.lock().unwrap();
        let mut update_robot_view = robot_view.lock().unwrap();
        let mut update_positions = positions.lock().unwrap();
        let mut update_energy = energy.lock().unwrap();
        let mut update_backpack_content = backpack_content.lock().unwrap();

        *update_positions = (
            self.robot.coordinate.get_row(),
            self.robot.coordinate.get_col(),
        );
        *update_points = get_score(world);
        *update_robot_view = robot_map(world).unwrap();
        *update_energy = self.robot.energy.get_energy_level();
        update_backpack_content.clone_from(self.get_backpack().get_contents()); // was clone before
    }
    fn handle_event(&mut self, event: Event) {
        let mut update_events = events.lock().unwrap();
        update_events.push(event.clone());
    }
    fn get_energy(&self) -> &Energy {
        &self.robot.energy
    }
    fn get_energy_mut(&mut self) -> &mut Energy {
        &mut self.robot.energy
    }
    fn get_coordinate(&self) -> &Coordinate {
        &self.robot.coordinate
    }
    fn get_coordinate_mut(&mut self) -> &mut Coordinate {
        &mut self.robot.coordinate
    }
    fn get_backpack(&self) -> &BackPack {
        &self.robot.backpack
    }
    fn get_backpack_mut(&mut self) -> &mut BackPack {
        &mut self.robot.backpack
    }
}
