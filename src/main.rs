use notan::draw::*;
use notan::prelude::*;
use std::collections::VecDeque;
use std::process;
use rand::Rng;

const WIN_WIDTH: i32 = 1280;
const WIN_HEIGHT: i32 = 720;

const BOARD_WIDTH: u32 = 32;
const BOARD_HEIGHT: u32 = 32;
const BOARD_CELL_SIZE: u32 = 16;
const BOARD_OFFSET_X: f32 = (WIN_WIDTH as f32 / 2.0) - (BOARD_PX_WIDTH / 2.0);
const BOARD_OFFSET_Y: f32 = (WIN_HEIGHT as f32 / 2.0) - (BOARD_PX_HEIGHT / 2.0);
const BOARD_PX_WIDTH: f32 = BOARD_CELL_SIZE as f32 * BOARD_WIDTH as f32;
const BOARD_PX_HEIGHT: f32 = BOARD_CELL_SIZE as f32 * BOARD_HEIGHT as f32;


#[derive(PartialEq)]
enum Direction { UP, DOWN, LEFT, RIGHT }

#[derive(Clone, Copy, PartialEq)]
struct GridPoint {
    x : u32,
    y : u32,
}

impl GridPoint {
    fn new(x: u32, y: u32) -> Self { Self { x, y } }
}

struct Snake {
    dir: Direction,
    body: VecDeque<GridPoint>,
    time_since_last_move: f32,
    move_delay: f32,
}

impl Snake {
    // bool states whether fruit was collected or not
    fn update(&mut self, food: &GridPoint, delta: f32) -> Result <bool, String> {
        self.time_since_last_move += delta;
        if self.time_since_last_move >= self.move_delay {
            self.time_since_last_move = 0.0;
            let n: GridPoint;
            match self.check_move() {
                Ok(x) => n = x,
                Err(s) => {
                    return Err(s.to_string());
                },
            }
            return Ok(self.do_move(n, food));
        }
        Ok(false)
    }

    fn do_move(&mut self, mv: GridPoint, food: &GridPoint) -> bool {
        self.body.push_front(mv);
        let food_eaten = mv == *food;
        if !food_eaten {
            self.body.pop_back();
        }
        food_eaten
    }

    fn check_move(& self) -> Result <GridPoint, String> {
        let head = self.body.front().unwrap().clone();
        if (head.x == 0 && self.dir == Direction::LEFT) ||
        (head.x == BOARD_WIDTH-1 && self.dir == Direction::RIGHT) ||
        (head.y == 0 && self.dir == Direction::UP) ||
        (head.y == BOARD_HEIGHT-1 && self.dir == Direction::DOWN) {
            return Err("Crashed Into Wall!".to_string());
        }

        let pos = (head.x as i32, head.y as i32);
        let mv = match self.dir {
            Direction::UP => (0, -1),
            Direction::DOWN => (0, 1),
            Direction::LEFT => (-1, 0),
            Direction::RIGHT => (1, 0),
        };

        let newhead = GridPoint::new((pos.0 + mv.0) as u32, (pos.1 + mv.1) as u32);

        if self.body.contains(&newhead) {
            return Err("Crashed Into Self!".to_string());
        }
        Ok(newhead)
    }
}

#[derive(AppState)]
struct State {
    snake: Snake,
    font: Font,
    fps : f32,
    delta : f32,
    food: GridPoint,
    score : u32,
}

#[notan_main]
fn main() -> Result<(), String> {
    notan::init_with(init)
        .add_config(DrawConfig)
        .add_config(WindowConfig::new().size(WIN_WIDTH,WIN_HEIGHT).vsync().title("snake"))
        .draw(draw)
        .update(update)
        .build()
}

