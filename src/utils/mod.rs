pub(crate) mod spytrash;
pub(crate) mod travel;

use robotics_lib::world::tile::{Content, Tile, TileType};
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

const TILE_SIZE: u32 = 20;
const CONTENT_DOT_RADIUS: i32 = 4;

// bot: row,col
pub fn render_world(robot_position: (usize, usize), world_map: Vec<Vec<Tile>>) {
    let map_width = world_map.len();
    let map_height = if let Some(row) = world_map.first() {
        row.len()
    } else {
        0
    };

    let window_width = map_width as u32 * TILE_SIZE;
    let window_height = map_height as u32 * TILE_SIZE;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("World Map Renderer", window_width, window_height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for x in 0..map_height {
            for y in 0..map_width {
                let rect = Rect::new(
                    (y as u32 * TILE_SIZE) as i32,
                    (x as u32 * TILE_SIZE) as i32,
                    TILE_SIZE,
                    TILE_SIZE,
                );

                // Set color based on tile type
                let tile_color = match world_map[x][y].tile_type {
                    TileType::DeepWater => Color::RGB(0, 0, 128),
                    TileType::ShallowWater => Color::RGB(0, 128, 255),
                    TileType::Sand => Color::RGB(255, 255, 128),
                    TileType::Grass => Color::RGB(34, 139, 34),
                    TileType::Street => Color::RGB(169, 169, 169),
                    TileType::Hill => Color::RGB(139, 69, 19),
                    TileType::Mountain => Color::RGB(139, 137, 137),
                    TileType::Snow => Color::RGB(255, 250, 250),
                    TileType::Lava => Color::RGB(255, 0, 0),
                    TileType::Teleport(_) => Color::RGB(255, 0, 255),
                    TileType::Wall => Color::RGB(139, 69, 19),
                };

                canvas.set_draw_color(tile_color);
                canvas.fill_rect(rect).unwrap();

                // Render content dot for Garbage and Bin
                match world_map[x][y].content {
                    Content::Garbage(_) | Content::Bin(_) => {
                        render_content_dot(&mut canvas, rect, &world_map[x][y].content);
                    }
                    _ => {}
                }
            }
        }
        // Render robot position
        let (robot_y, robot_x) = robot_position;
        draw_approximate_circle(
            &mut canvas,
            robot_x as i32 * TILE_SIZE as i32 + TILE_SIZE as i32 / 2,
            robot_y as i32 * TILE_SIZE as i32 + TILE_SIZE as i32 / 2,
            Color::RGB(201, 201, 255),
        );

        canvas.present();
    }
}

fn render_content_dot(canvas: &mut Canvas<Window>, rect: Rect, content: &Content) {
    let center_x = rect.x() + (TILE_SIZE / 2) as i32;
    let center_y = rect.y() + (TILE_SIZE / 2) as i32;

    let dot_color = match content {
        Content::Garbage(_) => Color::RGB(255, 0, 0),
        Content::Bin(_) => Color::RGB(0, 0, 255),
        _ => return,
    };

    draw_approximate_circle(canvas, center_x, center_y, dot_color);
}

fn draw_approximate_circle(
    canvas: &mut Canvas<Window>,
    center_x: i32,
    center_y: i32,
    color: Color,
) {
    for i in -CONTENT_DOT_RADIUS..=CONTENT_DOT_RADIUS {
        for j in -CONTENT_DOT_RADIUS..=CONTENT_DOT_RADIUS {
            let dx = center_x + i;
            let dy = center_y + j;

            if (dx - center_x).pow(2) + (dy - center_y).pow(2) <= CONTENT_DOT_RADIUS.pow(2) {
                canvas.set_draw_color(color);
                canvas.fill_rect(Rect::new(dx, dy, 1, 1)).unwrap();
            }
        }
    }
}
