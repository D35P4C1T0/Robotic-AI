use robotics_lib::interface::{go, teleport, Direction};
use robotics_lib::utils::LibError;
use robotics_lib::world::World;
use sense_and_find_by_Rusafariani::{Action, Lssf};

use crate::bot::Thumbot;

pub(crate) fn update_lssf_map_smart_sensing(
    lssf: &mut Lssf,
    robot: &mut Thumbot,
    world: &mut World,
    world_size: usize,
) -> Result<(), LibError> {
    lssf.smart_sensing_centered(world_size, world, robot, world_size - 1)
}

pub(crate) fn move_robot_to_target(
    lssf: &mut Lssf,
    robot: &mut Thumbot,
    world: &mut World,
    target: (usize, usize), // row,col | y,x
) -> Result<(), LibError> {
    let shortest_path_result = lssf.get_action_vec(target.1, target.0);
    match shortest_path_result {
        Ok(vec) => {
            for action in vec {
                match action {
                    Action::North => {
                        go(robot, world, Direction::Up)?;
                    }
                    Action::South => {
                        go(robot, world, Direction::Down)?;
                    }
                    Action::West => {
                        go(robot, world, Direction::Left)?;
                    }
                    Action::East => {
                        go(robot, world, Direction::Right)?;
                    }
                    Action::Teleport(i, j) => {
                        teleport(robot, world, (i, j))?;
                    }
                }
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}
