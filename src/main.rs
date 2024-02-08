use oxagworldgenerator::world_generator::content_options::OxAgContentOptions;
use oxagworldgenerator::world_generator::world_generator_builder::OxAgWorldGeneratorBuilder;
use oxagworldgenerator::world_generator::OxAgWorldGenerator;
use robotics_lib::runner::{Robot, Runner};
use robotics_lib::world::tile::Content;
use sense_and_find_by_Rusafariani::Lssf;

use crate::bot::ThumbotState;

mod bot;
mod utils;

fn main() {
    const WORLD_SIZE: usize = 50;

    let lssf = Lssf::new();
    let robot = bot::Thumbot::new(
        Robot::default(),  // Robot
        ThumbotState::Start, // ThumbotState
        WORLD_SIZE,          // world size
        vec![],              // bins locations
        vec![],              // garbage locations
    );



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
