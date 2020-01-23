extern crate tesserae;
extern crate sdl2;


use rand::Rng;
use tesserae::{*};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::gfx::framerate::FPSManager;
use sdl2::render::TextureCreator;
use sdl2::render::RenderTarget;
use sdl2::render::Texture;
use sdl2::render::Canvas;
use utils::color::{*};
use sdl2::rect::{Rect};
use utils::menu::{*};
use rand::thread_rng;
use std::io::Cursor;
use std::collections::VecDeque;
use utils::{*};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Direction { N, S, E, W, NW, SW, NE, SE }
impl Direction {
    const ALL : [Direction;8] = [Self::N, Self::S, Self::E, Self::W, Self::NW, Self::SW, Self::NE, Self::SE ];
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
enum GameObject { Cat(bool, usize), Cheese, PushBox, Block, Sink, Trap }


struct Game<'r> {
    player: (i32,i32),
    cats: Vec<(i32,i32)>,
    tile_set: TileSet,
    ticks: u32,
    tiles: [[Option<GameObject>;HEIGHT as usize];WIDTH as usize],
    happy_cat: OutlinedTile<'r>,
    sad_cat: OutlinedTile<'r>,
    dying_cat: OutlinedTile<'r>,
    cheese: OutlinedTile<'r>,
    sink: OutlinedTile<'r>,
    trap: OutlinedTile<'r>,
    player_gfx: OutlinedTile<'r>,
    player_gfx_trapped: OutlinedTile<'r>,
    player_gfx_dying: OutlinedTile<'r>,
    block: Graphic<Texture<'r>>,
    push_box: Graphic<Texture<'r>>,
    cursor_gfx: [OutlinedTile<'r>;9],
    speed:u32,
    won_level: bool,
    turbo: bool,
    sunk_ticks:u32,
    dying_ticks:u32,
    dead: bool,
    remaining_cats: u32,
    lives: u32,
    cheeses: u32,
    recently_got_cheese: u32,
    score: u32,
    cat_timer: u32,
}


const WIDTH : u32 = 23;
const HEIGHT : u32 = 23;

fn load_from_graphic(gfx: Graphic<()>) -> [[Option<GameObject>;HEIGHT as usize];WIDTH as usize] {
    let mut ret = [[None;HEIGHT as usize];WIDTH as usize];
    for i in 0..WIDTH {
        for j in 0..HEIGHT {
            ret[i as usize][j as usize] = match gfx[(i,j)].index {
                1 => Some(GameObject::Block),
                2 => Some(GameObject::PushBox),
                3 => Some(GameObject::Sink),
                6 => Some(GameObject::Cheese),
                7 => Some(GameObject::Trap),
                _ => None,
            };
        }
    }
    ret
}
struct Level {
    map: [[Option<GameObject>;HEIGHT as usize];WIDTH as usize],
    random_blocks: u32,
    random_traps: u32,
    random_sinks: u32,
    random_boxes: u32,
}

fn level(index: u32) -> Level {
    match index {
        1 => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level1")[..])).unwrap()),
                random_blocks: 0,
                random_sinks: 0,
                random_traps: 0,
                random_boxes: 0,
        },
        2 => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level1")[..])).unwrap()),
                random_blocks: 5,
                random_sinks: 0,
                random_traps: 0,
                random_boxes: 0,
        },
        3 => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level1")[..])).unwrap()),
                random_blocks: 7,
                random_sinks: 3,
                random_traps: 0,
                random_boxes: 0,
        },
        4 => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level1")[..])).unwrap()),
                random_blocks: 7,
                random_sinks: 3,
                random_traps: 3,
                random_boxes: 0,
        },
        5 => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level2")[..])).unwrap()),
                random_blocks: 2,
                random_sinks: 0,
                random_traps: 0,
                random_boxes: 0,
        },
        6 => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level2")[..])).unwrap()),
                random_blocks: 8,
                random_sinks: 1,
                random_traps: 0,
                random_boxes: 0,
        },
        7 => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level2")[..])).unwrap()),
                random_blocks: 8,
                random_sinks: 3,
                random_traps: 3,
                random_boxes: 0,
        },
        8 => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level3")[..])).unwrap()),
                random_blocks: 6,
                random_sinks: 3,
                random_traps: 1,
                random_boxes: 15 * 10,
        },
        9 => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level3")[..])).unwrap()),
                random_blocks: 10,
                random_sinks: 8,
                random_traps: 5,
                random_boxes: 15 * 10,
        },
        _ => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level3")[..])).unwrap()),
                random_blocks: 20,
                random_sinks: 8,
                random_traps: 8,
                random_boxes: 10 * 10,
        },
    }
}


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
        let mut block = Graphic::load_from(Cursor::new(&include_bytes!("../garbage_block")[..])).unwrap().textured(&texture_creator);
        block.update_texture(&tile_set);
        let mut push_box = Graphic::load_from(Cursor::new(&include_bytes!("../movable_block")[..])).unwrap().textured(&texture_creator);
        push_box.update_texture(&tile_set);
        let cheese =  OutlinedTile::new(164,YELLOW,&tile_set,texture_creator);
        let happy_cat = OutlinedTile::new(151,ORANGE,&tile_set,texture_creator);
        let sad_cat = OutlinedTile::new(152,ORANGE,&tile_set,texture_creator);
        let dying_cat = OutlinedTile::new(152,PALE_ORANGE,&tile_set,texture_creator);
        let player_gfx=  OutlinedTile::new(153,NEUTRAL_GRAY,&tile_set,texture_creator);
        let player_gfx_trapped=  OutlinedTile::new(153,DARKER_GRAY,&tile_set,texture_creator);
        let player_gfx_dying =  OutlinedTile::new(153,WHITE,&tile_set,texture_creator);
        let sink = OutlinedTile::new(166,CHARCOAL,&tile_set,texture_creator);
        let trap = OutlinedTile::new(165,LIGHT_BROWN,&tile_set,texture_creator);
        Game {
            tile_set: tile_set,
            player: (WIDTH as i32/2,HEIGHT as i32/2),
            won_level: false,
            dead: false,        
            ticks: 0,
            cats: Vec::new(),
            tiles: [[None;HEIGHT as usize]; WIDTH as usize],
            lives: 3,
            cheeses: 0,
            remaining_cats: 9,
            score: 0,
            sunk_ticks: 0,
            dying_ticks: 0,
            cat_timer: 60,
            speed:50,
            turbo: false,
            recently_got_cheese: 0, dying_cat,
            block, push_box, player_gfx_trapped,player_gfx_dying,
            cursor_gfx, cheese,happy_cat,sad_cat,player_gfx,sink,trap
        }
    }
    fn set_up_level(&mut self, lev : u32) {
        let mut l = level(lev);
        self.player = (WIDTH as i32/2,HEIGHT as i32/2);
        for _ in 0..l.random_blocks {
            let mut x = self.player.0;
            let mut y = self.player.1;
            while (x,y) == self.player {
                x = thread_rng().gen_range(1,WIDTH as i32-1);
                y = thread_rng().gen_range(1,HEIGHT as i32-1);
            }
            l.map[x as usize][y as usize] = Some(GameObject::Block);
        }
        for _ in 0..l.random_sinks {
            let mut x = self.player.0;
            let mut y = self.player.1;
            while (x,y) == self.player {
                x = thread_rng().gen_range(1,WIDTH as i32-1);
                y = thread_rng().gen_range(1,HEIGHT as i32-1);
            }
            l.map[x as usize][y as usize] = Some(GameObject::Sink);
        }
        for _ in 0..l.random_traps {
            let mut x = self.player.0;
            let mut y = self.player.1;
            while (x,y) == self.player {
                x = thread_rng().gen_range(1,WIDTH as i32-1);
                y = thread_rng().gen_range(1,HEIGHT as i32-1);
            }
            l.map[x as usize][y as usize] = Some(GameObject::Trap);
        }
        for _ in 0..l.random_boxes {
            let mut x = self.player.0;
            let mut y = self.player.1;
            while (x,y) == self.player {
                x = thread_rng().gen_range(1,WIDTH as i32-1);
                y = thread_rng().gen_range(1,HEIGHT as i32-1);
            }
            l.map[x as usize][y as usize] = Some(GameObject::PushBox);
        }
        self.won_level = false;
        self.dead = false;
        self.turbo = false;
        self.tiles = l.map;
        self.cats = Vec::new();
        self.remaining_cats = 9;
        self.sunk_ticks = 0;
        self.dying_ticks = 0;
        self.ticks = 0;
        
        self.cat_timer = 60;
        self.recently_got_cheese = 0;
        self.deposit_cat();
    }
    fn is_blank_or_cat(i : Option<GameObject>) -> bool {
        match i {
            Some(GameObject::Cat(_,_)) => true, 
            None => true,
            _ => false
        }
    }
    fn deposit_cat(&mut self) {
        if self.remaining_cats == 0 { return };
        self.remaining_cats -= 1;
        let cs = self.cost_map();
        let p = self.player;
        let mut candidates : Vec<(i32,i32)> = Vec::new();
        for _ in 0..5 {
            let mut x = thread_rng().gen_range(0,WIDTH as i32);
            let mut y = thread_rng().gen_range(0,HEIGHT as i32);
            while self.tiles[x as usize][y as usize].is_some() {
                x = thread_rng().gen_range(0,WIDTH as i32);
                y = thread_rng().gen_range(0,HEIGHT as i32);
            }
            candidates.push((x,y));
        }
        candidates.sort_by(|a, b| Self::move_heuristic(*b,p,cs).cmp(&Self::move_heuristic(*a,p,cs)) );
        let c = candidates.first().unwrap();
        let i = self.cats.len();
        self.cats.push(*c);
        self.tiles[c.0 as usize][c.1 as usize] = Some(GameObject::Cat(true,i));
    }
    fn cost_map(&self) -> [[usize;HEIGHT as usize]; WIDTH as usize] {
        let mut ret = [[std::usize::MAX; HEIGHT as usize];WIDTH as usize];
        ret[self.player.0 as usize][self.player.1 as usize] = 0;
        let mut q : VecDeque<(i32,i32)> = VecDeque::new();
        q.push_back(self.player);
        while let Some((x,y)) = q.pop_front() {
            let c = ret[x as usize][y as usize] + 1;
            for d in Direction::ALL.iter() {
                let p = d.move_point((x,y));
                if Self::clamp_bounds(p) == p && Self::is_blank_or_cat(self.tiles[p.0 as usize][p.1 as usize]) {
                    if ret[p.0 as usize][p.1 as usize] > c {
                        ret[p.0 as usize][p.1 as usize] = c;
                        q.push_back(p);
                    }
                }
            }
        }
        ret
    }
    fn draw<T:RenderTarget>(&self, canvas : &mut Canvas<T>) {
        canvas.set_draw_color(rgba(238,238,236,255));
        canvas.clear();
        canvas.set_draw_color(rgba(218,218,216,255));
        for i in 0..WIDTH as i32 {
            for j in 0..HEIGHT as i32 {
                if i % 2 == j %2 {
                    canvas.fill_rect(Rect::new(i*12+1,j*12+17+1,10,10)).unwrap();
                }
                if let Some(x) = self.tiles[i as usize][j as usize] {
                    match x {
                        GameObject::Block => self.block.draw(canvas,(i*12-6,j*12-6+17)),
                        GameObject::PushBox => self.push_box.draw(canvas,(i*12-6,j*12-6+17)),
                        GameObject::Cat(true,_) => self.happy_cat.draw(canvas,(i*12+1,j*12+1+17)),
                        GameObject::Cat(false,_) => if self.turbo && self.ticks / 3 % 2 == 1 {
                            self.dying_cat.draw(canvas,(i*12+1,j*12+1+17)) 
                        } else {
                            self.sad_cat.draw(canvas,(i*12+1,j*12+1+17)) 
                        },
                        GameObject::Cheese => self.cheese.draw(canvas,(i*12+1,j*12+1+17)),
                        GameObject::Trap => self.trap.draw(canvas,(i*12+1,j*12+1+17)),
                        GameObject::Sink => self.sink.draw(canvas,(i*12+1,j*12+1+17)),
                    }
                }
            }
        }
        if self.dying_ticks > 0 {
            match (self.ticks /3) % 4 {
                0 => &self.player_gfx_dying,
                1 => &self.player_gfx,
                2 => &self.player_gfx_trapped,
                _ => &self.player_gfx,
            }.draw(canvas,(self.player.0*12+1,self.player.1*12+17+1));
        } else { 
            if self.sunk_ticks > 0 && (self.ticks /3) % 2 == 0 {
                self.player_gfx_trapped.draw(canvas,(self.player.0*12+1,self.player.1*12+17+1));
            } else {
                self.player_gfx.draw(canvas,(self.player.0*12+1,self.player.1*12+17+1));
            }
        }
    }
    fn push(&mut self, p:(i32,i32), tile: Option<GameObject>, d : Direction) -> Option<Option<GameObject>> {
        if Self::clamp_bounds(d.move_point(p)) != d.move_point(p) { return None };
        let (x,y) = d.move_point(p);
        match self.tiles[x as usize][y as usize] {
            None | Some(GameObject::Cheese) => { 
                let ret = self.tiles[x as usize][y as usize];
                self.tiles[x as usize][y as usize] = tile;
                return Some(ret);
            },
            Some(GameObject::Sink) => return Some(Some(GameObject::Sink)),
            Some(GameObject::Trap) => if tile.is_none() { return Some(Some(GameObject::Trap)) } else { return None },
            Some(GameObject::Cat(bo,i)) => {
                if let Some(d) = self.move_for_cat((x,y),self.cost_map()) {
                    let nc = d.move_point((x,y));
                    self.tiles[nc.0 as usize][nc.1 as usize] = Some(GameObject::Cat(true,i));
                    self.tiles[x as usize][y as usize] = tile;
                    self.cats[i] = nc;
                    return Some(Some(GameObject::Cat(bo,i)));
                } else {
                    self.tiles[x as usize][y as usize] = Some(GameObject::Cat(false,i));
                    return None;
                }
            },
            Some(GameObject::PushBox) => {
                if self.push((x,y),Some(GameObject::PushBox),d).is_some() {
                    self.tiles[x as usize][y as usize] = tile;
                    return Some(Some(GameObject::PushBox));
                } else {
                    return None;
                }
            },
            _ => return None ,
        }
    }
    fn clamp_bounds(p:(i32,i32)) -> (i32,i32) {
        (p.0.max(0).min(WIDTH as i32 -1), p.1.max(0).min(HEIGHT as i32 - 1))
    }
    fn move_heuristic( p : (i32, i32), player: (i32,i32), costs: [[usize;HEIGHT as usize]; WIDTH as usize]) -> (usize,i32) {
        let (px,py) = player;
        (costs[p.0 as usize][p.1 as usize], (p.0 - px).abs() + (p.1 - py).abs())
    }
    fn move_for_cat(&self, c : (i32,i32),  cs: [[usize;HEIGHT as usize]; WIDTH as usize] ) -> Option<Direction> {
        let mut moves : Vec<Direction> = Direction::ALL.iter().filter(|x| {
            let p = x.move_point(c);
            Self::clamp_bounds(p) == p && self.tiles[p.0 as usize][p.1 as usize].is_none()
        }).cloned().collect();
        if moves.len() == 0 {
            return None;
        } else {
            let player = self.player;
            moves.sort_by(|d1, d2| {
                let p = d1.move_point(c);
                let q = d2.move_point(c);
                Self::move_heuristic(p,player,cs).cmp(&Self::move_heuristic(q,player,cs))
            });
            let best_heuristic = Self::move_heuristic(moves.first().unwrap().move_point(c),player,cs);
            let best_moves : Vec<Direction> = moves.into_iter().take_while(|x| Self::move_heuristic(x.move_point(c), player,cs) == best_heuristic ).collect();
            let x = thread_rng().gen_range(0,best_moves.len());
            return Some(best_moves[x]);
        }

    }
    fn all_cats_to_cheese(&mut self) {
        for c in self.cats.clone() {
            self.tiles[c.0 as usize][c.1 as usize] = Some(GameObject::Cheese);
        }
        self.cats = Vec::new();
    }
    fn move_cats(&mut self) {
        let cs = self.cost_map();
        let mut new_cats = Vec::new();
        let mut i = 0;
        for c in self.cats.clone() {
            if let Some(x) = self.move_for_cat(c,cs) {
                let nc = x.move_point(c);
                self.tiles[c.0 as usize][c.1 as usize] = None;
                self.tiles[nc.0 as usize][nc.1 as usize] = Some(GameObject::Cat(true,i));
                new_cats.push(nc);
            } else {
                self.tiles[c.0 as usize][c.1 as usize] = Some(GameObject::Cat(false,i));
                new_cats.push(c);
            }
            i += 1;
        }
        self.cats = new_cats;
        if self.cats.iter().any(|x| *x == self.player ) {
            self.dying_ticks = 2;
        }
    }
    fn check_cats( &self) -> bool {
        for (x,y) in &self.cats {
            match self.tiles[*x as usize][*y as usize] {
                Some(GameObject::Cat(c,_)) => if c { return false },
                _ => {},
            }
        }
        true
    }
    fn tick(&mut self) {
        if self.ticks == 0 {
            if self.recently_got_cheese > 0 { self.recently_got_cheese -= 1 };
            if self.dying_ticks == 0  && !self.turbo {
                if self.cat_timer == 0 && self.remaining_cats > 0 {
                    self.deposit_cat();
                    self.deposit_cat();
                    if self.remaining_cats > 0 {
                        self.cat_timer = 60;
                    }
                } else if self.cat_timer > 0 { 
                    self.cat_timer -= 1; 
                }
                self.move_cats();
                if self.check_cats() {
                    self.turbo = true;
                    self.ticks = 60;
                }
                if self.sunk_ticks > 0 { 
                    self.sunk_ticks -= 1;
                    if self.sunk_ticks == 0 {
                        self.tiles[self.player.0 as usize][self.player.1 as usize] = None;
                    }
                };
            } else if self.dying_ticks > 0 {
                self.dying_ticks -= 1;
                if self.dying_ticks == 0 {
                    if self.lives == 0 {                                                                        
                        self.dead = true;
                    } else {
                        self.sunk_ticks = 0;
                        if self.tiles[self.player.0 as usize][self.player.1 as usize] == Some(GameObject::Trap) {
                            self.tiles[self.player.0 as usize][self.player.1 as usize] = None;
                        }
                        self.lives -= 1;
                        self.recently_got_cheese = 0;
                        self.turbo = false;
                        let mut x = thread_rng().gen_range(0,WIDTH as i32);
                        let mut y = thread_rng().gen_range(0,HEIGHT as i32);
                        while self.tiles[x as usize][y as usize].is_some() {
                            x = thread_rng().gen_range(0,WIDTH as i32);
                            y = thread_rng().gen_range(0,HEIGHT as i32);
                        }
                        self.player = (x,y);
                    }
                }
            } else if self.cat_timer > 0 {
                self.cat_timer -= 1;
                self.score += 1;
                if self.cat_timer == 0 {
                    self.turbo = false;
                    self.all_cats_to_cheese();
                }
            } else {
                self.won_level = true;
            }

            if self.turbo { self.ticks += 3 } else { self.ticks += self.speed }; 
        } else { 
            self.ticks -= 1;
        }
    }
    fn new_game(&mut self) {
        self.lives = 3;
        self.cheeses = 0;
        self.score = 0;
        
        self.set_up_level(1);
    }
    fn dir_for_cursor(&self, ix : i32, iy : i32) -> Option<Direction> {
        let (x,y) = Self::clamp_bounds((ix  / 12, (iy - 17) / 12));
        let move_table = [Direction::W, Direction::NW, Direction::N, Direction:: NE, Direction::E, Direction::SE, Direction::S, Direction::SW];
        if (x,y) == self.player {
          return None
        }
        let idx = x - self.player.0;
        let idy = y - self.player.1;
        if idx.abs() < 2 && idy.abs() < 2 {
            return Direction::from_vector((idx,idy));
        }
        let dx : f64 = ix as f64 - (self.player.0 as f64 + 0.5) * 12.0;
        let dy : f64 = iy as f64 - 17.0 - (self.player.1 as f64 + 0.5) * 12.0;
      
        let angle : f64 = dy.atan2(dx);
        /* Note the adjustment we have to make (+9, not +8) because atan2's idea 
         * of octants and the ones we want are shifted by PI/8. */
        let octant = (((8.0 * angle / std::f64::consts::PI) + 9.0) as usize / 2) % 8;
        Some(move_table[octant])
    }
    fn make_move(&mut self, mov : Move) {
        if self.sunk_ticks > 0  || self.dying_ticks > 0 || self.turbo { return };
        match mov {
            Move::Move(d) => match self.push(self.player,None,d) { 
                None => {},
                Some(t) => { 
                    self.player = Self::clamp_bounds(d.move_point(self.player)); 
                    match t {
                        Some(GameObject::Sink) => self.sunk_ticks = 5,
                        Some(GameObject::Trap) => self.dying_ticks = 2,
                        Some(GameObject::Cheese) => { self.cheeses += 1; if self.cheeses % 3 == 0 { self.lives += 1 }; self.recently_got_cheese = 2; self.score += 100; },
                        _ => {},
                    }
                },
            },
            Move::Stay => {},
        }
    }
}

