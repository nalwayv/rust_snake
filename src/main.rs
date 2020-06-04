//! RUST SNAKE

// IMPORTS
use sfml::{graphics::*, system::*, window::*};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};

// -----------------------------------
// CONSTS
// -----------------------------------
const BLOCK_SIZE: f32 = 25.0;
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

// -----------------------------------
// ENUMS
// -----------------------------------
#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq, Clone)]
enum TileType {
    Blocked,
    NonBlocked,
    Active,
    NonActive,
}

// -----------------------------------
// HEAD
// -----------------------------------
#[allow(dead_code)]
struct Head<'a> {
    position: Vector2f,
    scale: f32,
    is_active: bool,
    dir: Direction,
    rect_shape: RectangleShape<'a>,
}

impl<'a> Head<'a> {
    fn new(x: f32, y: f32, scale: f32, color: Color, dir: Direction) -> Self {
        let mut r = RectangleShape::new();
        r.set_size((scale, scale));
        r.set_fill_color(color);
        r.set_position((x, y));
        r.set_origin((0.0, 0.0));

        Self {
            position: Vector2f::new(x, y),
            scale: scale,
            is_active: true,
            dir: dir,
            rect_shape: r,
        }
    }

    fn reset(&mut self, x: f32, y: f32) {
        self.set_pos(x, y);
        self.set_direction(Direction::Right);
    }

    fn set_pos(&mut self, x: f32, y: f32) {
        self.rect_shape.set_position((x, y));
    }

    fn set_direction(&mut self, new_dir: Direction) {
        self.dir = new_dir;
    }

    /// get x screen position
    fn get_x(&self) -> f32 {
        self.rect_shape.position().x
    }

    /// get y screen position
    fn get_y(&self) -> f32 {
        self.rect_shape.position().y
    }

    fn draw(&mut self, win: &mut RenderWindow) {
        if !self.is_active {
            return;
        }

        win.draw(&self.rect_shape);
    }

    fn inputs(&mut self, input_map: &HashMap<&Key, bool>) {
        if !self.is_active {
            return;
        }

        if input_map[&Key::W] && self.dir != Direction::Down {
            self.dir = Direction::Up;
            return;
        }

        if input_map[&Key::S] && self.dir != Direction::Up {
            self.dir = Direction::Down;
            return;
        }

        if input_map[&Key::A] && self.dir != Direction::Right {
            self.dir = Direction::Left;
            return;
        }

        if input_map[&Key::D] && self.dir != Direction::Left {
            self.dir = Direction::Right;
            return;
        }
    }

    fn update(&mut self) {
        if !self.is_active {
            return;
        }

        let new_dir = match self.dir {
            Direction::Up => Vector2f::new(0.0, -1.0),
            Direction::Down => Vector2f::new(0.0, 1.0),
            Direction::Left => Vector2f::new(-1.0, 0.0),
            Direction::Right => Vector2f::new(1.0, 0.0),
        };

        self.rect_shape.move_(new_dir * self.scale);
    }
}

// -----------------------------------
// TAIL
// -----------------------------------
#[allow(dead_code)]
struct Tail<'a> {
    position: Vector2f,
    scale: f32,
    is_active: bool,
    rect_shape: RectangleShape<'a>,
}

impl<'a> Tail<'a> {
    fn new(x: f32, y: f32, scale: f32, color: Color) -> Self {
        let mut r = RectangleShape::new();
        r.set_size((scale, scale));
        r.set_fill_color(color);
        r.set_position((x, y));
        r.set_origin((0.0, 0.0));

        Self {
            position: Vector2f::new(x, y),
            scale: scale,
            is_active: true,
            rect_shape: r,
        }
    }

    /// get x screen position
    fn get_x(&self) -> f32 {
        self.rect_shape.position().x
    }

    /// get y screen position
    fn get_y(&self) -> f32 {
        self.rect_shape.position().y
    }

    fn draw(&mut self, win: &mut RenderWindow) {
        if !self.is_active {
            return;
        }

        win.draw(&self.rect_shape);
    }

    fn update(&mut self, px: f32, py: f32) {
        if !self.is_active {
            return;
        }
        // get direction to take
        // rect in front previous screen position - this rect's current screen position
        // (-1, 0) or (1, 1) ...
        let nd = Vector2f::new(px, py) - self.rect_shape.position();

        self.rect_shape.move_(nd);
    }
}

// -----------------------------------
// TILE
// -----------------------------------
#[derive(Clone)]
struct Tile<'a> {
    rect: RectangleShape<'a>,
    scale: f32,
    tile_type: TileType,
}

impl<'a> Tile<'a> {
    fn new(scale: f32, tile: TileType) -> Self {
        Self {
            rect: RectangleShape::new(),
            scale: scale,
            tile_type: tile,
        }
    }

