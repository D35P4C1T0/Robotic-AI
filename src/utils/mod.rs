use robotics_lib::interface::robot_map;
use robotics_lib::runner::Runnable;
use robotics_lib::world::tile::{Content, Tile, TileType};
use robotics_lib::world::World;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

pub(crate) mod spytrash;
pub(crate) mod travel;

const TILE_SIZE: u32 = 10;
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

        for (x, row) in world_map.iter().enumerate() {
            for (y, tile) in row.iter().enumerate() {
                let rect = Rect::new(
                    (y as u32 * TILE_SIZE) as i32,
                    (x as u32 * TILE_SIZE) as i32,
                    TILE_SIZE,
                    TILE_SIZE,
                );

                let tile_color = match tile.tile_type {
                    TileType::DeepWater => Color::RGB(0, 0, 128),
                    TileType::ShallowWater => Color::RGB(0, 128, 255),
                    TileType::Sand => Color::RGB(255, 255, 128),
                    TileType::Grass => Color::RGB(34, 139, 34),
                    TileType::Street | TileType::Wall => Color::RGB(169, 169, 169),
                    TileType::Hill => Color::RGB(139, 69, 19),
                    TileType::Mountain => Color::RGB(139, 137, 137),
                    TileType::Snow => Color::RGB(255, 250, 250),
                    TileType::Lava => Color::RGB(255, 0, 0),
                    TileType::Teleport(_) => Color::RGB(255, 0, 255),
                };

                canvas.set_draw_color(tile_color);
                canvas.fill_rect(rect).unwrap();

                if let Content::Garbage(_) | Content::Bin(_) = tile.content {
                    render_content_dot(&mut canvas, rect, &tile.content);
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

pub(crate) fn nearest_border_distance(robot: &impl Runnable, world: &World) -> usize {
    let robot_pos = robot.get_coordinate();
    // Assumiamo che robot_map(world) restituisca una griglia quadrata,
    // quindi prendiamo la lunghezza di uno dei lati per calcolare world_size.

    let world_size = world_dim(world);

    let row = robot_pos.get_row();
    let col = robot_pos.get_col();

    // Calcola le distanze dai quattro bordi della mappa.
    let dist_top = row; // Distanza dal bordo superiore.
    let dist_bottom = world_size - row - 1; // Distanza dal bordo inferiore.
    let dist_left = col; // Distanza dal bordo sinistro.
    let dist_right = world_size - col - 1; // Distanza dal bordo destro.

    // Restituisce la distanza minima tra quelle calcolate.
    *[dist_top, dist_bottom, dist_left, dist_right]
        .iter()
        .min()
        .unwrap()
}

pub(crate) fn world_dim(world: &World) -> usize {
    robot_map(world)
        .expect("Failed to retrieve robot_map()")
        .len()
}

// Reborn from the ashes

pub fn valid_coords(x: i32, y: i32, size: i32) -> bool {
    !(x >= size || x < 0 || y >= size || y < 0)
}
