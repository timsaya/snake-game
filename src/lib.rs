use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

slint::include_modules!();

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq)]
struct GamePosition {
    x: i32,
    y: i32,
}

struct SnakeGame {
    snake: VecDeque<GamePosition>,
    food: GamePosition,
    direction: Direction,
    grid_size: i32,
    cell_size: i32,
    score: i32,
    high_score: i32,
    game_over: bool,
    game_paused: bool,
    game_running: bool,
}

impl SnakeGame {
    fn new() -> Self {
        let grid_size = 20;
        let cell_size = 20;
        let mut snake = VecDeque::new();
        snake.push_back(GamePosition { x: 10, y: 10 });
        snake.push_back(GamePosition { x: 10, y: 11 });
        snake.push_back(GamePosition { x: 10, y: 12 });

        Self {
            snake,
            food: GamePosition { x: 5, y: 5 },
            direction: Direction::Up,
            grid_size,
            cell_size,
            score: 0,
            high_score: 0,
            game_over: false,
            game_paused: false,
            game_running: false,
        }
    }

    fn reset(&mut self) {
        let mut snake = VecDeque::new();
        snake.push_back(GamePosition { x: 10, y: 10 });
        snake.push_back(GamePosition { x: 10, y: 11 });
        snake.push_back(GamePosition { x: 10, y: 12 });

        self.snake = snake;
        self.food = GamePosition { x: 5, y: 5 };
        self.direction = Direction::Up;
        self.score = 0;
        self.game_over = false;
        self.game_paused = false;
        self.game_running = false;
    }

    fn change_direction(&mut self, new_direction: Direction) {
        match (self.direction, new_direction) {
            (Direction::Up, Direction::Down)
            | (Direction::Down, Direction::Up)
            | (Direction::Left, Direction::Right)
            | (Direction::Right, Direction::Left) => return,
            _ => self.direction = new_direction,
        }
    }

    fn move_snake(&mut self) {
        if self.game_over || self.game_paused || !self.game_running {
            return;
        }

        let head = self.snake.front().unwrap();
        let new_head = match self.direction {
            Direction::Up => GamePosition {
                x: head.x,
                y: (head.y - 1 + self.grid_size) % self.grid_size,
            },
            Direction::Down => GamePosition {
                x: head.x,
                y: (head.y + 1) % self.grid_size,
            },
            Direction::Left => GamePosition {
                x: (head.x - 1 + self.grid_size) % self.grid_size,
                y: head.y,
            },
            Direction::Right => GamePosition {
                x: (head.x + 1) % self.grid_size,
                y: head.y,
            },
        };

        // Check if snake hits itself
        if self.snake.contains(&new_head) {
            self.game_over = true;
            return;
        }

        self.snake.push_front(new_head);

        // Check if snake eats food
        if new_head == self.food {
            self.score += 10;
            if self.score > self.high_score {
                self.high_score = self.score;
            }
            self.generate_food();
        } else {
            self.snake.pop_back();
        }
    }

    fn generate_food(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        loop {
            let food = GamePosition {
                x: rng.gen_range(0..self.grid_size),
                y: rng.gen_range(0..self.grid_size),
            };

            if !self.snake.contains(&food) {
                self.food = food;
                break;
            }
        }
    }

    fn get_snake_positions_array(&self) -> Vec<Position> {
        self.snake
            .iter()
            .map(|pos| Position { x: pos.x, y: pos.y })
            .collect()
    }

    fn get_food_position(&self) -> Position {
        Position {
            x: self.food.x,
            y: self.food.y,
        }
    }
}

fn update_ui(ui: &AppWindow, game: &SnakeGame) {
    ui.set_score(game.score);
    ui.set_high_score(game.high_score);
    ui.set_game_over(game.game_over);
    ui.set_game_paused(game.game_paused);
    ui.set_game_running(game.game_running);
    let snake_positions = game.get_snake_positions_array();
    ui.set_snake_positions(snake_positions.as_slice().into());
    ui.set_food_position(game.get_food_position());
    ui.set_grid_size(game.grid_size);
    ui.set_cell_size(game.cell_size);
}