    fn draw_tile(&mut self, x: f32, y: f32, win: &mut RenderWindow) {
        let pos_x = x * self.scale;
        let pos_y = y * self.scale;

        self.rect.set_position((pos_x, pos_y));
        self.rect.set_size((self.scale, self.scale));
        self.rect.set_origin((0.0, 0.0));

        let mut col = Color::rgb(21, 21, 21);

        match self.tile_type {
            TileType::Active => {
                col = Color::GREEN;
            }
            TileType::Blocked => {
                col = Color::BLACK;
            }
            _ => {}
        }

        self.rect.set_fill_color(col);

        win.draw(&self.rect);
    }
}

// -----------------------------------
// MAP
// -----------------------------------
#[allow(dead_code)]
struct Map<'a> {
    tiles: Vec<Tile<'a>>,
    width: i32,
    height: i32,
}

impl<'a> Map<'a> {
    fn new(width: i32, height: i32, map_data: Vec<Tile<'a>>) -> Self {
        Self {
            tiles: map_data,
            width,
            height,
        }
    }

    /// get tile row/column coord from screen coord
    fn get_tile_coord(&self, x: i32, y: i32) -> (i32, i32) {
        let cx = x / BLOCK_SIZE as i32;
        let cy = y / BLOCK_SIZE as i32;
        (cx, cy)
    }

    fn is_tile_active(&self, x: i32, y: i32) -> bool {
        let coord = x + self.width * y;
        if coord < 0 {
            return false;
        }
        if let Some(t) = self.tiles.get(coord as usize) {
            if t.tile_type == TileType::Active {
                return true;
            }
        }
        false
    }

    fn is_tile_blocked(&self, x: i32, y: i32) -> bool {
        let coord = x + self.width * y;
        if coord < 0 {
            return false;
        }
        if let Some(t) = self.tiles.get(coord as usize) {
            if t.tile_type == TileType::Blocked {
                return true;
            }
        }
        false
    }

    fn activate_tile(&mut self, x: i32, y: i32) {
        let coord = x + self.width * y;
        if coord < 0 {
            return;
        }
        if let Some(t) = self.tiles.get_mut(coord as usize) {
            t.tile_type = TileType::Active;
        }
    }

    fn deactivate_tile(&mut self, x: i32, y: i32) {
        let coord = x + self.width * y;
        if coord < 0 {
            return;
        }
        if let Some(t) = self.tiles.get_mut(coord as usize) {
            t.tile_type = TileType::NonActive;
        }
    }

    fn draw(&mut self, win: &mut RenderWindow) {
        let mut i = 0;
        // draw 1d array as a 2d array
        for t in self.tiles.iter_mut() {
            let x = i % 32;
            let y = i / 32;
            t.draw_tile(x as f32, y as f32, win);
            i = i + 1;
        }
    }
}

// -----------------------------------
// FUNCS
// -----------------------------------
fn rand_range(min_value: i32, max_value: i32) -> i32 {
    use rand::{thread_rng, Rng};
    use std::cmp::{max, min};

    let mut rng = thread_rng();

    let min_v = min(min_value, max_value);
    let max_v = max(min_value, max_value);

    let result: i32 = rng.gen_range(min_v, max_v);

    result
}

fn on_key_down(map: &mut HashMap<&Key, bool>, key: &Key) {
    if let Some(x) = map.get_mut(key) {
        *x = true;
    }
}

fn on_key_up(map: &mut HashMap<&Key, bool>, key: &Key) {
    if let Some(x) = map.get_mut(key) {
        *x = false;
    }
}

// TODO: clean up / improve ?
fn new_random_tile<'a>(
    rows: i32,
    cols: i32,
    current_head: &Head<'a>,
    current_tail: &Vec<Tail<'a>>,
    map_data: &Map<'a>,
) -> (i32, i32) {
    loop {
        let rng_x = rand_range(1, rows - 1);
        let rng_y = rand_range(1, cols - 1);

        if rng_x == (current_head.get_x() / BLOCK_SIZE) as i32
            || rng_y == (current_head.get_y() / BLOCK_SIZE) as i32
        {
            // println!("was on head !");
            continue;
        }

        let on_tail = current_tail.iter().any(|x| {
            if (x.get_x() / BLOCK_SIZE) as i32 == rng_x || (x.get_y() / BLOCK_SIZE) as i32 == rng_y
            {
                return true;
            }
            false
        });

        if on_tail {
            // println!("was on tail !");
            continue;
        }

        if map_data.is_tile_blocked(rng_x, rng_y) {
            // println!("was on blocked tile!");
            continue;
        }

        return (rng_x, rng_y);
    }
}

