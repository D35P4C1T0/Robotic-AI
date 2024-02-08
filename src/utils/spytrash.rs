// ### SPYGLASS EXAMPLE ###
// let mut spyglass_garbage = Spyglass::new_default(
//     robot.get_coordinate().get_row(), // center row
//     robot.get_coordinate().get_col(), // center col
//     50, // distance100 //
//     robot.2, // world size
// );

use robotics_lib::runner::Runnable;
use robotics_lib::world::tile::{Content, Tile};
use robotics_lib::world::World;
use spyglass::spyglass::{Spyglass, SpyglassResult};

pub(crate) fn get_nearby_content(
    robot: &mut impl Runnable,
    world: &mut World,
    world_size: usize,
    content: Content,
) -> Result<Vec<(usize, usize)>, String> {
    let closure = match content {
        Content::Garbage(_) => |tile: &Tile| matches!(tile.clone().content, Content::Garbage(_)),
        Content::Bin(_) => |tile: &Tile| matches!(tile.clone().content, Content::Bin(_)),
        _ => return Err("Invalid content type".to_string()),
    };

    let mut spyglass = Spyglass::new(
        robot.get_coordinate().get_row(),
        robot.get_coordinate().get_col(),
        world_size,
        world_size,
        Some(500),
        true,
        (world_size / 2) as f64,
        closure,
    );

    let mut content_locations: Vec<(usize, usize)> = vec![];
    match spyglass.new_discover(robot, world) {
        SpyglassResult::Stopped(v) => {
            for (_tile, row, col) in v {
                // println!("{} found at: {}, {}", tile.content, row, col);
                content_locations.push((row, col));
            }
        }
        SpyglassResult::Complete => {
            println!("Spyglass complete");
        }
        _ => {
            return Err("Spyglass failed".to_string());
        }
    }

    Ok(content_locations)
}
