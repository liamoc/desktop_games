extern crate tesserae;
extern crate sdl2;


use rand::Rng;
use tesserae::{*};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use utils::framerate::FPSManager;
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
enum Direction { N, S, E, W }
impl Direction {
    fn move_point (&self, p: (i32,i32)) -> (i32,i32) {
        match *self {
            Self::N => (p.0,p.1-1),
            Self::S => (p.0,p.1+1),
            Self::E => (p.0+1,p.1),
            Self::W => (p.0-1,p.1),
        }
    }
    fn right(&self) -> Direction {
        match *self {
            Self::E => Self::S,
            Self::S => Self::W,
            Self::W => Self::N,
            Self::N => Self::E,
        }
    }
    fn left(&self) -> Direction {
        match *self {
            Self::E => Self::N,
            Self::N => Self::W,
            Self::W => Self::S,
            Self::S => Self::E,
        }
    }
    fn opposite(&self) -> Direction {
        match *self {
            Self::E => Self::W,
            Self::N => Self::S,
            Self::W => Self::E,
            Self::S => Self::N,
        }
    }
    fn to_index(&self) -> usize {
        match *self {
            Self::E => 1,
            Self::N => 2,
            Self::W => 3,
            Self::S => 4,
        }
    }
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum GameObject { LeftFrom(Direction), RightFrom(Direction), Straight(Direction), Apple, GoldenApple, Block }
impl GameObject {
    fn direction(&self) -> Option<Direction> {
        match *self {
            Self::LeftFrom(d) => Some(d),
            Self::RightFrom(d) => Some(d),
            Self::Straight(d) => Some(d),
            _ => None,
        }
    }
}

struct Game<'r> {
    player: (i32,i32),
    tail: VecDeque<(i32,i32)>,
    bonus: Option<(i32,i32)>,
    direction: Direction,
    approach_direction: Direction,
    tile_set: TileSet,
    ticks: u32,
    tiles: [[Option<GameObject>;HEIGHT as usize];WIDTH as usize],
    apple: OutlinedTile<'r>,
    golden_apple: OutlinedTile<'r>,
    left_turn_gfx: [Graphic<Texture<'r>>;4],
    right_turn_gfx: [Graphic<Texture<'r>>;4],
    straight_gfx: [Graphic<Texture<'r>>;4],
    head_gfx : [Graphic<Texture<'r>>;4],
    tail_gfx: [Graphic<Texture<'r>>;4],
    block: Graphic<Texture<'r>>,
    cursor_gfx: [OutlinedTile<'r>;5],
    relative_controls: bool,
    speed:u32,
    dying_ticks:u32,
    growth: u32,
    dead: bool,
    score: u32,
    bonus_timer: u32,
}


const WIDTH : u32 = 23;
const HEIGHT : u32 = 23;

fn load_from_graphic(gfx: Graphic<()>) -> [[Option<GameObject>;HEIGHT as usize];WIDTH as usize] {
    let mut ret = [[None;HEIGHT as usize];WIDTH as usize];
    for i in 0..WIDTH {
        for j in 0..HEIGHT {
            ret[i as usize][j as usize] = match gfx[(i,j)].index {
                1 => Some(GameObject::Block),
                _ => None,
            };
        }
    }
    ret
}
struct Level {
    map: [[Option<GameObject>;HEIGHT as usize];WIDTH as usize],
}

fn level(index: u32) -> Level {
    match index {
        2 => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level2")[..])).unwrap()),
        },
        3 => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level3")[..])).unwrap()),
        },
        4 => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level4")[..])).unwrap()),
        },
        5 => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level5")[..])).unwrap()),
        },
        6 => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level6")[..])).unwrap()),
        },
        7 => Level {
                map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level7")[..])).unwrap()),
        },
        _ => Level {
                map:[[None;HEIGHT as usize]; WIDTH as usize]
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
        ];
        let mut block = Graphic::load_from(Cursor::new(&include_bytes!("../garbage_block")[..])).unwrap().textured(&texture_creator);
        block.update_texture(&tile_set);
        let apple =  OutlinedTile::new(153,PALE_BROWN,&tile_set,texture_creator);
        let golden_apple =  OutlinedTile::new(151,ORANGE,&tile_set,texture_creator);
        let mut tail_gfx = [
             Graphic::load_from(Cursor::new(&include_bytes!("../tail_e")[..])).unwrap().textured(&texture_creator),
             Graphic::load_from(Cursor::new(&include_bytes!("../tail_n")[..])).unwrap().textured(&texture_creator),
             Graphic::load_from(Cursor::new(&include_bytes!("../tail_w")[..])).unwrap().textured(&texture_creator),
             Graphic::load_from(Cursor::new(&include_bytes!("../tail_s")[..])).unwrap().textured(&texture_creator)
        ];
        for i in tail_gfx.iter_mut() {
            i.update_texture(&tile_set);
        }
        let mut head_gfx = [
             Graphic::load_from(Cursor::new(&include_bytes!("../head_e")[..])).unwrap().textured(&texture_creator),
             Graphic::load_from(Cursor::new(&include_bytes!("../head_n")[..])).unwrap().textured(&texture_creator),
             Graphic::load_from(Cursor::new(&include_bytes!("../head_w")[..])).unwrap().textured(&texture_creator),
             Graphic::load_from(Cursor::new(&include_bytes!("../head_s")[..])).unwrap().textured(&texture_creator)
        ];
        for i in head_gfx.iter_mut() {
            i.update_texture(&tile_set);
        }
        let mut left_turn_gfx = [
             Graphic::load_from(Cursor::new(&include_bytes!("../tl_e")[..])).unwrap().textured(&texture_creator),
             Graphic::load_from(Cursor::new(&include_bytes!("../tl_n")[..])).unwrap().textured(&texture_creator),
             Graphic::load_from(Cursor::new(&include_bytes!("../tl_w")[..])).unwrap().textured(&texture_creator),
             Graphic::load_from(Cursor::new(&include_bytes!("../tl_s")[..])).unwrap().textured(&texture_creator)
        ];
        for i in left_turn_gfx.iter_mut() {
            i.update_texture(&tile_set);
        }

        let mut right_turn_gfx = [
             Graphic::load_from(Cursor::new(&include_bytes!("../tl_n")[..])).unwrap().textured(&texture_creator),
             Graphic::load_from(Cursor::new(&include_bytes!("../tl_w")[..])).unwrap().textured(&texture_creator),
             Graphic::load_from(Cursor::new(&include_bytes!("../tl_s")[..])).unwrap().textured(&texture_creator),
             Graphic::load_from(Cursor::new(&include_bytes!("../tl_e")[..])).unwrap().textured(&texture_creator)
        ];
        for i in right_turn_gfx.iter_mut() {
            i.update_texture(&tile_set);
        }

        let mut straight_gfx = [
             Graphic::load_from(Cursor::new(&include_bytes!("../straight_e")[..])).unwrap().textured(&texture_creator),
             Graphic::load_from(Cursor::new(&include_bytes!("../straight_n")[..])).unwrap().textured(&texture_creator),
             Graphic::load_from(Cursor::new(&include_bytes!("../straight_w")[..])).unwrap().textured(&texture_creator),
             Graphic::load_from(Cursor::new(&include_bytes!("../straight_s")[..])).unwrap().textured(&texture_creator)
        ];
        for i in straight_gfx.iter_mut() {
            i.update_texture(&tile_set);
        }
        let tiles = [[None; HEIGHT as usize];WIDTH as usize];
        Game {
            tile_set: tile_set,
            player: (WIDTH as i32/2,HEIGHT as i32/2),
            dead: false,        
            ticks: 0,
            growth:0,
            tiles: tiles,
            score: 0,
            dying_ticks: 0,
            speed:7,
            approach_direction: Direction::N,
            direction: Direction::N,
            relative_controls: false,
            bonus: None,
            bonus_timer: 0,
            tail: VecDeque::new(),
            left_turn_gfx, right_turn_gfx, straight_gfx,head_gfx,tail_gfx,
            block, cursor_gfx, apple,golden_apple
        }
    }
    fn place_apple(&mut self) {
        let mut x = self.player.0;
        let mut y = self.player.1;
        while self.tiles[x as usize][y as usize].is_some() {
            x = thread_rng().gen_range(1,WIDTH as i32-1);
            y = thread_rng().gen_range(1,HEIGHT as i32-1);
        }
        self.tiles[x as usize][y as usize] = Some(GameObject::Apple);
    }
    fn set_up_level(&mut self, lev : u32) {
        let l = level(lev);
        self.player = (WIDTH as i32/2,HEIGHT as i32/2);
        self.dead = false;
        self.tiles = l.map;
        self.tiles[WIDTH as usize/2][HEIGHT as usize/2] = Some(GameObject::Straight(Direction::N));
        self.tiles[WIDTH as usize/2][HEIGHT as usize/2+1] = Some(GameObject::Straight(Direction::N));
        self.dying_ticks = 0;
        self.direction = Direction::N;
        self.approach_direction = Direction::N;
        self.ticks = 0;
        self.tail = VecDeque::from(vec![(WIDTH as i32/2, HEIGHT as i32/2+1)]);
        self.bonus_timer = 0;
        self.bonus = None;
        self.place_apple();
    }
    fn draw<T:RenderTarget>(&self, canvas : &mut Canvas<T>) {
        canvas.set_draw_color(DARK_CHARCOAL);
        canvas.clear();
        canvas.set_draw_color(BLACK);
        for i in 0..WIDTH as i32 {
            for j in 0..HEIGHT as i32 {
                if i % 2 == j %2 {
                    canvas.fill_rect(Rect::new(i*12,j*12+17,12,12)).unwrap();
                }
                if let Some(x) = self.tiles[i as usize][j as usize] {
                    match x {
                        GameObject::Block => self.block.draw(canvas,(i*12-6,j*12-6+17)),
                        GameObject::Straight(d) => self.straight_gfx[d.to_index()-1].draw(canvas,(i*12-6,j*12-6+17)),
                        GameObject::LeftFrom(d) => self.left_turn_gfx[d.to_index()-1].draw(canvas,(i*12-6,j*12-6+17)),
                        GameObject::RightFrom(d) => self.right_turn_gfx[d.to_index()-1].draw(canvas,(i*12-6,j*12-6+17)),
                        GameObject::Apple => self.apple.draw(canvas,(i*12+1,j*12+1+17)),
                        GameObject::GoldenApple => self.golden_apple.draw(canvas,(i*12+1,j*12+1+17)),
                    }
                }
            }
        }
        let (hx,hy) = Self::clamp_bounds(self.direction.move_point(self.player));
        if self.dying_ticks == 0 { self.head_gfx[self.direction.to_index()-1].draw(canvas,(hx*12-6,hy*12-6+17)); }
        if let Some(c) = self.tail.back() {
            if let Some(x) = self.tiles[c.0 as usize][c.1 as usize] {
                if let Some(d) = x.direction() {
                    let (i,j) = Self::clamp_bounds(d.opposite().move_point(*c));
                    self.tail_gfx[d.to_index()-1].draw(canvas,d.move_point(d.move_point((i*12-6,j*12-6+17))));
                }
            }
        }
    }
    fn clamp_bounds(p:(i32,i32)) -> (i32,i32) {
        let (mut x, mut y) = p;
        while x < 0 { x += WIDTH as i32 };
        while y < 0 { y += HEIGHT as i32};
        return (x % WIDTH as i32, y % HEIGHT as i32);
    }
    fn place_bonus(&mut self) {
        let mut x = self.player.0;
        let mut y = self.player.1;
        while self.tiles[x as usize][y as usize].is_some() {
            x = thread_rng().gen_range(1,WIDTH as i32-1);
            y = thread_rng().gen_range(1,HEIGHT as i32-1);
        }
        self.tiles[x as usize][y as usize] = Some(GameObject::GoldenApple);
        self.bonus = Some((x,y));
        self.bonus_timer = ((x-self.player.0).abs() + (y-self.player.1).abs() + thread_rng().gen_range(0,10)) as u32;
        self.bonus_timer -= thread_rng().gen_range(0,8).min(self.bonus_timer);
        if self.bonus_timer < 6 { self.bonus_timer = 6; }
    }
    fn clear_bonus(&mut self) {
        if let Some((x,y)) = self.bonus {
            self.tiles[x as usize][y as usize] = None;
            self.bonus = None;
        }
        self.bonus_timer = 30;
    }
    fn advance_snake(&mut self) -> bool {
        self.tail.push_front(self.player);
        self.player = Self::clamp_bounds(self.direction.move_point(self.player));
        self.approach_direction = self.direction;
        if let Some(x) = self.tiles[self.player.0 as usize][self.player.1 as usize] {
            match x {
                GameObject::Apple => {
                    self.score += 5;
                    self.growth += 1;
                    self.place_apple();
                },
                GameObject::GoldenApple => {
                    self.score += 50;
                    self.growth += 3;
                    self.bonus = None;
                    self.bonus_timer = 30;
                }
                _ => {
                    return true;
                }
            }
        } 
        if self.growth == 0 {
            let end = self.tail.pop_back().unwrap();
            self.tiles[end.0 as usize][end.1 as usize] = None;
        } else {
            self.growth -= 1;
        }
        self.adjust_head_tile();
        false
    }
    fn tick(&mut self) {
        if self.ticks == 0 {
            if self.dying_ticks == 0  {
                if self.bonus_timer == 0 && self.bonus == None {
                    self.place_bonus();
                } else if self.bonus_timer == 0 {
                    self.clear_bonus();
                } else {
                    self.bonus_timer -= 1;
                }
                
                if self.advance_snake() {
                    self.dying_ticks = 2;
                }
                self.ticks += self.speed; 
            } else {
                if self.tail.len() > 0 && self.dying_ticks == 1 {
                    let (x,y) = self.tail.pop_front().unwrap();
                    self.tiles[x as usize][y as usize] = None;
                    self.ticks = 4;
                } else if self.dying_ticks == 1 {
                    self.dying_ticks = 0;
                    self.dead = true;
                    self.ticks = 2;
                } else {
                    self.dying_ticks -= 1;
                    self.ticks = 30;
                }
            }
        } else { 
            self.ticks -= 1;
        }
    }
    fn dir_for_cursor(&self, ix : i32, iy : i32) -> Option<Direction> {
        let (x,y) = ((ix/12).max(0).min(WIDTH as i32) - (WIDTH as i32 / 2), ((iy - 17) / 12).max(0).min(HEIGHT as i32) - (HEIGHT as i32 / 2));
        if self.relative_controls {
            if x > 0 { Some(Direction::E) } else if x < 0 { Some(Direction::W) } else { None }
        } else {
            match (x-y,x+y) {
                (a,b) if a > 0 && b > 0 => Some(Direction::E),
                (a,b) if a < 0 && b < 0 => Some(Direction::W),
                (a,b) if a < 0 && b > 0 => Some(Direction::S),
                (a,b) if a > 0 && b < 0 => Some(Direction::N),
                _ => None
            }
        }
    }
    fn make_move(&mut self, d : Direction) {
        let mut d = d;
        if self.relative_controls { 
            match d {
                Direction::W => d = self.direction.left(),
                Direction::E => d = self.direction.right(),
                _ => return
            }
        }
        if self.dying_ticks > 0 { return };
        if self.approach_direction.opposite() == d { return; }
        self.direction = d;
        self.adjust_head_tile();

    }
    fn adjust_head_tile(&mut self) {
        self.tiles[self.player.0 as usize][self.player.1 as usize] = Some(
        match (self.approach_direction, self.direction) {
            (Direction::N, Direction::E) => GameObject::RightFrom(Direction::N),
            (Direction::N, Direction::W) => GameObject::LeftFrom(Direction::N),
            (Direction::N, _) => GameObject::Straight(Direction::N),
            (Direction::S, Direction::W) => GameObject::RightFrom(Direction::S),
            (Direction::S, Direction::E) => GameObject::LeftFrom(Direction::S),
            (Direction::S, _) => GameObject::Straight(Direction::S),
            (Direction::E, Direction::S) => GameObject::RightFrom(Direction::E),
            (Direction::E, Direction::N) => GameObject::LeftFrom(Direction::E),
            (Direction::E, _) => GameObject::Straight(Direction::E),
            (Direction::W, Direction::N) => GameObject::RightFrom(Direction::W),
            (Direction::W, Direction::S) => GameObject::LeftFrom(Direction::W),
            (Direction::W, _) => GameObject::Straight(Direction::W),
        })
    }
}

enum Splash {
    Level,
    Loss,
    Pause,
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.text_input().start();
    let window = video_subsystem.window("taipan", WIDTH*12, HEIGHT*12 + 16 + 17)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    let mut splash = Some(Splash::Level);
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(WIDTH*12, HEIGHT*12 + 16 + 17).unwrap();
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut game = Game::new(&texture_creator);
    let mut level = 1;
    game.set_up_level(level);
    let mut status_level = Graphic::blank(8,1).textured(&texture_creator);
    let mut status_score = Graphic::blank(7,1).textured(&texture_creator);
    let mut lose = Graphic::load_from(Cursor::new(&include_bytes!("../lose")[..])).unwrap().textured(&texture_creator);
    lose.update_texture(&game.tile_set);
    let mut level_splash = Graphic::load_from(Cursor::new(&include_bytes!("../level_splash")[..])).unwrap().textured(&texture_creator);
    let mut paused_splash = Graphic::load_from(Cursor::new(&include_bytes!("../paused_splash")[..])).unwrap().textured(&texture_creator);
    paused_splash.update_texture(&game.tile_set);
    level_splash.update_texture(&game.tile_set);
    let mut rate_limiter = FPSManager::new();    
    rate_limiter.set_framerate(40).unwrap();
    let mut micro_mode = false;    
    let mut menu = MenuBar::new(WIDTH*12)
                    .add(Menu::new("GAME",144,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Restart",15, Keycode::N,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(128, &texture_creator,&game.tile_set))
                            .add(MenuItem::new("Prev Stage",356, Keycode::F5,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Next Stage",357, Keycode::F6,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(128, &texture_creator,&game.tile_set))
                            .add(MenuItem::new("Relative Ctrls", 358,Keycode::F7,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(128, &texture_creator,&game.tile_set))
                            .add(MenuItem::new("Quit", 363, Keycode::F12,&texture_creator,&game.tile_set)))
                    .add(Menu::new("SPEED",88,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Slow", 352,Keycode::F1,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Medium", 353, Keycode::F2,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Fast", 354, Keycode::F3,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Blazing", 355, Keycode::F4,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(72, &texture_creator,&game.tile_set))
                            .add(MenuItem::new("Pause", 17, Keycode::P,&texture_creator,&game.tile_set)))
                    .add(Menu::new("VIEW",112,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Micro-mode",360, Keycode::F9, &texture_creator, &game.tile_set)));
    let mut cx = 0;
    let mut cy = 0;

    loop {
        if splash.is_none() { 
            if game.dead {
                game.score = 0;
                game.set_up_level(level);
                splash = Some(Splash::Loss);
            }
        }
        if splash.is_none() {
            game.tick();
        }

        game.draw(&mut canvas);
        let c = game.dir_for_cursor(cx, cy).map_or(0, |x| x.to_index());
        canvas.set_draw_color(BLACK);
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*12 + 17,WIDTH*12,40)).unwrap();
        canvas.set_draw_color(rgba(176,179,172,255));
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*12 + 18,WIDTH*12,16)).unwrap();
        status_level.draw_rect(0,0,8,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_level.draw_text(&("Stage ".to_owned() + &level.to_string()),&game.tile_set,0,0,BLACK,TRANSPARENT);
        status_level.update_texture(&game.tile_set);
        status_score.draw_rect(0,0,2,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_score.draw_text(&game.score.to_string(),&game.tile_set,0,0,BLACK,TRANSPARENT);
        status_score.update_texture(&game.tile_set);
        status_level.draw(&mut canvas, (8, HEIGHT as i32 * 12 + 22 ));
        status_score.draw(&mut canvas, (WIDTH as i32 * 12 - 8 - game.score.to_string().len() as i32 * 8, HEIGHT as i32 * 12 +22 ));
        if let Some(s) = &splash {
            match *s {
                Splash::Level => { 
                    level_splash.draw(&mut canvas, (WIDTH as i32 * 6 - (11*4), HEIGHT as i32 * 6 - 16 + 17 ));
                    status_level.draw(&mut canvas, (WIDTH as i32 * 6 - (8 * 4) + if level < 10 { 4} else {0}, HEIGHT as i32 * 6 - 4 + 17));
                },
                Splash::Loss => lose.draw(&mut canvas, (WIDTH as i32 * 6 - (21*4), HEIGHT as i32 * 6 - (21*4) + 17 )),
                Splash::Pause => paused_splash.draw(&mut canvas, (WIDTH as i32 * 6 - (11*4), HEIGHT as i32 * 6 - 20 + 17 )),
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
                Event::KeyDown { keycode: Some(Keycode::F5), ..} => {
                    if level > 1 {
                        level -= 1;
                        game.score = 0;
                        game.set_up_level(level);
                        splash = Some(Splash::Level);
                    }
                },              
                Event::KeyDown { keycode: Some(Keycode::F6), ..} => {
                    if level < 7 {
                        level += 1;
                        game.score = 0;
                        game.set_up_level(level);
                        splash = Some(Splash::Level);
                    }
                },              
                Event::KeyDown { keycode: Some(Keycode::F7), ..}
                  => game.relative_controls = !game.relative_controls,
                Event::KeyDown { keycode: Some(Keycode::F9), ..} => {
                    if micro_mode {
                        micro_mode = false;
                        canvas.window_mut().set_size(WIDTH*12, HEIGHT*12+16+17).unwrap_or_default();
                    } else {
                        micro_mode = true;
                        canvas.window_mut().set_size(WIDTH*6, (HEIGHT*12+16+17)/2).unwrap_or_default();
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::F1), ..}
                  => game.speed = 12,
                Event::KeyDown { keycode: Some(Keycode::F2), ..}
                  => game.speed = 6,
                Event::KeyDown { keycode: Some(Keycode::F3), ..}
                  => game.speed = 2,
                Event::KeyDown { keycode: Some(Keycode::F4), ..}
                  => game.speed = 1,
                Event::KeyDown { keycode: Some(Keycode::N), .. } => {
                    game.set_up_level(level);
                    splash = Some(Splash::Level);
                },
                Event::KeyDown {..} if splash.is_some() => {
                    splash = None  
                }
                Event::MouseButtonUp {..} if splash.is_some() => {
                    splash = None
                }
                Event::KeyDown { keycode: Some(Keycode::Up), ..} |
                Event::KeyDown { keycode: Some(Keycode::W), ..}
                  => game.make_move(Direction::N),
                Event::KeyDown { keycode: Some(Keycode::Down), ..} |
                Event::KeyDown { keycode: Some(Keycode::S), ..}
                  => game.make_move(Direction::S),
                Event::KeyDown { keycode: Some(Keycode::Left), ..} |
                Event::KeyDown { keycode: Some(Keycode::A), ..}
                  => game.make_move(Direction::W),
                Event::KeyDown { keycode: Some(Keycode::Right), ..} |
                Event::KeyDown { keycode: Some(Keycode::D), ..}
                  => game.make_move(Direction::E),
                Event::KeyDown { keycode: Some(Keycode::P), ..}
                  => splash = Some(Splash::Pause),
                Event::MouseButtonUp {..} if cy > 17 => {
                    match game.dir_for_cursor(cx, cy) {
                        None => {},
                        Some(d) => game.make_move(d),
                    }
                }
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