fn load_from_file<'a>() -> Result<Vec<Tile<'a>>> {
    let mut tiles = Vec::new();

    let file = File::open("assets/map/data.txt")?;
    let buffer = BufReader::new(file);

    for line in buffer.lines() {
        let v_line: Vec<char> = line?.chars().collect();
        for x in v_line.iter() {
            match *x {
                '0' => {
                    tiles.push(Tile::new(BLOCK_SIZE, TileType::NonBlocked));
                }
                '1' => {
                    tiles.push(Tile::new(BLOCK_SIZE, TileType::Blocked));
                }
                '2' => {
                    tiles.push(Tile::new(BLOCK_SIZE, TileType::Active));
                }
                _ => {}
            }
        }
    }

    Ok(tiles)
}

fn run(width: u32, height: u32) {
    let mut window = RenderWindow::new((width, height), "sfml", Style::CLOSE, &Default::default());
    window.set_mouse_cursor_visible(true);
    window.set_framerate_limit(30);

    let mut is_running = true;
    let mut pause = false;
    let mut add_segment = false;
    let mut update_snake = Clock::start();

    // key mapings
    let mut keys_hm: HashMap<&Key, bool> = HashMap::new();
    keys_hm.insert(&Key::W, false);
    keys_hm.insert(&Key::D, false);
    keys_hm.insert(&Key::A, false);
    keys_hm.insert(&Key::S, false);

    // objs
    let mut head = Head::new(150.0, 150.0, BLOCK_SIZE, Color::WHITE, Direction::Right);
    let mut tail: Vec<Tail<'_>> = vec![];

    // MAP SIZE = 32 X 24
    let rows = (width / 25) as i32;
    let cols = (height / 25) as i32;
    let map_data = load_from_file().expect("failed to find file");
    let mut map = Map::new(rows, cols, map_data);

    while is_running && window.is_open() {
        // --------------------------
        // inputs
        // --------------------------
        while let Some(ev) = window.poll_event() {
            match ev {
                Event::Closed => {
                    is_running = false;
                }

                Event::KeyPressed { code, .. } => match code {
                    Key::Escape => is_running = false,
                    Key::P => pause = !pause,
                    Key::W => on_key_down(&mut keys_hm, &Key::W),
                    Key::A => on_key_down(&mut keys_hm, &Key::A),
                    Key::S => on_key_down(&mut keys_hm, &Key::S),
                    Key::D => on_key_down(&mut keys_hm, &Key::D),
                    _ => {}
                },
                Event::KeyReleased { code, .. } => match code {
                    Key::W => on_key_up(&mut keys_hm, &Key::W),
                    Key::A => on_key_up(&mut keys_hm, &Key::A),
                    Key::S => on_key_up(&mut keys_hm, &Key::S),
                    Key::D => on_key_up(&mut keys_hm, &Key::D),
                    _ => {}
                },
                _ => {}
            }
        }

        if !pause {

            // --------------------------
            // inputs
            // --------------------------
            head.inputs(&keys_hm);

            // --------------------------
            // update
            // --------------------------
            // current head pos.
            let (hx, hy) = map.get_tile_coord(head.get_x() as i32, head.get_y() as i32);

            // check if head is on blocked tile
            if map.is_tile_blocked(hx, hy) {
                // reset
                head.reset(150.0, 150.0);
                tail.clear();
            }

            // check if head is on active tile
            if map.is_tile_active(hx, hy) {
                map.deactivate_tile(hx, hy);
                let (new_tile_x, new_tile_y) = new_random_tile(rows, cols, &head, &tail, &map);
                map.activate_tile(new_tile_x, new_tile_y);
                add_segment = true;
            }

            // check head is on same tile as one of the tails.
            for t in tail.iter_mut() {
                let (tx, ty) = map.get_tile_coord(t.get_x() as i32, t.get_y() as i32);
                if tx == hx && ty == hy {
                    head.reset(150.0, 150.0);
                    tail.clear();
                    break;
                }
            }

            // update snake every so oftern as to not fly off screen
            if update_snake.elapsed_time().as_milliseconds() >= 95 {
                // store last position
                let mut prev_x = head.get_x();
                let mut prev_y = head.get_y();
                head.update();

                for t in tail.iter_mut() {
                    // store last position
                    let prev_tx = t.get_x();
                    let prev_ty = t.get_y();
                    t.update(prev_x, prev_y);
                    prev_x = prev_tx;
                    prev_y = prev_ty;
                }

                if add_segment {
                    // prev_x and prev_y should be last tail seg prev x and y
                    let new_seg = Tail::new(prev_x, prev_y, BLOCK_SIZE, Color::RED);
                    tail.push(new_seg);
                    add_segment = false;
                }

                update_snake.restart();
            }  

            // --------------------------
            // render
            // --------------------------
            window.clear(Color::WHITE);
            map.draw(&mut window);
            head.draw(&mut window);
            for t in tail.iter_mut() {
                t.draw(&mut window);
            }
            window.display();
        }
    }
}

fn main() {
    run(SCREEN_WIDTH, SCREEN_HEIGHT);
}