#[cfg(not(target_family = "wasm"))]
pub fn start_app() -> Result<(), Box<dyn std::error::Error>> {
    let ui = AppWindow::new()?;
    let game_state = Rc::new(RefCell::new(SnakeGame::new()));

    // Initialize UI
    {
        let game = game_state.borrow();
        update_ui(&ui, &game);
    }

    // Game timer callback
    let ui_weak = ui.as_weak();
    let game_state_clone = game_state.clone();
    ui.on_game_tick(move || {
        let ui = ui_weak.upgrade().unwrap();
        let mut game = game_state_clone.borrow_mut();
        game.move_snake();
        update_ui(&ui, &game);
    });

    // Start game callback
    let ui_weak = ui.as_weak();
    let game_state_clone = game_state.clone();
    ui.on_start_game(move || {
        let ui = ui_weak.upgrade().unwrap();
        let mut game = game_state_clone.borrow_mut();
        game.game_running = true;
        game.game_paused = false;
        update_ui(&ui, &game);
    });

    // Pause game callback
    let ui_weak = ui.as_weak();
    let game_state_clone = game_state.clone();
    ui.on_pause_game(move || {
        let ui = ui_weak.upgrade().unwrap();
        let mut game = game_state_clone.borrow_mut();
        game.game_paused = !game.game_paused;
        update_ui(&ui, &game);
    });

    // Reset game callback
    let ui_weak = ui.as_weak();
    let game_state_clone = game_state.clone();
    ui.on_reset_game(move || {
        let ui = ui_weak.upgrade().unwrap();
        let mut game = game_state_clone.borrow_mut();
        game.reset();
        update_ui(&ui, &game);
    });

    // Move snake callback
    let ui_weak = ui.as_weak();
    let game_state_clone = game_state.clone();
    ui.on_move_snake(move |direction| {
        let ui = ui_weak.upgrade().unwrap();
        let mut game = game_state_clone.borrow_mut();
        let new_direction = match direction {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Right,
            _ => return,
        };
        game.change_direction(new_direction);
        update_ui(&ui, &game);
    });

    ui.run()?;
    Ok(())
}



#[cfg(target_family = "wasm")]
#[wasm_bindgen]
pub fn start_app() -> Result<(), wasm_bindgen::JsValue> {
    let ui = AppWindow::new().map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))?;
    let game_state = Rc::new(RefCell::new(SnakeGame::new()));

    // Initialize UI
    {
        let game = game_state.borrow();
        update_ui(&ui, &game);
    }

    // Game timer callback
    let ui_weak = ui.as_weak();
    let game_state_clone = game_state.clone();
    ui.on_game_tick(move || {
        let ui = ui_weak.upgrade().unwrap();
        let mut game = game_state_clone.borrow_mut();
        game.move_snake();
        update_ui(&ui, &game);
    });

    // Start game callback
    let ui_weak = ui.as_weak();
    let game_state_clone = game_state.clone();
    ui.on_start_game(move || {
        let ui = ui_weak.upgrade().unwrap();
        let mut game = game_state_clone.borrow_mut();
        game.game_running = true;
        game.game_paused = false;
        update_ui(&ui, &game);
    });

    // Pause game callback
    let ui_weak = ui.as_weak();
    let game_state_clone = game_state.clone();
    ui.on_pause_game(move || {
        let ui = ui_weak.upgrade().unwrap();
        let mut game = game_state_clone.borrow_mut();
        game.game_paused = !game.game_paused;
        update_ui(&ui, &game);
    });

    // Reset game callback
    let ui_weak = ui.as_weak();
    let game_state_clone = game_state.clone();
    ui.on_reset_game(move || {
        let ui = ui_weak.upgrade().unwrap();
        let mut game = game_state_clone.borrow_mut();
        game.reset();
        update_ui(&ui, &game);
    });

    // Move snake callback
    let ui_weak = ui.as_weak();
    let game_state_clone = game_state.clone();
    ui.on_move_snake(move |direction| {
        let ui = ui_weak.upgrade().unwrap();
        let mut game = game_state_clone.borrow_mut();
        let new_direction = match direction {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Right,
            _ => return,
        };
        game.change_direction(new_direction);
        update_ui(&ui, &game);
    });

    ui.run().map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))?;
    Ok(())
} 