fn init(gfx: &mut Graphics) -> State {
    let body: VecDeque<GridPoint> = VecDeque::from([
    GridPoint::new(0, 0), GridPoint::new(0, 1), GridPoint::new(0, 2) ]);

    let snake = Snake {
        dir: Direction::RIGHT,
        body,
        time_since_last_move: 0.0,
        move_delay: 0.075};

    let font = gfx
        .create_font(include_bytes!("fonts/font.ttf"))
        .unwrap();

    let food = generate_food();

    State { snake , font , fps : 0.0, delta : 0.0 , food, score: 0 }
}

fn generate_food() -> GridPoint {
    GridPoint::new(rand::thread_rng().gen_range(0..BOARD_WIDTH),
        rand::thread_rng().gen_range(0..BOARD_WIDTH))
}

fn update(app: &mut App, state: &mut State) {
    if  app.keyboard.is_down(KeyCode::Escape) ||
        app.keyboard.is_down(KeyCode::Q) {
        quit(None);
    }

    if app.keyboard.is_down(KeyCode::W) && state.snake.dir != Direction::DOWN {
        state.snake.dir = Direction::UP;
    }
    else if app.keyboard.is_down(KeyCode::S) && state.snake.dir != Direction::UP {
        state.snake.dir = Direction::DOWN;
    }
    else if app.keyboard.is_down(KeyCode::A) && state.snake.dir != Direction::RIGHT {
        state.snake.dir = Direction::LEFT;
    }
    else if app.keyboard.is_down(KeyCode::D) && state.snake.dir != Direction::LEFT {
        state.snake.dir = Direction::RIGHT;
    }

    match state.snake.update(&state.food, app.timer.delta_f32()) {
        Ok(food_eaten) => {
            if food_eaten {
                state.score += 1;
                state.food = generate_food();
            }
        },
        Err(s) => {println!("{}", s); process::exit(0);}
    };

    state.fps = app.timer.fps();
    state.delta = app.timer.delta_f32();
}

fn draw(gfx: &mut Graphics, state: &mut State) {
    let mut draw = gfx.create_draw();
    draw.clear(Color::BLACK);
    for i in 0..=BOARD_WIDTH {
        draw.line((BOARD_OFFSET_X + BOARD_CELL_SIZE as f32 * i as f32, BOARD_OFFSET_Y),
            (BOARD_OFFSET_X + BOARD_CELL_SIZE as f32 * i as f32,
                BOARD_OFFSET_Y as f32 + BOARD_PX_HEIGHT));
    }
    for i in 0..=BOARD_HEIGHT {
        draw.line((BOARD_OFFSET_X, BOARD_OFFSET_Y+BOARD_CELL_SIZE as f32 * i as f32),
            (BOARD_OFFSET_X + BOARD_PX_WIDTH, BOARD_OFFSET_Y + BOARD_CELL_SIZE as f32 * i as f32));
    }

    for gp in state.snake.body.iter().skip(1) {
        fill_square(gp, &mut draw, Color::WHITE);
    }
    fill_square(&state.snake.body[0], &mut draw, Color::GREEN);

    fill_square(&state.food, &mut draw, Color::RED);

    // draw.text(&state.font, format!("FPS: {:.2}\nDELTA: {:.2}", state.fps, state.delta).as_str());

    draw.text(&state.font, format!("SCORE: {}", state.score).as_str()).h_align_center().position(WIN_WIDTH as f32 / 2.0, 50.0);

    gfx.render(&draw);
}

fn fill_square(gp: &GridPoint, d: &mut Draw, col: Color) {
    d.rect((BOARD_OFFSET_X + (gp.x as f32 * BOARD_CELL_SIZE as f32) - 1.0, BOARD_OFFSET_Y + (gp.y as f32 * BOARD_CELL_SIZE as f32)),
        (BOARD_CELL_SIZE as f32 + 1.0, BOARD_CELL_SIZE as f32 + 1.0)).color(col);
}

fn quit(code: Option<String>) {
    match code {
        None => process::exit(0),
        Some(s) => {
            eprintln!("{}", s);
            process::exit(1);
        }
    }
}
