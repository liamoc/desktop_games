pub mod piece;

use self::piece::{Piece,FieldTile, PieceColor};

use imprint::{Imprint, Cell};
use rand::Rng;
use std::path::Path;

mod score_table;

use self::score_table::ScoreTable;
pub const WIDTH: usize = 8;
pub const HEIGHT: usize = 17;
pub const BUFFER: usize = 1;
pub const MAX_LEVEL: u32 = 20;
pub const KEY_DELAY: u32 = 2;


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GameSpeed { Slow, Medium, Fast }
impl GameSpeed {
    pub fn to_index(&self) -> usize {
        match *self {
            GameSpeed::Fast => 2,
            GameSpeed::Medium => 1,
            _ => 0
        }
    }
}
pub struct Config {
    pub speed: GameSpeed,
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
    Reacting,
    Clearing(u32),
    Falling(u32,bool),
    Infecting(u32, usize)
}
impl Status {
    pub fn is_inactive(&self) -> bool {
        match *self {
            Status::Raising(_) | Status::Lowering(_) | Status::Paused => true,
            _ => false
        }
    }
}
pub struct Game {
    pub config: Config,
    pub status: Status,
    pub current: Piece,
    pub position: (i32, i32),
    next: Piece,
    input: InputState,
    board: Imprint<FieldTile>,
    points: u32,
    drop_rate: u32,
    gravity_tick: u32,
    speed: u32,
    level: u32,
    available_colors: Vec<PieceColor>,
    pub clearing: Vec<(usize,usize)>,
    killed_viruses: usize,
    score_table: ScoreTable,
}


impl Game {
    pub fn new(filename: &Path) -> ::std::io::Result<Self> {
        let mut g = Game {
            config: Config { speed: GameSpeed::Medium, level: 0 },
            status: Status::Menu(0),
            board: Imprint::empty(WIDTH, HEIGHT + BUFFER),
            gravity_tick: 0,
            speed: 0,
            level: 0,
            drop_rate: 0,
            current: Piece::CapH(PieceColor::Red, PieceColor::Red),
            next: Piece::CapH(PieceColor::Red, PieceColor::Red),
            position: (0, 0),
            points: 0,
            input: InputState::new(),
            clearing: Vec::new(),
            available_colors: Vec::new(),
            killed_viruses: 0,
            score_table: ScoreTable::new(filename)?
        };
        g.new_game();
        Ok(g)
    }

    fn new_piece(&mut self) {
        self.current = self.next;
        self.gravity_tick = 0;
        self.next = Piece::rand(&mut rand::thread_rng());
        let x = (WIDTH as i32 - 2) / 2;
        let y = 0;
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
        self.level = self.config.level;
        self.points = 0;
        self.drop_rate = 0;
        self.speed = match self.config.speed {
            GameSpeed::Fast => 4,
            GameSpeed::Medium => 8,
            GameSpeed::Slow => 16
        };
        self.setup_board()
    }
    fn setup_board(&mut self) {
        self.board = Imprint::empty(WIDTH, HEIGHT + BUFFER);
        self.available_colors.clear();
        for _i in 0..((self.config.level+1)*4).min(84) {
            self.place_virus()
        }
    } 
    fn place_virus(&mut self) {
        if self.available_colors.is_empty() {
            self.available_colors = vec![PieceColor::Red, PieceColor::Green, PieceColor::Yellow];
        }
        loop {
            let c = rand::thread_rng().gen_range(0, self.available_colors.len());
            let x = rand::thread_rng().gen_range(0, WIDTH);
            let y = rand::thread_rng().gen_range(4+BUFFER, HEIGHT+BUFFER);
            if self.board[(x,y)].is_empty() {
                self.board[(x,y)] = Cell::Filled(FieldTile::Virus(self.available_colors[c]));
                if !self.check_combos(true) {
                    self.available_colors.remove(c);                    
                    break
                } else {
                    self.board[(x,y)] = Cell::Empty
                }
            }
        }
    }


    fn switch_piece(&mut self, p: Piece) -> bool {
        if self.board.accepts(&p.imprint(), self.position) {
            self.current = p;
            true
        } else {
            false
        }
    }

