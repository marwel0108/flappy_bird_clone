use bracket_lib::prelude::*;

#[derive(Debug)]
enum GameMode {
    Menu,
    Playing,
    GameOver,
    Pause,
    Quit,
}

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;

#[derive(Debug)]
struct Player {
    x: i32,
    y: i32,
    velocity: f32,
}

#[derive(Debug, Clone, Copy)]
struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
}
#[derive(Debug)]
struct State {
    player: Player,
    frame_time: f32,
    obstacles: [Obstacle; 2],
    mode: GameMode,
    score: i32,
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(2, 20 - score),
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        self.x = self.x - 1;
        let half_size = self.size / 2;

        for y in 0..self.gap_y - half_size {
            ctx.set(self.x, y, RED, BLACK, to_cp437('|'));
        }

        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(self.x, y, RED, BLACK, to_cp437('|'));
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = self.x == player.x;
        let player_above_gap = player.y < self.gap_y - half_size;
        let player_below_gap = player.y > self.gap_y + half_size;
        does_x_match && (player_above_gap || player_below_gap)
    }
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(5, self.y, RGB::from_u8(117, 47, 243), BLACK, 18)
    }

    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.5;
        }

        self.y += self.velocity as i32;
        if self.y < 0 {
            self.y = 0;
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.5;
    }
}

impl State {
    fn new() -> Self {
        State {
            player: Player::new(5, 25),
            frame_time: 0.0,
            obstacles: [
                Obstacle::new(SCREEN_WIDTH, 0),
                Obstacle::new(SCREEN_WIDTH + (SCREEN_WIDTH / 2), 0),
            ],
            mode: GameMode::Menu,
            score: 0,
        }
    }

    fn menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Menu");
        ctx.print_centered(8, "(P) Play game");
        ctx.print_centered(9, "(Q) Quit game");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.mode = GameMode::Playing,
                VirtualKeyCode::Q => self.mode = GameMode::Quit,
                _ => {}
            }
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(BLUE);
        ctx.print(0, 0, "Press [SPACE] to flap");
        ctx.print(0, 2, format!("Score: {}", self.score));

        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
        }

        // if let Some(second_obstacle) = &self.obstacles[0] {

        // }
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Space => self.player.flap(),
                VirtualKeyCode::Escape => self.mode = GameMode::Pause,
                _ => {}
            }
        }
        self.player.render(ctx);

        if self.player.y > SCREEN_HEIGHT || self.obstacles[0].hit_obstacle(&self.player) {
            self.mode = GameMode::GameOver;
        }

        self.obstacles[0].render(ctx);
        self.obstacles[1].render(ctx);

        if self.player.x > self.obstacles[0].x {
            self.obstacles[0] = self.obstacles[1];
            self.obstacles[1] = Obstacle::new(SCREEN_WIDTH, self.score);
            self.score += 1;
        }
    }

    fn restart(&mut self) {
        self.clean_state();
        self.mode = GameMode::Playing;
    }

    fn pause(&mut self, ctx: &mut BTerm) {
        ctx.print_centered(35, "Pause!");
        ctx.print_centered(8, "(SPACE) Resume game");
        ctx.print_centered(9, "(Q) Go to the menu");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Space => self.mode = GameMode::Playing,
                VirtualKeyCode::Q => {
                    self.clean_state();
                    self.mode = GameMode::Menu
                },
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.print_centered(35, "You died");
        ctx.print_centered(8, "(R) Restart game");
        ctx.print_centered(9, "(Q) Go to the menu");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::R => self.restart(),
                VirtualKeyCode::Q => {
                    self.clean_state();
                    self.mode = GameMode::Menu
                },
                _ => {}
            }
        }
    }

    fn clean_state(&mut self) {
        self.player = Player::new(5, 25);
        self.frame_time = 0.0;
        self.obstacles = [
            Obstacle::new(SCREEN_WIDTH, 0),
            Obstacle::new(SCREEN_WIDTH + (SCREEN_WIDTH / 2), 0),
        ];
        self.score = 0;
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::GameOver => self.dead(ctx),
            GameMode::Pause => self.pause(ctx),
            GameMode::Quit => ctx.quitting = true,
            _ => {}
        }
    }
}

fn main() -> BError {
    let b = BTermBuilder::simple80x50()
        .with_title("First test")
        .build()?;

    main_loop(b, State::new())
}
