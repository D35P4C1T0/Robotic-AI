use lazy_static::lazy_static;
use oxagworldgenerator::world_generator::content_options::OxAgContentOptions;
use oxagworldgenerator::world_generator::world_generator_builder::OxAgWorldGeneratorBuilder;
use oxagworldgenerator::world_generator::OxAgWorldGenerator;
use robotics_lib::event::events::Event;
use robotics_lib::runner::Runner;
use robotics_lib::world::tile::{Content, Tile};
use std::collections::HashMap;
use std::sync::Mutex;

mod bot;
mod utils;

// Static variables for data exchange between bevy and non bevy code
lazy_static! {
    // Store your variables here
    pub static ref points: Mutex<f32> = Mutex::new(0.00);
    pub static ref energy: Mutex<usize> = Mutex::new(0);
    pub static ref robot_view: Mutex<Vec<Vec<Option<Tile>>>> = Mutex::new(vec![]);
    pub static ref positions: Mutex<(usize, usize)> = Mutex::new((0, 0));
    pub static ref backpack_content: Mutex<HashMap<Content, usize>> = Mutex::new(HashMap::new());
    pub static ref events: Mutex<Vec<Event>> = Mutex::new(vec![]);
}

fn main() {
    const WORLD_SIZE: usize = 70;

    let robot = bot::Scrapbot::new();

    // World generation

    let content_vec = vec![
        (
            Content::Garbage(1),
            OxAgContentOptions {
                in_batches: true,
                is_present: true,
                min_spawn_number: 20,
                max_radius: 2,
                with_max_spawn_number: true,
                max_spawn_number: 20,
                percentage: 1f64,
            },
        ),
        (
            Content::Bin(0..1),
            OxAgContentOptions {
                in_batches: false,
                is_present: true,
                min_spawn_number: 3,
                max_radius: 0,
                with_max_spawn_number: true,
                max_spawn_number: 3,
                percentage: 1f64,
            },
        ),
    ];

    let mut generator: OxAgWorldGenerator = OxAgWorldGeneratorBuilder::new()
        .set_content_options(content_vec)
        .unwrap()
        .set_size(WORLD_SIZE)
        .set_seed(3456)
        .build()
        .unwrap();

    // World generation end

    let run = Runner::new(Box::new(robot), &mut generator);

    match run {
        Ok(mut robot) => {
            let _ = robot.game_tick();
        }
        Err(e) => println!("{:?}", e),
    }
}