    fn move_piece(&mut self, x: i32, y: i32) -> bool {
        let c = (x, y);
        if self.board.accepts(&self.current.imprint(), c) {
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

    fn update_gravity(&mut self) -> bool {
        let mut ret = false;
        for i in 0..self.board.size().0 {
            for j in (0..self.board.size().1-1).rev() {
                match self.board[(i,j)] {
                    Cell::Empty => {},
                    Cell::Filled(FieldTile::Orphan(_)) | Cell::Filled(FieldTile::CapTop(_)) | Cell::Filled(FieldTile::CapBottom(_)) => {
                        if self.board[(i,j+1)].is_empty() {
                            self.board[(i,j+1)] = self.board[(i,j)];
                            self.board[(i,j)] = Cell::Empty;
                            ret = true;
                        }
                    }
                    Cell::Filled(FieldTile::CapLeft(_)) => {
                        if self.board[(i,j+1)].is_empty() && self.board[(i+1,j+1)].is_empty() {
                            self.board[(i,j+1)] = self.board[(i,j)];
                            self.board[(i+1,j+1)] = self.board[(i+1,j)];
                            self.board[(i,j)] = Cell::Empty;
                            self.board[(i+1,j)] = Cell::Empty;
                            ret = true;
                        }
                    }
                    Cell::Filled(_) => {}
                }
            }
        }
        ret
    }
    fn down(&mut self) {
        let (x, y) = self.position;
        if !self.move_piece(x, y + 1) {
            self.drop_rate = 0;
            self.board.stamp(&self.current.imprint(), self.position);
            self.status = Status::Reacting;
            self.new_piece();            
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
    fn clear_square(&mut self, coord : (usize,usize)) {
        self.clearing.push(coord);
        match self.board[coord] {
            Cell::Empty => {}, //we must already have cleared it
            Cell::Filled(FieldTile::Orphan(_)) => {},
            Cell::Filled(FieldTile::Virus(_)) => {
                self.killed_viruses += 1
            },
            Cell::Filled(FieldTile::CapLeft(_)) => {
                if let Cell::Filled(FieldTile::CapRight(c)) = self.board[(coord.0+1,coord.1)] {
                    self.board[(coord.0+1,coord.1)] = Cell::Filled(FieldTile::Orphan(c));
                }
            },
            Cell::Filled(FieldTile::CapRight(_)) => {
                if let Cell::Filled(FieldTile::CapLeft(c)) = self.board[(coord.0-1,coord.1)] {
                    self.board[(coord.0-1,coord.1)] = Cell::Filled(FieldTile::Orphan(c));
                }
            },
            Cell::Filled(FieldTile::CapTop(_)) => {
                if let Cell::Filled(FieldTile::CapBottom(c)) = self.board[(coord.0,coord.1+1)] {
                    self.board[(coord.0,coord.1+1)] = Cell::Filled(FieldTile::Orphan(c));
                }
            },
            Cell::Filled(FieldTile::CapBottom(_)) => {
                if let Cell::Filled(FieldTile::CapTop(c)) = self.board[(coord.0,coord.1-1)] {
                    self.board[(coord.0,coord.1-1)] = Cell::Filled(FieldTile::Orphan(c));
                }
            }, 
        }
        self.board[coord] = Cell::Empty;
    }
    fn check_won_level(&mut self) -> bool {        
        for i in 0..self.board.size().0 {
            for j in 0..self.board.size().1 {
                match self.board[(i,j)] {
                    Cell::Filled(FieldTile::Virus(_)) => return false,
                    _ => {},
                }
            }
        }
        return true
    }
    fn check_combos(&mut self, readonly:bool) -> bool {
        let mut ret = false;
        for i in 0..self.board.size().1 {
            for j in 0..self.board.size().0-3 {
                let c1 = self.board[(j,i)].as_option().map(|x| x.color());
                let mut combo_len = 0;
                if c1.is_some() {
                    for j2 in j+1..self.board().size().0 {
                        if self.board[(j2,i)].as_option().map(|x| x.color()) == c1 {
                            combo_len += 1
                        } else {
                            break
                        }
                    }
                    if combo_len >= 3 {
                        for j2 in j..=j+combo_len {
                            if !readonly { self.clear_square((j2,i)); }
                        }
                        ret = true
                    }
                }
            }
        }
        for j in 0..self.board.size().1-3 {
            for i in 0..self.board.size().0 {
                let c1 = self.board[(i,j)].as_option().map(|x| x.color());
                let mut combo_len = 0;
                if c1.is_some() {
                    for j2 in j+1..self.board().size().1 {
                        if self.board[(i,j2)].as_option().map(|x| x.color()) == c1 {
                            combo_len += 1
                        } else {
                            break
                        }
                    }
                    if combo_len >= 3 {
                        for j2 in j..=j+combo_len {
                            if !readonly { self.clear_square((i,j2)); }
                        }
                        ret = true
                    }
                }
            }
        }
        return ret        
    } 
    pub fn deposit_score(&mut self) {
        let mut d = 10;
        while self.killed_viruses > 0 {
            self.points += d;
            if d >= 320 { d = 320; } else {
                d = d * 2;
            }
            self.killed_viruses -= 1;
        }
    }
    pub fn current_level(&self) -> u32 {
        self.level
    }
    pub fn score(&self) -> u32 {
        self.points
    }
    pub fn board(&self) -> &Imprint<FieldTile > {
        &self.board
    }
    pub fn next(&self) -> Option<Imprint<FieldTile >> {
        match self.status {
            Status::Menu(_) => None,
            _ => Some(self.next.imprint()),
        }
    }
    pub fn tick(&mut self) -> TickResult {
        match self.status {
            Status::Infecting(n,t) => {
                if t == 0 {
                    if n == 0 {
                        self.status = Status::Active
                    } else {
                        self.status = Status::Infecting(n-1,2);
                        self.place_virus()
                    }
                } else {
                    self.status = Status::Infecting(n, t-1)
                }
            }
            Status::Falling(n,nextlevel) => {
                let y = self.board.size().1-1;
                if n == 0 {
                    if nextlevel {
                        for i in 0..WIDTH {
                            self.board[(i,y)] = Cell::Empty
                        }
                    }
                    if !self.update_gravity() {
                        //self.board = Imprint::empty(WIDTH, HEIGHT + BUFFER);
                        if nextlevel { 
                            self.status = Status::Infecting(self.level * 4 + 4,2);
                        } else {
                            self.status = Status::Reacting;
                        }
                    } else {
                        self.status = Status::Falling(2,nextlevel)
                    }
                } else {
                    self.status = Status::Falling(n-1, nextlevel)
                }                
            }
            Status::Clearing(n) => {
                if n == 0 {
                    self.clearing.clear();
                    if self.check_won_level() {
                        self.deposit_score();
                        self.level += 1;
                        self.status = Status::Falling(2, true);                        
                    } else {
                        self.status = Status::Falling(2, false);    
                    }                    
                } else {
                    self.status = Status::Clearing(n-1)
                }
            }
            Status::Reacting => {
                if self.check_combos(false) {
                    self.status = Status::Clearing(10);
                } else {
                    self.deposit_score();
                    self.status = Status::Active
                }
            },
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
                if self.input.right && self.config.speed != GameSpeed::Fast {
                    self.input.right = false;
                    if self.config.speed == GameSpeed::Medium { self.config.speed = GameSpeed::Fast };
                    if self.config.speed == GameSpeed::Slow { self.config.speed = GameSpeed::Medium };
                    self.new_game()
                }
                if self.input.left && self.config.speed != GameSpeed::Slow {
                    self.input.left = false;
                    if self.config.speed == GameSpeed::Medium { self.config.speed = GameSpeed::Slow };
                    if self.config.speed == GameSpeed::Fast { self.config.speed = GameSpeed::Medium };
                    self.new_game()
                }
                if self.input.drop {
                    self.input.drop = false;
                    self.status = Status::Active;
                }
                if self.input.up {
                    self.input.up = false;
                    if self.config.level < MAX_LEVEL - 1 {
                        self.config.level += 1;
                        self.new_game()
                    }
                }
                if self.input.down {
                    self.input.down = false;
                    if self.config.level > 0 {
                        self.config.level -= 1;
                        self.new_game()
                    }
                }
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
