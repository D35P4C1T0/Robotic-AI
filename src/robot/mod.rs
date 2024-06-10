use std::cmp::{max, min};
use std::collections::HashMap;

use oxagaudiotool::OxAgAudioTool;
use robot_for_visualizer::RobotForVisualizer;
use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface::robot_map;
use robotics_lib::runner::backpack::BackPack;
use robotics_lib::runner::{Robot, Runnable, Runner};
use robotics_lib::utils::LibError;
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::world::tile::Content;
use robotics_lib::world::tile::Content::Garbage;
use robotics_lib::world::world_generator::Generator;
use robotics_lib::world::World;
use sense_and_find_by_rustafariani::{Action, Lssf};
use spyglass::spyglass::Spyglass;

use crate::robot::sound::{populate_sounds, populate_sounds_given_path};

mod movement;
mod print;
mod routines;
mod sound;
mod trash_collection;

// Each bin can handle max 10 of garbage.
// Reborn from the ashes

const MAX_BACKPACK_ITEMS: usize = 20;

pub enum BotAction {
    Put,
    Destroy,
    Start,
    Walk,
}

pub struct Scrapbot {
    pub robot: Robot,
    pub audio: Option<OxAgAudioTool>,
    pub bin_coords: Option<Vec<(usize, usize)>>,
    pub trash_coords: Option<Vec<(usize, usize)>>,
    pub lssf: Option<Lssf>,
    pub actions_vec: Option<Vec<Action>>,
    pub bot_action: BotAction,
    pub search_radius: Option<usize>,
    pub quadrants_visited: HashMap<usize, bool>,
}

impl Default for Scrapbot {
    fn default() -> Self {
        Self::new()
    }
}

impl Scrapbot {
    pub fn new() -> Scrapbot {
        Scrapbot {
            robot: Robot::new(),
            audio: Some(
                OxAgAudioTool::new(populate_sounds(), HashMap::new(), HashMap::new()).unwrap(),
            ),
            bin_coords: None,
            trash_coords: None,
            lssf: Some(Lssf::new()),
            actions_vec: None,
            bot_action: BotAction::Start,
            search_radius: None,
            quadrants_visited: HashMap::from([
                (1usize, false),
                (2usize, false),
                (3usize, false),
                (4usize, false),
            ]),
        }
    }

    // backpack methods
    pub fn get_remaining_backpack_space(&mut self) -> usize {
        let used_space: usize = self.robot.backpack.get_contents().values().sum();
        MAX_BACKPACK_ITEMS - used_space
    }

    pub fn get_content_quantity(&mut self, content: &Content) -> usize {
        *self.robot.backpack.get_contents().get(content).unwrap()
    }

    // energy
    pub fn full_recharge(&mut self) {
        *self.get_energy_mut() = Robot::new().energy;
        self.handle_event(Event::EnergyRecharged(1000));
    }

    fn round_down_to_nearest_odd(value: usize) -> usize {
        if value % 2 == 0 {
            max(value.saturating_sub(1), 3)
        } else {
            value
        }
    }

    // unused, yet
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
                self.lssf_update(world, None);

                let old_lssf = self.lssf.take().unwrap();
                let next_trash_location = old_lssf.get_content_vec(&Garbage(0));
                match !next_trash_location.is_empty() {
                    true => {
                        is_work_done = false;
                    }
                    false => {
                        is_work_done = true;
                    }
                }
                self.lssf = Some(old_lssf);

                if self.trash_coords.is_none() {
                    // prima inizializzazione
                    let old_lssf = self.lssf.take().unwrap();
                    self.trash_coords = Some(old_lssf.get_content_vec(&Garbage(0)));
                    self.lssf = Some(old_lssf);
                } else if self.trash_coords.as_ref().unwrap().is_empty() {
                    // trash_points esauriti
                    // self.must_find_new_trash = true;
                    if self.bin_coords.is_some() && self.bin_coords.as_ref().unwrap().is_empty() {
                        // finiti i bin
                        is_work_done = true;
                    }
                }
            }
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

    pub fn lssf_update(&mut self, world: &mut World, input_radius: Option<usize>) {
        self.full_recharge();
        // Use the specified radius if provided, otherwise use default (1/8 of map size)
        // or the nearest border distance so that the tool doesn't shit itself

        let world_dim = robot_map(world).unwrap().len();
        let mut scan_diameter = input_radius.unwrap_or(world_dim / 4);

        print!("proposed scan diameter {} | ", scan_diameter);
        scan_diameter = min(
            Self::round_down_to_nearest_odd(scan_diameter),
            Self::round_down_to_nearest_odd(self.nearest_border_distance(world) * 2),
        );

        // println!("nearest border: {}", self.nearest_border_distance(world));
        println!("scan diameter: {}", scan_diameter);

        // Update LSSF
        let mut lssf = self.lssf.take().unwrap();
        lssf.smart_sensing_centered(scan_diameter, world, self, 0)
            .ok();
        // self.spyglass_explore(world);
        // lssf.update_map(&robot_map(world).unwrap());

        self.lssf = Some(lssf);

        // Return result
        // match result {
        //     Ok(_) => Ok(()),
        //     Err(err) => {
        //         println!("Error updating LSSF: {:?}", err);
        //         Err(err)
        //     }
        // }
    }

    pub fn use_discovery_tools(&mut self, world: &mut World) {
        // to be used when the robot is stuck or at first start
        self.full_recharge();
        self.spyglass_explore(world);
    }

    pub fn util_sort_points_from_nearest(&mut self, content: Content) {
        // Take the coordinates vector to be ordered based on the content type
        let mut coords_vec_to_be_ordered = if content == Garbage(0) {
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
        if content == Garbage(0) {
            self.trash_coords = coords_vec_to_be_ordered;
        } else {
            self.bin_coords = coords_vec_to_be_ordered;
        }
    }

    pub(crate) fn valid_lssf_coords(&self, col: usize, row: usize) -> bool {
        let action_vec = self.lssf.as_ref().unwrap().get_action_vec(col, row);
        match action_vec {
            Ok(actions) => !actions.is_empty(),
            Err(_) => false,
        }
    }
}

impl Runnable for Scrapbot {
    fn process_tick(&mut self, world: &mut World) {
        self.routine(world);

        self.store_environmental_condition(world);
        self.store_tiles(world);
    }
    fn handle_event(&mut self, event: Event) {
        // let mut update_events = events.lock().unwrap();
        // update_events.push(event.clone());
        self.store_event(event);
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

impl RobotForVisualizer for Scrapbot {
    fn get_runner(generator: &mut impl Generator) -> Result<Runner, LibError> {
        Runner::new(Box::new(Scrapbot::new()), generator)
    }
    fn set_audio_path(path: String) {
        populate_sounds_given_path(path);
    }
}
