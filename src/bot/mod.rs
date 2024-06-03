use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface::{debug, get_score, go, put, robot_map, teleport, Direction};
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

        let square_side = if nearest_border < LARGEST_SQUARE_SIDE {
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
                self.garbage_locations = garbage_vec.clone();
                println!("Garbage locations: {:?}", garbage_vec);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        let bins_locations = get_nearby_content(self, world, world_dim(world), Content::Bin(0..1));
        match bins_locations {
            Ok(bins_vec) => {
                self.bins_locations = bins_vec.clone();
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

pub struct Scrapbot {
    pub robot: Robot,
    pub bin_coords: Option<Vec<(usize, usize)>>,
    pub trash_coords: Option<Vec<(usize, usize)>>,
    pub ticks: usize,
    pub must_empty: bool,
    pub lssf: Option<Lssf>,
    pub actions_vec: Option<Vec<Action>>,
    pub target: Content,
}

impl Scrapbot {
    pub fn new() -> Scrapbot {
        Scrapbot {
            robot: Robot::new(),
            bin_coords: None,
            trash_coords: None,
            ticks: 0,
            must_empty: false,
            lssf: Some(Lssf::new()),
            actions_vec: None,
            target: Content::Garbage(0),
        }
    }

    pub fn full_recharge(&mut self) {
        *self.get_energy_mut() = Robot::new().energy;
        self.handle_event(Event::EnergyRecharged(1000));
    }

    pub fn empty(&mut self, world: &mut World) {
        // empties the backpack of the robot inside
        // the bin, if the robot is near a bin.
        // max trash in bin: 10
        println!("EMPTY ROUTINE");
        let contents = self.robot.backpack.get_contents().clone();
        let mut iterations = 0;
        while self.get_remaining_backpack_space() < 10 && iterations < 10 {
            //println!("empty routine loop");
            for (content, quantity) in contents.iter() {
                println!("_content {:?}", content);
                if *quantity > 0usize {
                    //println!("inside empty if");
                    self.full_recharge();
                    self.drop_from_backpack((*content).clone(), world);
                }
            }
            iterations += 1;
        }
        self.must_empty = false;
    }

    pub fn drop_from_backpack(&mut self, content: Content, world: &mut World) {
        // TODO: implement drop from backpack in the direction
        println!("putting content of type: {:?}", content);

        let any_usize = 50;
        let next_content_to_reach = if content == Content::Garbage(0) {
            Content::Bin(0..any_usize)
        } else {
            Content::None
        };
        let result = self
            .lssf
            .take()
            .unwrap()
            .smart_sensing_centered(5, world, self, 1);

        match result {
            Ok(_) => {
                self.trash_coords = Some(
                    self.lssf
                        .take()
                        .unwrap()
                        .get_content_vec(&next_content_to_reach),
                );
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }

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

    // routine methods

    pub fn execute_actions(
        &mut self,
        world: &mut World,
        ends_with_put: bool,
        content: Option<Content>,
    ) {
        if self.actions_vec.is_some() {
            let last_move = if ends_with_put {
                self.actions_vec.take().and_then(|mut vec| {
                    let last = vec.pop();
                    self.actions_vec = Some(vec);
                    last
                })
            } else {
                None
            };

            // iterate over the actions and execute them
            if let Some(mut actions) = self.actions_vec.take() {
                for action in &mut actions {
                    self.full_recharge(); // because why not
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

            if let Some(content) = content {
                let quantity = self.get_content_quantity(&content);
                let put_direction: Direction = match last_move {
                    Some(Action::North) => Direction::Up,
                    Some(Action::South) => Direction::Down,
                    Some(Action::East) => Direction::Right,
                    Some(Action::West) => Direction::Left,
                    _ => Direction::Up, // hope it doesnt get here
                };

                let removed_result = put(self, world, content, quantity, put_direction);
                // handles a full bin
                match removed_result {
                    Ok(0) => {
                        // assumes the first bin is the bin I went to put the trash in
                        println!("No content removed from backpack");
                        if let Some(mut bin_coords) = self.bin_coords.take() {
                            if !bin_coords.is_empty() {
                                // remove the bin from the list
                                let bad_bin = bin_coords.remove(0);
                                println!("Removed the bin at: {}{}", bad_bin.0, bad_bin.1);
                            } else {
                                println!("The vector is empty, no element to remove.");
                            }
                            self.bin_coords = Some(bin_coords); // Put back the modified bin_coords
                        }
                    }
                    Ok(r) => println!("Content removed from backpack: {:?}", r),
                    Err(err) => println!("Error removing content from backpack: {:?}", err),
                }

                self.actions_vec.as_mut().unwrap().clear(); // clear the actions vector
            }
        }
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
                    // finiti i trash
                    self.search_bins(world);
                    if self.bin_coords.is_some() && self.bin_coords.take().unwrap().is_empty() {
                        // finiti i bin
                        is_work_done = true;
                    } else {
                        let c = self.bin_coords.clone().unwrap();
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

    pub fn move_to_coords(&mut self, coords: (usize, usize), world: &mut World) {
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
    }

    pub fn search_bins(&mut self, world: &mut World) {
        let result = self
            .lssf
            .take()
            .unwrap()
            .smart_sensing_centered(5, world, self, 1);

        match result {
            Ok(_) => {
                self.bin_coords = Some(
                    self.lssf
                        .take()
                        .unwrap()
                        .get_content_vec(&Content::Bin(0..1)),
                );
            }
            Err(err) => {
                println!("Error finding trash: {:?}", err);
            }
        }
    }

    pub fn search_trash(&mut self, world: &mut World) {
        let result = self
            .lssf
            .take()
            .unwrap()
            .smart_sensing_centered(5, world, self, 1);

        match result {
            Ok(_) => {
                self.bin_coords = Some(
                    self.lssf
                        .take()
                        .unwrap()
                        .get_content_vec(&Content::Garbage(0)),
                );
            }
            Err(err) => {
                println!("Error finding garbage: {:?}", err);
            }
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
