use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface::debug;
use robotics_lib::runner::{Robot, Runnable};
use robotics_lib::runner::backpack::BackPack;
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::world::tile::Content;
use robotics_lib::world::World;

use crate::utils::render_world;
use crate::utils::spytrash::get_nearby_content;

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

pub(crate) struct Thumbot(
    pub(crate) Robot, // robot
    pub(crate) ThumbotState, // state
    pub(crate) usize, // world size
    pub(crate) Vec<(usize, usize)>, // bins locations
    pub(crate) Vec<(usize, usize)>, // garbage locations
    // pub(crate) Lssf,                // lssf
);

impl Thumbot {
    pub(crate) fn new(
        robot: Robot,
        state: ThumbotState,
        world_size: usize,
        bins_locations: Vec<(usize, usize)>,
        garbage_locations: Vec<(usize, usize)>,
        // lssf: Lssf,
    ) -> Self {
        Thumbot(robot, state, world_size, bins_locations, garbage_locations)
    }

    // pub(crate) fn update_lssf_map(&mut self, world: &mut World) -> Result<(), LibError> {
    //     self.5.smart_sensing_centered(self.2, world, self, (self.2)-1)
    // }


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
        let garbage_locations = get_nearby_content(self, world, self.2, Content::Garbage(0));
        match garbage_locations {
            Ok(garbage_vec) => {
                self.4 = garbage_vec.clone();
                println!("Garbage locations: {:?}", garbage_vec);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        let bins_locations = get_nearby_content(self, world, self.2, Content::Bin(0..1));
        match bins_locations {
            Ok(bins_vec) => {
                self.3 = bins_vec.clone();
                println!("Bins locations: {:?}", bins_vec);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        // let map_update_res = self.update_lssf_map(world);
        // println!("Map update result: {:?}", map_update_res);

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
        &self.0.energy
    }
    fn get_energy_mut(&mut self) -> &mut Energy {
        &mut self.0.energy
    }
    fn get_coordinate(&self) -> &Coordinate {
        &self.0.coordinate
    }
    fn get_coordinate_mut(&mut self) -> &mut Coordinate {
        &mut self.0.coordinate
    }
    fn get_backpack(&self) -> &BackPack {
        &self.0.backpack
    }
    fn get_backpack_mut(&mut self) -> &mut BackPack {
        &mut self.0.backpack
    }
}
