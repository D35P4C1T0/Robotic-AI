use colored::Colorize;

use robotics_lib::world::tile::Content::Scarecrow;
use robotics_lib::world::tile::{Content, Tile, TileType};

pub fn print_world(map: &[Vec<Option<Tile>>], robot_coords: (usize, usize)) {
    for (i, row) in map.iter().enumerate() {
        for (j, tile) in row.iter().enumerate() {
            if (i, j) == robot_coords {
                print!("{}", "R ".bright_magenta());
            } else {
                match tile {
                    None => print!("? "),
                    Some(t) => match &t.content {
                        Content::Tree(_) => print!("T "),
                        Content::Coin(_) => print!("{}", "C ".bright_cyan()),
                        Content::Bank(_) | Content::Garbage(_) => {
                            print!("{}", "B ".bright_purple())
                        }
                        _ => match t.tile_type {
                            TileType::DeepWater => print!("{}", "D ".bright_blue()),
                            TileType::ShallowWater => print!("{}", "W ".blue()),
                            TileType::Lava => print!("{}", "L ".bright_red()),
                            _ => print!("{}", "O ".bright_green()),
                        },
                    },
                }
            }
        }
        println!();
    }
}

pub fn print_debug(map: Vec<Vec<Tile>>, size: usize) {
    for i in 0..size {
        for j in 0..size {
            let t = &map[i][j];
            tile_matching(t);
        }
        println!();
    }
}

fn tile_matching(t: &Tile) {
    match &t.content {
        Content::Rock(_) => print!("🪨"),
        Content::Tree(_) => print!("🌳"),
        Content::Garbage(_) => print!("🛢️"),
        Content::Fire => print!("🔥"),
        Content::Coin(_) => print!("🪙"),
        Content::Bin(_) => print!("🗑️"),
        Content::Crate(_) => print!("🚪"),
        Content::Bank(_) => print!("🏦"),
        Content::Water(_) => print!("🌊"),
        Content::Market(_) => print!("💸"),
        Content::Fish(_) => print!("🐟"),
        Content::Building => print!("🏠"),
        Content::Bush(_) => print!("🥦"),
        Content::JollyBlock(_) => print!("🍄"),
        Scarecrow => print!("🐔"),
        Content::None => match t.tile_type {
            TileType::DeepWater => print!("🔹"),
            TileType::ShallowWater => print!("🟦"),
            TileType::Sand => print!("🟨"),
            TileType::Grass => print!("🟩"),
            TileType::Street => print!("🛣"),
            TileType::Hill => print!("🌸"),
            TileType::Mountain => print!("🟫"),
            TileType::Snow => print!("⬜"),
            TileType::Lava => print!("🟥"),
            TileType::Teleport(_) => print!("🟪"),
            TileType::Wall => print!("🧱"),
        },
    }
}
