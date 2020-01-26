extern crate tesserae;
extern crate sdl2;


use rand::Rng;
use tesserae::{*};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::gfx::framerate::FPSManager;
use sdl2::render::TextureCreator;
use sdl2::render::RenderTarget;
use sdl2::render::Canvas;
use utils::color::{*};
use sdl2::rect::{Rect};
use utils::menu::{*};
use rand::thread_rng;
use std::io::Cursor;
use utils::{*};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Direction { N, S, E, W, NW, SW, NE, SE }
impl Direction {
    fn move_point (&self, p: (i32,i32)) -> (i32,i32) {
        match *self {
            Self::NW => (p.0 - 1, p.1 - 1),
            Self::SW => (p.0 - 1, p.1 + 1),
            Self::NE => (p.0 + 1, p.1 - 1),
            Self::SE => (p.0 + 1, p.1 + 1),
            Self::N => (p.0,p.1-1),
            Self::S => (p.0,p.1+1),
            Self::E => (p.0+1,p.1),
            Self::W => (p.0-1,p.1),
        }
    }
    fn to_index(&self) -> usize {
        match *self {
            Self::E => 1,
            Self::N => 2,
            Self::W => 3,
            Self::S => 4,
            Self::NW => 5,
            Self::NE => 6,
            Self::SW => 7,
            Self::SE => 8,
        }

    }
    fn from_vector(p:(i32,i32)) -> Option<Direction> {
        match p {
            (0,y) if y < 0 => Some(Direction::N),
            (0,y) if y > 0 => Some(Direction::S),
            (x,0) if x > 0 => Some(Direction::E),
            (x,0) if x < 0 => Some(Direction::W),
            (x,y) if x < 0 && y < 0 => Some(Direction::NW),
            (x,y) if x < 0 && y > 0 => Some(Direction::SW),
            (x,y) if x > 0 && y < 0 => Some(Direction::NE),
            (x,y) if x > 0 && y > 0=> Some(Direction::SE),
            _ => None
        }

    }
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum DangerType { Cylon, DangerCylon, Garbage, EMP }

struct Game<'r> {
    player_gfx: OutlinedTile<'r>,
    explosion_gfx: OutlinedTile<'r>,
    cylon_gfx: OutlinedTile<'r>,
    danger_cylon_gfx: OutlinedTile<'r>,
    emp_gfx: OutlinedTile<'r>,
    cursor_gfx: [OutlinedTile<'r>;9],
    player: (i32,i32),
    dangers: Vec<((i32,i32),DangerType)>,
    render_dangers: Vec<((i32,i32),DangerType)>,
    moves: Vec<((i32,i32),(i32,i32),Option<DangerType>)>,
    wave_one: bool,
    tile_set: TileSet,
    safe_teleports: u32,
    emp_pulses: u32,
    won_level: bool,
    dead: bool,
    ticks: u32,
    difficulty: Difficulty
}


const WIDTH : u32 = 30;
const HEIGHT : u32 = 30;
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Difficulty {
    level_increment: u32,
    danger_start: u32,
    danger_multiplier: u32,
    danger_divider: u32,
    teleports_divider: u32,
    emps_divider: u32,
    max_safe_teles: u32,
    max_emp_pulses: u32,
    starting_emps: u32,
    starting_teles: u32,
}
const EASY : Difficulty = Difficulty {
    level_increment: 2,
    danger_start: 8,
    danger_multiplier: 1,
    danger_divider: 2,
    teleports_divider: 3,
    emps_divider: 2,
    max_safe_teles: 15,
    max_emp_pulses: 20,
    starting_emps: 7,
    starting_teles: 5
};
const MEDIUM : Difficulty = Difficulty {
    level_increment: 3,
    danger_start: 4,
    danger_multiplier: 1,
    danger_divider: 1,
    teleports_divider: 5,
    emps_divider: 3,
    max_safe_teles: 10,
    max_emp_pulses: 15,
    starting_emps:5,
    starting_teles: 2,
};
const HARD : Difficulty = Difficulty {
    level_increment: 4,
    danger_start: 2,
    danger_multiplier: 2,
    danger_divider: 1,
    teleports_divider: 8,
    emps_divider: 5,
    max_safe_teles: 6,
    max_emp_pulses: 10,
    starting_emps: 1,
    starting_teles: 0
};

impl <'r>Game<'r> {
    fn new<T>(texture_creator : &'r TextureCreator<T>) -> Game<'r> {
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
        let cursor_gfx = [
            OutlinedTile::new(188,WHITE,&tile_set,texture_creator),
            OutlinedTile::new(64,WHITE,&tile_set,texture_creator),
            OutlinedTile::new(65,WHITE,&tile_set,texture_creator),
            OutlinedTile::new(66,WHITE,&tile_set,texture_creator),
            OutlinedTile::new(67,WHITE,&tile_set,texture_creator),
            OutlinedTile::new(68,WHITE,&tile_set,texture_creator),
            OutlinedTile::new(69,WHITE,&tile_set,texture_creator),
            OutlinedTile::new(70,WHITE,&tile_set,texture_creator),
            OutlinedTile::new(71,WHITE,&tile_set,texture_creator),
        ];
        Game {
            player_gfx: OutlinedTile::new(73,YELLOW,&tile_set,texture_creator),
            cylon_gfx: OutlinedTile::new(148,NEUTRAL_GRAY,&tile_set,texture_creator),
            danger_cylon_gfx: OutlinedTile::new(150,TEAL,&tile_set,texture_creator),
            explosion_gfx: OutlinedTile::new(149,ORANGE,&tile_set,texture_creator),
            emp_gfx: OutlinedTile::new(77,BRIGHT_GREEN,&tile_set,texture_creator),
            tile_set: tile_set,
            player: (WIDTH as i32/2,HEIGHT as i32/2),
            dangers: Vec::new(),
            moves: Vec::new(),
            wave_one: false,
            render_dangers: Vec::new(),
            safe_teleports: 0,
            emp_pulses: 0,    
            won_level: false,
            dead: false,        
            cursor_gfx,
            ticks: 0,
            difficulty: MEDIUM
        }
    }
    fn collision(&self, x:i32,y:i32) -> bool {
        self.dangers.iter().any(|(p,_)| (x,y) == *p)
    }
    fn set_up_level(&mut self, level : u32) {
        self.won_level = false;
        self.dead = false;
        self.wave_one = false;        
        self.moves = Vec::new();
        self.dangers = Vec::new();
        self.player = (WIDTH as i32/2,HEIGHT as i32/2);
        for _ in 0..1+level*self.difficulty.level_increment {
            let (mut rx,mut ry) = (thread_rng().gen_range(0,WIDTH as i32), thread_rng().gen_range(0,HEIGHT as i32));
            while self.collision(rx,ry) || (rx,ry) == self.player  {
                rx = thread_rng().gen_range(0,WIDTH as i32);
                ry = thread_rng().gen_range(0,HEIGHT as i32);
            } 
            
            self.dangers.push(((rx,ry),DangerType::Cylon));
        }
        if level > self.difficulty.danger_start {
            for _ in 0..(level-self.difficulty.danger_start)*self.difficulty.danger_multiplier/self.difficulty.danger_divider {
                let (mut rx,mut ry) = (thread_rng().gen_range(0,WIDTH as i32), thread_rng().gen_range(0,HEIGHT as i32));
                while self.collision(rx,ry) || (rx,ry) == self.player  {
                    rx = thread_rng().gen_range(0,WIDTH as i32);
                    ry = thread_rng().gen_range(0,HEIGHT as i32);
                } 
                self.dangers.push(((rx,ry),DangerType::DangerCylon));
            }
        }
        self.safe_teleports += level / self.difficulty.teleports_divider;
        self.emp_pulses += level / self.difficulty.emps_divider;
        if self.safe_teleports > self.difficulty.max_safe_teles { self.safe_teleports = self.difficulty.max_safe_teles };
        if self.emp_pulses > self.difficulty.max_emp_pulses { self.emp_pulses = self.difficulty.max_emp_pulses };
        self.render_dangers = self.dangers.clone();
    }
    fn draw<T:RenderTarget>(&self, canvas : &mut Canvas<T>) {
        canvas.set_draw_color(rgba(238,238,236,255));
        canvas.clear();
        canvas.set_draw_color(rgba(218,218,216,255));
        for i in 0..WIDTH as i32 {
            for j in 0..HEIGHT as i32 {
                if i % 2 == j %2 {
                    canvas.fill_rect(Rect::new(i*10,j*10+17,10,10)).unwrap();
                }
            }
        }
        for (p,t) in &self.render_dangers {
            match t {
                DangerType::Cylon => &self.cylon_gfx,
                DangerType::DangerCylon => &self.danger_cylon_gfx,
                DangerType::Garbage => &self.explosion_gfx,
                DangerType::EMP => &self.emp_gfx,
            }.draw(canvas, (p.0*10, p.1*10+17));
        }
        if !self.wave_one  { self.player_gfx.draw(canvas,(self.player.0*10,self.player.1*10+17)) };
        if self.ticks > 0 {
            if self.moves.len() > 0 {
                for (sp,ep,v) in &self.moves {
                    let start = (sp.0 as i32 * 10, sp.1 as i32 * 10 + 17);
                    let end = (ep.0 as i32 * 10, ep.1 as i32 * 10 + 17);
                    let loc = (start.0 + ((end.0 - start.0)*(5 - self.ticks as i32))/5,
                               start.1 + ((end.1 - start.1)*(5 - self.ticks as i32))/5);
                    
                    match v {
                        Some(DangerType::Cylon) => &self.cylon_gfx,
                        Some(DangerType::DangerCylon) => &self.danger_cylon_gfx,
                        Some(DangerType::Garbage) => &self.explosion_gfx,
                        Some(DangerType::EMP) => &self.emp_gfx,
                        None => &self.player_gfx,
                    }.draw(canvas, loc);
                }
            }
        }
    }
    fn clamp_bounds(p:(i32,i32)) -> (i32,i32) {
        (p.0.max(0).min(WIDTH as i32 -1), p.1.max(0).min(HEIGHT as i32 - 1))
    }
    fn move_towards(p1:(i32,i32),p2:(i32,i32)) -> (i32,i32) {
        let (x,y) = p1;
        let (dx,dy) = p2;
        (if x < dx { x + 1 } else if x > dx {x - 1} else { x }, 
            if y < dy { y + 1 } else if y > dy {y - 1} else { y })
    }
    fn check_collisions(&mut self) {
        let mut new_dangers : Vec<((i32,i32),DangerType)> = Vec::new();
        self.won_level = true;
        for (p,t) in &self.dangers {
            if *t == DangerType::Cylon || *t == DangerType::DangerCylon {
                self.won_level = false;
            }
            if *p == self.player && *t != DangerType::EMP {
                self.dead = true;
            }
            if let Some(i) = new_dangers.iter().position(|(x,_)| *x == *p) {
                new_dangers[i].1 = DangerType::Garbage;                
            } else {
                new_dangers.push((*p,*t));
            }
        }
        self.dangers = new_dangers;
    }
    fn tick(&mut self) {
        if self.ticks > 0 {
            self.ticks -= 1;
            if self.ticks == 0 {
                self.moves = Vec::new();
                self.render_dangers = self.dangers.clone();
                if self.wave_one {
                    let pl = self.player;
                    for (p,t) in &mut self.dangers {
                        if *t == DangerType::DangerCylon {
                            let old = *p;
                            *p = Self::move_towards(*p,pl);
                            self.moves.push((old,*p,Some(DangerType::DangerCylon)));
                        }
                    }
                    self.render_dangers = self.dangers.clone().drain(..).filter(|(_,t)| *t != DangerType::DangerCylon).collect();
                    self.check_collisions();
                    self.dangers = self.dangers.drain(..).filter(|(_,t)| *t != DangerType::EMP).collect();
                    self.ticks = 5;
                    self.wave_one = false;
                }
            }
        }
    }
    fn new_game(&mut self, diff: Difficulty) {
        self.emp_pulses = diff.starting_emps;
        self.safe_teleports = diff.starting_teles;
        self.difficulty = diff;
        self.set_up_level(1);
    }
    fn dir_for_cursor(&self, ix : i32, iy : i32) -> Option<Direction> {
        let (x,y) = Self::clamp_bounds((ix  / 10, (iy - 17) / 10));
        let move_table = [Direction::W, Direction::NW, Direction::N, Direction:: NE, Direction::E, Direction::SE, Direction::S, Direction::SW];
        if (x,y) == self.player {
          return None
        }
        let idx = x - self.player.0;
        let idy = y - self.player.1;
        if idx.abs() < 2 && idy.abs() < 2 {
            return Direction::from_vector((idx,idy));
        }
        let dx : f64 = ix as f64 - (self.player.0 as f64 + 0.5) * 10.0;
        let dy : f64 = iy as f64 - 17.0 - (self.player.1 as f64 + 0.5) * 10.0;
      
        let angle : f64 = dy.atan2(dx);
        /* Note the adjustment we have to make (+9, not +8) because atan2's idea 
         * of octants and the ones we want are shifted by PI/8. */
        let octant = (((8.0 * angle / std::f64::consts::PI) + 9.0) as usize / 2) % 8;
        Some(move_table[octant])
    }
    fn make_move(&mut self, mov : Move) {
        if self.ticks > 0 { return }
       let old = self.player;
        match mov {
            Move::Move(d) => {
                self.player = Self::clamp_bounds(d.move_point(self.player));
            },
            Move::Stay => {},
            Move::Teleport(safe) => {
                if safe && self.safe_teleports == 0 { return };
                let (mut rx,mut ry) = (thread_rng().gen_range(0,WIDTH as i32), thread_rng().gen_range(0,HEIGHT as i32));
                while safe && (self.collision(rx,ry) || (rx,ry) == self.player)  {
                    rx = thread_rng().gen_range(0,WIDTH as i32);
                    ry = thread_rng().gen_range(0,HEIGHT as i32);
                }
                self.player = (rx,ry);
                if safe { self.safe_teleports -= 1};
            }
            Move::EMP => {
                if self.emp_pulses == 0 { return };
                let (x,y) = self.player;
                self.dangers.push((Self::clamp_bounds((x-1,y-1)),DangerType::EMP));
                self.dangers.push((Self::clamp_bounds((x,y-1)),DangerType::EMP));
                self.dangers.push((Self::clamp_bounds((x+1,y-1)),DangerType::EMP));
                self.dangers.push((Self::clamp_bounds((x-1,y)),DangerType::EMP));
                self.dangers.push((Self::clamp_bounds((x+1,y)),DangerType::EMP));
                self.dangers.push((Self::clamp_bounds((x-1,y+1)),DangerType::EMP));
                self.dangers.push((Self::clamp_bounds((x,y+1)),DangerType::EMP));
                self.dangers.push((Self::clamp_bounds((x+1,y+1)),DangerType::EMP));
                self.check_collisions();
                self.emp_pulses -= 1;
            },
        }
        self.moves.push((old,self.player,None));
        let pl = self.player;
        for (p,t) in &mut self.dangers {
            if *t == DangerType::Cylon || *t == DangerType::DangerCylon {
                let old = *p;
                *p = Self::move_towards(*p,pl);
                self.moves.push((old,*p,Some(*t)));
            }
        }
        self.render_dangers = self.dangers.clone().drain(..).filter(|(_,t)| *t != DangerType::DangerCylon && *t != DangerType::Cylon).collect();
        self.wave_one = true;
        self.check_collisions();
        self.ticks = 5;
    }
}

enum Splash {
    Level,
    Loss,
}

enum Move {
    Move(Direction),
    Stay,
    Teleport(bool),
    EMP
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.text_input().start();
    let window = video_subsystem.window("cylons", WIDTH*10, HEIGHT*10 + 16 + 17)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    let mut splash = Some(Splash::Level);
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(WIDTH*10, HEIGHT*10 + 16 + 17).unwrap();
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut game = Game::new(&texture_creator);
    let mut level = 1;
    game.set_up_level(level);
    let mut status_level = Graphic::blank(8,1).textured(&texture_creator);
    let mut status_teleports = Graphic::blank(13,1).textured(&texture_creator);
    let mut status_emps = Graphic::blank(7,1).textured(&texture_creator);
    let mut lose = Graphic::load_from(Cursor::new(&include_bytes!("../lose")[..])).unwrap().textured(&texture_creator);
    lose.update_texture(&game.tile_set);
    let mut level_splash = Graphic::load_from(Cursor::new(&include_bytes!("../level_splash")[..])).unwrap().textured(&texture_creator);
    level_splash.update_texture(&game.tile_set);
    let mut rate_limiter = FPSManager::new();
    rate_limiter.set_framerate(40).unwrap();
    let mut micro_mode = false;
    let mut difficulty : Difficulty = MEDIUM;
    let mut menu = MenuBar::new(WIDTH*10)
                    .add(Menu::new("GAME",80,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Easy", 352,Keycode::F1,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Medium", 353, Keycode::F2,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Hard", 354, Keycode::F3,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(64, &texture_creator,&game.tile_set))
                            .add(MenuItem::new("Quit", 363, Keycode::F12,&texture_creator,&game.tile_set)))
                    .add(Menu::new("ACTION",136,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Teleport", 52,Keycode::Num1,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Safe Teleport",53, Keycode::Num2,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("EMP", 54,Keycode::Num3,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(120,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Wait", 20,Keycode::S,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(120,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Restart",15, Keycode::N,&texture_creator,&game.tile_set)))
                    .add(Menu::new("VIEW",112,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Micro-mode",360, Keycode::F9, &texture_creator, &game.tile_set)));
    let mut cx = 0;
    let mut cy = 0;

    loop {
        if splash.is_none() { 
            if game.won_level {
                level += 1;
                game.set_up_level(level);
                splash = Some(Splash::Level);
            }
            if game.dead {
                level = 1;
                game.new_game(difficulty);
                splash = Some(Splash::Loss);
            }
        }
        if splash.is_none() {
            game.tick();
        }
        game.draw(&mut canvas);
        let c = game.dir_for_cursor(cx, cy).map_or(0, |x| x.to_index());
        canvas.set_draw_color(BLACK);
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*10 + 17,WIDTH*10,17)).unwrap();
        canvas.set_draw_color(rgba(176,179,172,255));
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*10 + 18,WIDTH*10,16)).unwrap();
        status_level.draw_rect(0,0,8,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_level.draw_text(&("Level ".to_owned() + &level.to_string()),&game.tile_set,0,0,BLACK,TRANSPARENT);
        status_emps.draw_rect(0,0,6,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_emps.draw_text(&(game.emp_pulses.to_string() + " EMPs"),&game.tile_set,0,0,BLACK,TRANSPARENT);
        status_teleports.draw_rect(0,0,10,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_teleports.draw_text(&(game.safe_teleports.to_string() + " Safe Ts"),&game.tile_set,0,0,BLACK,TRANSPARENT);
        status_level.update_texture(&game.tile_set);
        status_emps.update_texture(&game.tile_set);
        status_teleports.update_texture(&game.tile_set);
        status_level.draw(&mut canvas, (8, HEIGHT as i32 *10 + 22 ));
        status_emps.draw(&mut canvas, (WIDTH as i32 * 10 - 7 * 9 + if game.emp_pulses < 10 { 8 } else { 0 }, HEIGHT as i32 *10 + 22 ));
        status_teleports.draw(&mut canvas, (WIDTH as i32 * 5 - (10 * 4) + if game.safe_teleports < 10 { 4 } else { 0 }, HEIGHT as i32 *10 + 22 ));
        if let Some(s) = &splash {
            match *s {
                Splash::Level => { 
                    level_splash.draw(&mut canvas, (WIDTH as i32 * 5 - (11*4), HEIGHT as i32 * 5 - 16 + 17 ));
                    status_level.draw(&mut canvas, (WIDTH as i32 * 5 - (8 * 4) + if level < 10 { 4} else {0}, HEIGHT as i32 * 5 - 4 + 17));
                },
                Splash::Loss => lose.draw(&mut canvas, (WIDTH as i32 * 5 - (21*4), HEIGHT as i32 * 5 - (21*4) + 17 )),
            }
        }
        game.cursor_gfx[c].draw_enlarged(&mut canvas,(cx-10,cy-10));
        menu.draw(&mut canvas);
        
        canvas.present();        
        for event in event_pump.poll_iter() {
            let h = menu.handle_event(event.clone(), &mut event_subsystem);
            match event {
                _ if h => { sdl_context.mouse().show_cursor(true)},
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::F12), .. } => {
                    return
                },
                Event::KeyDown {..} if splash.is_some() => {
                    splash = None  
                }
                Event::MouseButtonUp {..} if splash.is_some() => {
                    splash = None
                }
                Event::KeyDown { keycode: Some(Keycode::N), .. } => {
                    level = 1;
                    game.new_game(difficulty);
                    splash = Some(Splash::Level);
                },
                Event::KeyDown { keycode: Some(Keycode::F1), .. } => {
                    level = 1;
                    difficulty = EASY;
                    game.new_game(difficulty);
                    splash = Some(Splash::Level);
                },
                Event::KeyDown { keycode: Some(Keycode::F2), .. } => {
                    level = 1;
                    difficulty = MEDIUM;
                    game.new_game(difficulty);
                    splash = Some(Splash::Level);
                },
                Event::KeyDown { keycode: Some(Keycode::F3), .. } => {
                    level = 1;
                    difficulty = HARD;
                    game.new_game(difficulty);
                    splash = Some(Splash::Level);
                },
                Event::KeyDown { keycode: Some(Keycode::Up), ..} |
                Event::KeyDown { keycode: Some(Keycode::W), ..}
                  => game.make_move(Move::Move(Direction::N)),
                Event::KeyDown { keycode: Some(Keycode::Down), ..} |
                Event::KeyDown { keycode: Some(Keycode::X), ..}
                  => game.make_move(Move::Move(Direction::S)),
                Event::KeyDown { keycode: Some(Keycode::Left), ..} |
                Event::KeyDown { keycode: Some(Keycode::A), ..}
                  => game.make_move(Move::Move(Direction::W)),
                Event::KeyDown { keycode: Some(Keycode::Right), ..} |
                Event::KeyDown { keycode: Some(Keycode::D), ..}
                  => game.make_move(Move::Move(Direction::E)),
                Event::KeyDown { keycode: Some(Keycode::Q), ..}
                  => game.make_move(Move::Move(Direction::NW)),
                Event::KeyDown { keycode: Some(Keycode::E), ..}
                  => game.make_move(Move::Move(Direction::NE)),
                Event::KeyDown { keycode: Some(Keycode::Z), ..}
                  => game.make_move(Move::Move(Direction::SW)),
                Event::KeyDown { keycode: Some(Keycode::C), ..}
                  => game.make_move(Move::Move(Direction::SE)),
                Event::KeyDown { keycode: Some(Keycode::S), ..}
                  => game.make_move(Move::Stay),
                Event::KeyDown { keycode: Some(Keycode::Num1), ..}
                  => game.make_move(Move::Teleport(false)),
                Event::KeyDown { keycode: Some(Keycode::Num2), ..}
                  => game.make_move(Move::Teleport(true)),
                Event::KeyDown { keycode: Some(Keycode::Num3), ..}
                  => game.make_move(Move::EMP),
                Event::MouseButtonUp {..} if cy > 17 => {
                    match game.dir_for_cursor(cx, cy) {
                        None => game.make_move(Move::Stay),
                        Some(d) => game.make_move(Move::Move(d)),
                    }
                }
                Event::KeyDown { keycode: Some(Keycode::F9), ..} => {
                    if micro_mode {
                        micro_mode = false;
                        canvas.window_mut().set_size(WIDTH*10, HEIGHT*10+16+17).unwrap_or_default();
                    } else {
                        micro_mode = true;
                        canvas.window_mut().set_size(WIDTH*5, (HEIGHT*10+16+17)/2).unwrap_or_default();
                    }
                },
                Event::MouseMotion { x,y,..} if y < 17 => {
                    cx = x;
                    cy = y;
                    sdl_context.mouse().show_cursor(true);
                }
                Event::MouseMotion { x,y,..} => {
                    cx = x;
                    cy = y;
                    sdl_context.mouse().show_cursor(false);
                }
                _ => {},
            }
        }
        rate_limiter.delay();
    }
}
