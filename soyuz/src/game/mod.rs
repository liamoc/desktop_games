pub mod piece;

use self::piece::{Piece,PieceShape};

use imprint::{Imprint, Cell};
use std::path::{Path};

mod score_table;

use self::score_table::ScoreTable;
pub const WIDTH: usize = 10;
pub const HEIGHT: usize = 20;
pub const BUFFER: usize = 2;
pub const ADVANCE_SPEED: i32 = 11;
pub const MAX_LEVEL: u32 = ::FRAMERATE; // should always be <= FRAMERATE
pub const MAX_BTYPE: u32 = 14;
pub const KEY_DELAY: u32 = 2;

pub struct Config {
    pub btype: u32,
    pub level: u32,
}

pub struct InputState {
    pub escape: bool,
    pub down: bool,
    pub up: bool,
    pub left: bool,
    pub right: bool,
    pub button_a: bool,
    pub button_b: bool,
    pub drop: bool,
    pub skip: u32,
}
impl InputState {
    pub fn new() -> InputState {
        InputState {
            skip: 0,
            escape: false,
            down: false,
            left: false,
            right: false,
            button_a: false,
            button_b: false,
            up: false,
            drop: false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TickResult {
    Continue,
    Exit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Active,
    Paused,
    Raising(usize),
    Lowering(usize),
    Menu(u32),
    Clearing(i32),
    Placing(Piece, i32, i32),
}
impl Status {
    pub fn is_inactive(&self) -> bool {
        match *self {
            Status::Raising(_) | Status::Lowering(_) | Status::Paused => true,
            _ => false
        }
    }
}
pub struct Tetris {
    pub config: Config,
    pub status: Status,
    pub current: Piece,
    pub position: (i32, i32),
    pub lines: Vec<usize>,
    next: Piece,
    input: InputState,
    board: Imprint<PieceShape>,
    points: u32,
    drop_rate: u32,
    gravity_tick: u32,
    speed: u32,
    remaining: i32,
    score_table: ScoreTable,
}


impl Tetris {
    pub fn new(filename: &Path) -> ::std::io::Result<Self> {
        let mut g = Tetris {
            config: Config { btype: 0, level: 0 },
            status: Status::Menu(0),
            board: Imprint::empty(WIDTH, HEIGHT + BUFFER),
            gravity_tick: 0,
            speed: MAX_LEVEL,
            remaining: ADVANCE_SPEED,
            drop_rate: 0,
            current: Piece::I2,
            next: Piece::I2,
            position: (0, 0),
            points: 0,
            input: InputState::new(),
            lines: Vec::new(),
            score_table: ScoreTable::new(filename)?
        };
        g.new_piece();
        g.new_piece();
        Ok(g)
    }

    fn new_piece(&mut self) {
        self.current = self.next;
        self.gravity_tick = 0;
        self.next = Piece::rand(&mut rand::thread_rng());
        let x = (WIDTH as i32 - self.current.imprint().size().0 as i32) / 2;
        let y = if self.current == Piece::I1 { 0 } else { 1 };
        self.position = (x, y);
        if !self.move_piece(x, y) || !self.board.all_clear(BUFFER) {
            self.status = Status::Raising(self.board.size().1);
        }
    }
    pub fn top_score(&self) -> u32 {
        self.score_table.get_top_score(&self.config)
    }
    pub fn clear_current_score(&mut self) {
        self.score_table.clear_score(&self.config).unwrap_or_default()
    }
    fn new_game(&mut self) {
        self.board = Imprint::empty(WIDTH, HEIGHT + BUFFER);
        self.score_table
            .update_scores(&self.config, self.points)
            .unwrap();
        self.new_piece();
        self.new_piece();
        self.points = 0;
        self.drop_rate = 0;
        self.speed = MAX_LEVEL - self.config.level;
        self.remaining = (self.config.level + 1) as i32 * ADVANCE_SPEED;
        for i in 0..self.config.btype {
            let top = self.board.size().1 - 1 - i as usize;
            self.board.random_line(top, Cell::Filled(PieceShape::Garbage));
        }
    }

    fn award_points(&mut self, lines: u32) {
        let level = (MAX_LEVEL - self.speed) + 1;
        let award = match lines {
            1 => 40 * level,
            2 => 100 * level,
            3 => 300 * level,
            _ => 1200 * level,
        };
        self.points += award
    }

    fn switch_piece(&mut self, p: Piece) -> bool {
        if self.board.accepts(p.imprint(), self.position) {
            self.current = p;
            true
        } else {
            false
        }
    }

    fn move_piece(&mut self, x: i32, y: i32) -> bool {
        let c = (x, y);
        if self.board.accepts(self.current.imprint(), c) {
            self.position = c;
            true
        } else {
            false
        }
    }

    fn hard_drop(&mut self) {
        while self.status == Status::Active {
            self.drop_rate += 1;
            self.down();
        }
    }

    fn rotate_l(&mut self) {
        let p = self.current.rotate_l();
        self.switch_piece(p);
    }

    fn rotate_r(&mut self) {
        let p = self.current.rotate_r();
        self.switch_piece(p);
    }

    fn check_lines(&mut self) -> bool {
        self.board.full_lines(&mut self.lines)
    }

    fn clear_lines(&mut self) {
        let lines = self.lines.len() as u32;
        self.award_points(lines);
        self.board.clear_lines(&mut self.lines)
    }

    fn down(&mut self) {
        let (x, y) = self.position;
        if !self.move_piece(x, y + 1) {
            self.points += self.drop_rate;
            self.drop_rate = 0;
            self.board.stamp(self.current.imprint(), self.position);
            if !self.check_lines() {
                self.status = Status::Placing(self.current, x, y);
            } else {
                self.remaining -= self.lines.len() as i32;
                while self.remaining <= 0 {
                    self.remaining += ADVANCE_SPEED;
                    if self.speed > 1 {
                        self.speed -= 1;
                    }
                }
                self.status = Status::Clearing(self.lines.len() as i32 * 3);
            }
        }
    }

    fn left(&mut self) {
        let (x, y) = self.position;
        self.move_piece(x - 1, y);
    }

    fn right(&mut self) {
        let (x, y) = self.position;
        self.move_piece(x + 1, y);
    }

    pub fn current_level(&self) -> u32 {
        MAX_LEVEL - self.speed
    }
    pub fn score(&self) -> u32 {
        self.points
    }
    pub fn board(&self) -> &Imprint<PieceShape > {
        &self.board
    }
    pub fn next(&self) -> Option<&Imprint<PieceShape >> {
        match self.status {
            Status::Menu(_) => None,
            _ => Some(self.next.imprint()),
        }
    }
    pub fn tick(&mut self) -> TickResult {
        match self.status {
            Status::Active => {
                if self.input.escape {
                    self.status = Status::Paused;
                    self.input.escape = false;
                } else {
                    if self.input.left {
                        if self.input.skip == 0 || self.input.skip > KEY_DELAY {
                            self.left();
                        }
                        if self.input.skip <= KEY_DELAY {
                            self.input.skip += 1;
                        }
                    } else if self.input.right {
                        if self.input.skip == 0 || self.input.skip > KEY_DELAY {
                            self.right();
                        }
                        if self.input.skip <= KEY_DELAY {
                            self.input.skip += 1;
                        }
                    }
                    if self.input.button_b {
                        self.rotate_r();
                        self.input.button_b = false;
                    } else if self.input.button_a || self.input.up {
                        self.rotate_l();
                        self.input.button_a = false;
                        self.input.up = false;
                    }
                    if self.input.drop {
                        self.input.drop = false;
                        self.hard_drop()
                    } else if self.input.down {
                        self.drop_rate += 1;
                        self.down()
                    } else {
                        self.drop_rate = 0;
                        self.gravity_tick = (self.gravity_tick + 1) % self.speed;
                        if self.gravity_tick == 0 {
                            self.down()
                        }
                    }
                }
            }
            Status::Paused => {
                if self.input.drop || self.input.button_a || self.input.button_b ||
                    self.input.left || self.input.right || self.input.down || self.input.up
                {
                    self.input.drop = false;
                    self.status = Status::Active;
                } else if self.input.escape {
                    self.input.escape = false;
                    self.status = Status::Raising(self.board.size().1);
                }
            }

            Status::Raising(f) => {
                if f == BUFFER {
                    self.new_game();
                    self.status = Status::Lowering(f)
                } else {
                    self.status = Status::Raising(f - 1);
                }
            }
            Status::Lowering(f) => {
                if f == self.board.size().1 {
                    self.status = Status::Menu(0);
                } else {
                    self.status = Status::Lowering(f + 1);
                }
            }
            Status::Menu(f) => {
                self.status = Status::Menu((f + 1) % 70);
                if self.input.escape {
                    self.input.escape = false;
                    return TickResult::Exit;
                }
                if self.input.right && self.config.btype < MAX_BTYPE {
                    self.input.right = false;
                    let top = self.board.size().1 - 1 - self.config.btype as usize;
                    self.board.random_line(top, Cell::Filled(PieceShape::Garbage));
                    self.config.btype += 1;
                }
                if self.input.left && self.config.btype > 0 {
                    self.input.left = false;
                    self.config.btype -= 1;
                    let top = self.board.size().1 - 1 - self.config.btype as usize;
                    self.board.clear_line(top);
                }
                if self.input.drop {
                    self.input.drop = false;
                    self.status = Status::Active;
                }
                if self.input.up {
                    self.input.up = false;
                    if self.config.level < MAX_LEVEL - 1 {
                        self.config.level += 1;
                        self.speed = MAX_LEVEL - self.config.level;
                        self.remaining = (self.config.level + 1) as i32 * ADVANCE_SPEED;
                    }
                }
                if self.input.down {
                    self.input.down = false;
                    if self.config.level > 0 {
                        self.config.level -= 1;
                        self.speed = MAX_LEVEL - self.config.level;
                        self.remaining = (self.config.level + 1) as i32 * ADVANCE_SPEED;
                    }
                }
            }
            Status::Clearing(0) => {
                self.clear_lines();
                self.status = Status::Active;
                self.new_piece();
            }
            Status::Clearing(f) => self.status = Status::Clearing(f - 1),
            Status::Placing(_, _, _) => {
                self.status = Status::Active;
                self.new_piece();
            }
        }
        TickResult::Continue
    }
    pub fn is_paused(&self) -> bool {
        return self.status == Status::Paused;
    }

    pub fn input_state(&mut self) -> &mut InputState {
        &mut self.input
    }
}
