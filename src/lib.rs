use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet, VecDeque};
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct GamePosition {
    x: i32,
    y: i32,
}

// A*算法节点
#[derive(Clone, Copy, PartialEq, Eq)]
struct AStarNode {
    position: GamePosition,
    g_cost: i32,
    h_cost: i32,
    parent: Option<GamePosition>,
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // 优先队列需要最小堆，所以反转比较
        (other.g_cost + other.h_cost).cmp(&(self.g_cost + self.h_cost))
    }
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
    auto_mode: bool,
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
            auto_mode: false,
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
        self.auto_mode = false;
    }

    fn toggle_auto_mode(&mut self) {
        self.auto_mode = !self.auto_mode;
    }

    // 计算曼哈顿距离
    fn manhattan_distance(&self, pos1: GamePosition, pos2: GamePosition) -> i32 {
        let dx = (pos1.x - pos2.x).abs();
        let dy = (pos1.y - pos2.y).abs();
        dx + dy
    }

    // 获取相邻位置
    fn get_neighbors(&self, pos: GamePosition) -> Vec<GamePosition> {
        let mut neighbors = Vec::new();
        let directions = [
            (0, -1), // Up
            (0, 1),  // Down
            (-1, 0), // Left
            (1, 0),  // Right
        ];

        for (dx, dy) in directions.iter() {
            let new_x = (pos.x + dx + self.grid_size) % self.grid_size;
            let new_y = (pos.y + dy + self.grid_size) % self.grid_size;
            neighbors.push(GamePosition { x: new_x, y: new_y });
        }
        neighbors
    }

    // 检查位置是否安全（不在蛇身上）
    fn is_safe_position(&self, pos: GamePosition, snake_body: &VecDeque<GamePosition>) -> bool {
        !snake_body.contains(&pos)
    }

    // 预演：检查吃了食物后是否会被困住
    fn will_get_trapped_after_eating(&self) -> bool {
        // 模拟吃了食物后的蛇身
        let mut simulated_snake = self.snake.clone();
        simulated_snake.push_front(self.food);

        // 检查从食物位置是否能到达蛇尾
        let tail = simulated_snake.back().unwrap();
        let path_to_tail = self.find_path(self.food, *tail, &simulated_snake);

        path_to_tail.is_none()
    }

    // A*算法寻找最短路径
    fn find_path(
        &self,
        start: GamePosition,
        goal: GamePosition,
        snake_body: &VecDeque<GamePosition>,
    ) -> Option<Vec<GamePosition>> {
        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();
        let mut came_from = std::collections::HashMap::new();
        let mut g_score = std::collections::HashMap::new();

        open_set.push(AStarNode {
            position: start,
            g_cost: 0,
            h_cost: self.manhattan_distance(start, goal),
            parent: None,
        });

        g_score.insert(start, 0);

        while let Some(current) = open_set.pop() {
            if current.position == goal {
                // 重建路径
                let mut path = Vec::new();
                let mut current_pos = current.position;
                path.push(current_pos);

                while let Some(parent) = came_from.get(&current_pos) {
                    path.push(*parent);
                    current_pos = *parent;
                }
                path.reverse();
                return Some(path);
            }

            closed_set.insert(current.position);

            for neighbor in self.get_neighbors(current.position) {
                if closed_set.contains(&neighbor) || !self.is_safe_position(neighbor, snake_body) {
                    continue;
                }

                let tentative_g_score = g_score.get(&current.position).unwrap_or(&i32::MAX) + 1;

                if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
                    came_from.insert(neighbor, current.position);
                    g_score.insert(neighbor, tentative_g_score);

                    let h_score = self.manhattan_distance(neighbor, goal);
                    open_set.push(AStarNode {
                        position: neighbor,
                        g_cost: tentative_g_score,
                        h_cost: h_score,
                        parent: Some(current.position),
                    });
                }
            }
        }

        None
    }

    // 自动移动算法
    fn auto_move(&mut self) {
        if !self.auto_mode || self.game_over || self.game_paused || !self.game_running {
            return;
        }

        let head = self.snake.front().unwrap();

        // 首先尝试直接去吃食物
        let path_to_food = self.find_path(*head, self.food, &self.snake);

        if let Some(path) = &path_to_food {
            if path.len() > 1 {
                let next_pos = path[1];
                let new_direction = self.get_direction_to_position(*head, next_pos);
                self.direction = new_direction;
                return;
            }
        }

        // 如果无法直接到达食物，检查吃了食物后是否会被困住
        if !self.will_get_trapped_after_eating() {
            // 安全，继续尝试吃食物
            if let Some(path) = &path_to_food {
                if path.len() > 1 {
                    let next_pos = path[1];
                    let new_direction = self.get_direction_to_position(*head, next_pos);
                    self.direction = new_direction;
                    return;
                }
            }
        }

        // 如果吃食物不安全，寻找安全路径
        let tail = self.snake.back().unwrap();
        let path_to_tail = self.find_path(*head, *tail, &self.snake);

        if let Some(path) = path_to_tail {
            if path.len() > 1 {
                let next_pos = path[1];
                let new_direction = self.get_direction_to_position(*head, next_pos);
                self.direction = new_direction;
                return;
            }
        }

        // 如果连尾巴都到不了，尝试随机安全方向
        for direction in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            let next_pos = match direction {
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

            if self.is_safe_position(next_pos, &self.snake) {
                self.direction = direction;
                return;
            }
        }
    }

    // 获取从当前位置到目标位置的方向
    fn get_direction_to_position(&self, from: GamePosition, to: GamePosition) -> Direction {
        let dx = to.x - from.x;
        let dy = to.y - from.y;

        // 处理边界环绕
        let adjusted_dx = if dx.abs() > self.grid_size / 2 {
            if dx > 0 {
                dx - self.grid_size
            } else {
                dx + self.grid_size
            }
        } else {
            dx
        };

        let adjusted_dy = if dy.abs() > self.grid_size / 2 {
            if dy > 0 {
                dy - self.grid_size
            } else {
                dy + self.grid_size
            }
        } else {
            dy
        };

        if adjusted_dx.abs() > adjusted_dy.abs() {
            if adjusted_dx > 0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else {
            if adjusted_dy > 0 {
                Direction::Down
            } else {
                Direction::Up
            }
        }
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

        // 如果是自动模式，先计算下一步方向
        if self.auto_mode {
            self.auto_move();
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
    ui.set_auto_mode(game.auto_mode);
    let snake_positions = game.get_snake_positions_array();
    ui.set_snake_positions(snake_positions.as_slice().into());
    ui.set_food_position(game.get_food_position());
    ui.set_grid_size(game.grid_size);
    ui.set_cell_size(game.cell_size);
}

#[cfg(not(target_family = "wasm"))]
pub fn start_app() -> Result<(), Box<dyn std::error::Error>> {
    let mut backend = i_slint_backend_winit::Backend::new().unwrap();

    backend.window_attributes_hook = Some(Box::new(|builder| {
        use i_slint_backend_winit::winit::platform::macos::WindowAttributesExtMacOS;

        builder
            .with_fullsize_content_view(true)
            .with_title_hidden(true)
            .with_titlebar_transparent(true)
    }));

    let _ = slint::platform::set_platform(Box::new(backend));

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

    // Toggle auto mode callback
    let ui_weak = ui.as_weak();
    let game_state_clone = game_state.clone();
    ui.on_toggle_auto_mode(move || {
        let ui = ui_weak.upgrade().unwrap();
        let mut game = game_state_clone.borrow_mut();
        game.toggle_auto_mode();
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

    // Toggle auto mode callback
    let ui_weak = ui.as_weak();
    let game_state_clone = game_state.clone();
    ui.on_toggle_auto_mode(move || {
        let ui = ui_weak.upgrade().unwrap();
        let mut game = game_state_clone.borrow_mut();
        game.toggle_auto_mode();
        update_ui(&ui, &game);
    });

    ui.run()
        .map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))?;
    Ok(())
}