enum Splash {
    Level,
    Loss,
}

enum Move {
    Move(Direction),
    Stay,
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.text_input().start();
    let window = video_subsystem.window("vermin's vengeance", WIDTH*12, HEIGHT*12 + 16 + 17 + 17)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    let mut splash = Some(Splash::Level);
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(WIDTH*12, HEIGHT*12 + 16 + 17+17).unwrap();
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut game = Game::new(&texture_creator);
    let mut level = 1;
    game.set_up_level(level);
    let mut status : String = "".to_string();
    let mut status_color = ORANGE;
    let mut status_level = Graphic::blank(8,1).textured(&texture_creator);
    let mut status_score = Graphic::blank(7,1).textured(&texture_creator);
    let mut status_lives = Graphic::blank(2,1).textured(&texture_creator);
    let mut status_cheese = Graphic::blank(2,1).textured(&texture_creator);
    let mut status_status = Graphic::blank(16,1).textured(&texture_creator);
    let mut lose = Graphic::load_from(Cursor::new(&include_bytes!("../lose")[..])).unwrap().textured(&texture_creator);
    lose.update_texture(&game.tile_set);
    let mut level_splash = Graphic::load_from(Cursor::new(&include_bytes!("../level_splash")[..])).unwrap().textured(&texture_creator);
    level_splash.update_texture(&game.tile_set);
    let mut rate_limiter = FPSManager::new();    
    rate_limiter.set_framerate(40).unwrap();
    let mut micro_mode = false;    
    let mut menu = MenuBar::new(WIDTH*12)
                    .add(Menu::new("GAME",120,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Restart",15, Keycode::N,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Skip Level",357, Keycode::F6,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(104, &texture_creator,&game.tile_set))
                            .add(MenuItem::new("Quit", 363, Keycode::F12,&texture_creator,&game.tile_set)))
                    .add(Menu::new("SPEED",88,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Slow", 352,Keycode::F1,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Medium", 353, Keycode::F2,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Fast", 354, Keycode::F3,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Andale!", 355, Keycode::F4,&texture_creator,&game.tile_set)))
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
                game.new_game();
                splash = Some(Splash::Loss);
            }
        }
        if splash.is_none() {
            game.tick();
            if game.dying_ticks > 0 || game.dead {
                if game.tiles[game.player.0 as usize][game.player.1 as usize] == Some(GameObject::Trap) {
                    status = "caught in a trap!".to_string();
                } else {
                    status = "cat got you!".to_string();
                }
                status_color = ORANGE;
            } else if game.sunk_ticks > 0 {
                status = "stuck! (".to_string() + &game.sunk_ticks.to_string() + ")";
                status_color = PALE_BROWN;
            } else if game.recently_got_cheese > 0 {
                status = "got cheese!".to_string();
                status_color = YELLOW;
            } else if game.turbo {
                status = "well done! (".to_string() + &game.cat_timer.to_string() + ")";
                status_color = BRIGHT_GREEN;
            } else if game.cat_timer > 0 {
                status = "more cats in ".to_string() + &game.cat_timer.to_string();
                status_color = NEUTRAL_GRAY;
            } else if game.remaining_cats == 0 {
                status = "no further cats!".to_string();
                status_color = BRIGHT_GREEN;
            } else {
                status = "".to_string();
            }
        }

        game.draw(&mut canvas);
        let c = game.dir_for_cursor(cx, cy).map_or(0, |x| x.to_index());
        canvas.set_draw_color(BLACK);
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*12 + 17,WIDTH*12,40)).unwrap();
        canvas.set_draw_color(rgba(85,87,83,255));
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*12 + 18,WIDTH*12,16)).unwrap();
        canvas.set_draw_color(rgba(176,179,172,255));
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*12 + 18 + 17,WIDTH*12,16)).unwrap();
        status_status.draw_rect(0,0,16,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_status.draw_text(&status,&game.tile_set,0,0,status_color,TRANSPARENT);
        status_status.update_texture(&game.tile_set);
        status_level.draw_rect(0,0,8,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_level.draw_text(&("Level ".to_owned() + &level.to_string()),&game.tile_set,0,0,BLACK,TRANSPARENT);
        status_level.update_texture(&game.tile_set);
        game.player_gfx.draw(&mut canvas, (8, HEIGHT as i32 * 12 + 17 +5));
        game.cheese.draw(&mut canvas, (WIDTH as i32 * 12 - 16, HEIGHT as i32 * 12 + 17 + 4));
        status_lives.draw_rect(0,0,2,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_lives.draw_text(&game.lives.to_string(),&game.tile_set,0,0,WHITE,TRANSPARENT);
        status_lives.update_texture(&game.tile_set);
        status_cheese.draw_rect(0,0,2,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_cheese.draw_text(&game.cheeses.to_string(),&game.tile_set,0,0,WHITE,TRANSPARENT);
        status_cheese.update_texture(&game.tile_set);
        status_score.draw_rect(0,0,2,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_score.draw_text(&game.score.to_string(),&game.tile_set,0,0,BLACK,TRANSPARENT);
        status_score.update_texture(&game.tile_set);
        status_level.draw(&mut canvas, (8, HEIGHT as i32 * 12 + 17+ 22 ));
        status_lives.draw(&mut canvas, (22, HEIGHT as i32 * 12 + 23 ));
        status_cheese.draw(&mut canvas, (WIDTH as i32 * 12 - 16 - game.cheeses.to_string().len() as i32 * 8 - 4, HEIGHT as i32 * 12 + 23 ));
        status_score.draw(&mut canvas, (WIDTH as i32 * 12 - 8 - game.score.to_string().len() as i32 * 8, HEIGHT as i32 * 12 + 17+22 ));
        status_status.draw(&mut canvas, (WIDTH as i32 * 6 - 4 * status.len() as i32, HEIGHT as i32 * 12 + 23));
        if let Some(s) = &splash {
            match *s {
                Splash::Level => { 
                    level_splash.draw(&mut canvas, (WIDTH as i32 * 6 - (11*4), HEIGHT as i32 * 6 - 16 + 17 ));
                    status_level.draw(&mut canvas, (WIDTH as i32 * 6 - (8 * 4) + if level < 10 { 4} else {0}, HEIGHT as i32 * 6 - 4 + 17));
                },
                Splash::Loss => lose.draw(&mut canvas, (WIDTH as i32 * 6 - (21*4), HEIGHT as i32 * 6 - (21*4) + 17 )),
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
                    game.new_game();
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
                Event::KeyDown { keycode: Some(Keycode::F1), ..}
                  => game.speed = 80,
                Event::KeyDown { keycode: Some(Keycode::F2), ..}
                  => game.speed = 50,
                Event::KeyDown { keycode: Some(Keycode::F3), ..}
                  => game.speed = 20,
                Event::KeyDown { keycode: Some(Keycode::F4), ..}
                  => game.speed = 10,
                Event::KeyDown { keycode: Some(Keycode::F6), ..}
                  => game.won_level = true,
                Event::MouseButtonUp {..} if cy > 17 => {
                    match game.dir_for_cursor(cx, cy) {
                        None => game.make_move(Move::Stay),
                        Some(d) => game.make_move(Move::Move(d)),
                    }
                }
                Event::KeyDown { keycode: Some(Keycode::F9), ..} => {
                    if micro_mode {
                        micro_mode = false;
                        canvas.window_mut().set_size(WIDTH*12, HEIGHT*12+16+17 +17).unwrap_or_default();
                    } else {
                        micro_mode = true;
                        canvas.window_mut().set_size(WIDTH*6, (HEIGHT*12+16+17+17)/2).unwrap_or_default();
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

