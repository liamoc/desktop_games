
extern crate tesserae;
extern crate sdl2;
extern crate rand;
extern crate utils;

use utils::menu::{*};

use tesserae::{Graphic,Tile,TileSet};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::render::{Texture, BlendMode, TextureCreator};
use sdl2::video::Window;
use sdl2::Sdl; 
use std::collections::VecDeque;
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::gfx::framerate::FPSManager;
use sdl2::rect::Rect;
use std::io::Cursor;
use rand::{thread_rng,Rng};
use utils::color::{*};

struct GraphicsSet<T> {
    tile_set: TileSet,
    stones: [[Graphic<T>;3];4],
    wildcard: Graphic<T>,
    stone_outline: Graphic<T>,
    stone_blank: [Graphic<T>;4],
    stone_depth : [Graphic<T>;4],
    cursor: Graphic<T>,
    cursor_hint: Graphic<T>,
    cursor_outline: Graphic<T>,
    dragon: Graphic<T>,
    dragon_outline: Graphic<T>,
    win: Graphic<T>,
}
const UI_DARK : Color = rgba(85,87,83,255);
const UI_LIGHT : Color = rgba(176,179,172,255);
impl <'r> GraphicsSet<Texture<'r>> {
    fn dragon_emblem<T>(texture_creator: &'r TextureCreator<T>, color: Color, tile_set: &TileSet) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(1,2).textured(texture_creator);
        ret[(0,0)] = Tile { index: 431, fg: color, bg: TRANSPARENT };
        ret[(0,1)] = Tile { index: 447, fg: color, bg: TRANSPARENT };
        ret.update_texture(tile_set);
        ret
    }
    fn circle_stone<T>(texture_creator: &'r TextureCreator<T>, color: Color, tile_set: &TileSet) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(2,2).textured(texture_creator);
        ret[(0,0)] = Tile { index: 302, bg: color, fg: TRANSPARENT };
        ret[(1,0)] = Tile { index: 303, bg: color, fg: TRANSPARENT };
        ret[(0,1)] = Tile { index: 287, bg: color, fg: TRANSPARENT };
        ret[(1,1)] = Tile { index: 286, bg: color, fg: TRANSPARENT };
        ret.update_texture(tile_set);
        ret
    }
    fn square_stone<T>(texture_creator: &'r TextureCreator<T>, color: Color, tile_set: &TileSet) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(2,2).textured(texture_creator);
        ret[(0,0)] = Tile { index: 106, bg: color, fg: TRANSPARENT };
        ret[(1,0)] = Tile { index: 107, bg: color, fg: TRANSPARENT };
        ret[(0,1)] = Tile { index: 122, bg: color, fg: TRANSPARENT };
        ret[(1,1)] = Tile { index: 123, bg: color, fg: TRANSPARENT };
        ret.update_texture(tile_set);
        ret
    }
    fn star_stone<T>(texture_creator: &'r TextureCreator<T>, color: Color, tile_set: &TileSet) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(2,2).textured(texture_creator);
        ret[(0,0)] = Tile { index: 286, fg: color, bg: TRANSPARENT };
        ret[(1,0)] = Tile { index: 287, fg: color, bg: TRANSPARENT };
        ret[(0,1)] = Tile { index: 303, fg: color, bg: TRANSPARENT };
        ret[(1,1)] = Tile { index: 302, fg: color, bg: TRANSPARENT };
        ret.update_texture(tile_set);
        ret
    }
    fn diamond_stone<T>(texture_creator: &'r TextureCreator<T>, color: Color, tile_set: &TileSet) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(2,2).textured(texture_creator);
        ret[(0,0)] = Tile { index: 206, fg: color, bg: TRANSPARENT };
        ret[(1,0)] = Tile { index: 207, fg: color, bg: TRANSPARENT };
        ret[(0,1)] = Tile { index: 207, bg: color, fg: TRANSPARENT };
        ret[(1,1)] = Tile { index: 206, bg: color, fg: TRANSPARENT };
        ret.update_texture(tile_set);
        ret
    }
    fn dots_stone<T>(texture_creator: &'r TextureCreator<T>, color: Color, tile_set: &TileSet) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(2,2).textured(texture_creator);
        ret[(0,0)] = Tile { index: 254, bg: color, fg: TRANSPARENT };
        ret[(1,0)] = Tile { index: 254, bg: color, fg: TRANSPARENT };
        ret[(0,1)] = Tile { index: 254, bg: color, fg: TRANSPARENT };
        ret[(1,1)] = Tile { index: 254, bg: color, fg: TRANSPARENT };
        ret.update_texture(tile_set);
        ret
    }
    fn cursor<T>(texture_creator: &'r TextureCreator<T>, color: Color, tile_set: &TileSet) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(4,4).textured(texture_creator);
        ret[(0,0)] = Tile { index: 137, fg: TRANSPARENT, bg: color };
        ret[(3,0)] = Tile { index: 136, fg: TRANSPARENT, bg: color };
        ret[(3,3)] = Tile { index: 118, bg: TRANSPARENT, fg: color };
        ret[(0,3)] = Tile { index: 119, bg: TRANSPARENT, fg: color };
        ret.update_texture(tile_set);
        ret
    }
    fn blank_stone<T>(texture_creator: &'r TextureCreator<T>, color: Color, tile_set: &TileSet) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(4,4).textured(texture_creator);
        ret[(0,0)] = Tile { index: 168, bg: TRANSPARENT, fg: color };
        ret[(1,0)] = Tile { index: 169, bg: TRANSPARENT, fg: color };
        ret[(2,0)] = Tile { index: 169, bg: TRANSPARENT, fg: color };
        ret[(3,0)] = Tile { index: 170, bg: TRANSPARENT, fg: color };
        ret[(0,1)] = Tile { index: 184, bg: TRANSPARENT, fg: color };
        ret[(1,1)] = Tile { index: 0, fg: TRANSPARENT, bg: color };
        ret[(2,1)] = Tile { index: 0, fg: TRANSPARENT, bg: color };
        ret[(3,1)] = Tile { index: 186, bg: TRANSPARENT, fg: color };
        ret[(0,2)] = Tile { index: 184, bg: TRANSPARENT, fg: color };
        ret[(1,2)] = Tile { index: 0, fg: TRANSPARENT, bg: color };
        ret[(2,2)] = Tile { index: 0, fg: TRANSPARENT, bg: color };
        ret[(3,2)] = Tile { index: 186, bg: TRANSPARENT, fg: color };
        ret[(0,3)] = Tile { index: 200, bg: TRANSPARENT, fg: color };
        ret[(1,3)] = Tile { index: 201, bg: TRANSPARENT, fg: color };
        ret[(2,3)] = Tile { index: 201, bg: TRANSPARENT, fg: color };
        ret[(3,3)] = Tile { index: 202, bg: TRANSPARENT, fg: color };
        ret.update_texture(tile_set);
        ret
    }
    
    fn new<T>(texture_creator: &'r TextureCreator<T>) -> GraphicsSet<Texture<'r>> {
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
        let mut win = Graphic::load_from(Cursor::new(&include_bytes!("../win")[..])).unwrap().textured(&texture_creator);
        win.update_texture(&tile_set);
        let stone_outline = Self::blank_stone(texture_creator, CHARCOAL, &tile_set);
        let stone_depth = [
            Self::blank_stone(texture_creator, rgba(169,109,0,255), &tile_set),
            Self::blank_stone(texture_creator, rgba(50,120,0,255), &tile_set),
            Self::blank_stone(texture_creator, rgba(52,101,164,255), &tile_set),
            Self::blank_stone(texture_creator, rgba(85,87,83,255), &tile_set),
        ];
        let stone_blank = [
            Self::blank_stone(texture_creator, rgba(203,153,72,255), &tile_set),
            Self::blank_stone(texture_creator, rgba(78,154,6,255), &tile_set),
            Self::blank_stone(texture_creator, PALE_BLUE, &tile_set),
            Self::blank_stone(texture_creator, rgba(115,120,110,255), &tile_set),
        ];
        let my_yellow = rgba(237,212,0,255);
        let my_white = rgba(238,238,238,255);
        let stones : [[Graphic<Texture<'r>>;3];4] = [
            [
                Self::circle_stone(texture_creator, my_white, &tile_set),
                Self::circle_stone(texture_creator, CHARCOAL, &tile_set),
                Self::circle_stone(texture_creator, my_yellow, &tile_set),
            ],[
                Self::square_stone(texture_creator, my_white, &tile_set),
                Self::square_stone(texture_creator, CHARCOAL, &tile_set),
                Self::square_stone(texture_creator, my_yellow, &tile_set),
            ],[
                Self::diamond_stone(texture_creator, my_white, &tile_set),
                Self::diamond_stone(texture_creator, CHARCOAL, &tile_set),
                Self::diamond_stone(texture_creator, my_yellow, &tile_set),
            ],[
                Self::star_stone(texture_creator, my_white, &tile_set),
                Self::star_stone(texture_creator, CHARCOAL, &tile_set),
                Self::star_stone(texture_creator, my_yellow, &tile_set),
            ]
        ];
        let wildcard = Self::dots_stone(texture_creator, NEUTRAL_GRAY, &tile_set);
        let cursor = Self::cursor(texture_creator,WHITE, &tile_set);
        let cursor_outline = Self::cursor(texture_creator,CHARCOAL, &tile_set);
        let cursor_hint = Self::cursor(texture_creator,BRIGHT_GREEN, &tile_set);
        let dragon = Self::dragon_emblem(texture_creator, rgba(204,0,0,255), &tile_set);
        let dragon_outline = Self::dragon_emblem(texture_creator, YELLOW, &tile_set);
        GraphicsSet {
            tile_set: tile_set,cursor, cursor_outline,dragon,dragon_outline,
            win, stone_blank, stone_depth, stone_outline, stones, wildcard, cursor_hint
        }
    }

}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Shape {
    Circle = 0, Square = 1, Diamond = 2, Star = 3, Wildcard = 4
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShapeColor {
    White = 0, Black = 1, Yellow = 2
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StoneColor {
    Brown = 0, Green = 1, Blue = 2
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Stone {
    pub shape : Shape,
    pub bg : StoneColor,
    pub fg : ShapeColor,
}
impl Stone {
    pub fn random_stone() -> Stone {
        let n = thread_rng().gen_range(0,37);
        if n == 36 {
            Stone { shape: Shape::Wildcard, bg : StoneColor::Brown, fg : ShapeColor::White }
        } else {
            let shape = match n % 4 {
                0 => Shape::Circle,
                1 => Shape::Square,
                2 => Shape::Diamond,
                _ => Shape::Star,
            };
            let fg = match (n / 4) % 3 {
                0 => ShapeColor::White,
                1 => ShapeColor::Black,
                _ => ShapeColor::Yellow,
            };
            let bg = match ((n / 4) / 3) % 3 {
                0 => StoneColor::Brown,
                1 => StoneColor::Green,
                _ => StoneColor::Blue,
            };
            Stone { shape, fg, bg }
        }
    }
    pub fn matches(t : Stone, u : Stone) -> bool {
        if t.shape == Shape::Wildcard || u.shape == Shape::Wildcard { return true };
        let mut attributes = 0;
        if t.shape == u.shape { attributes += 1 }
        if t.fg == u.fg { attributes += 1 }
        if t.bg == u.bg { attributes += 1 }
        attributes >= 2
    }
    fn color_index(&self) -> usize {
        if self.shape == Shape::Wildcard { 3 } else { self.bg as usize }
    }
    fn draw_base<'r>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>, position : (i32,i32)) {
        for offset in 0..2 {
            graphics.stone_outline.draw(canvas,(position.0+offset-1,position.1+offset-1));
            graphics.stone_outline.draw(canvas,(position.0+offset-1,position.1+offset+1));
            graphics.stone_outline.draw(canvas,(position.0+offset+1,position.1+offset-1));
        }
        for offset in 0..2 {
            graphics.stone_depth[self.color_index()].draw(canvas,(position.0+offset,position.1+offset));
        }
    }
    fn draw<'r>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>, position : (i32,i32)) {
        graphics.stone_outline.draw(canvas,(position.0+1,position.1+1));
        graphics.stone_outline.draw(canvas,(position.0+2,position.1+1));
        graphics.stone_outline.draw(canvas,(position.0+3,position.1+1));
        graphics.stone_outline.draw(canvas,(position.0+1,position.1+2));
        graphics.stone_outline.draw(canvas,(position.0+3,position.1+2));
        graphics.stone_outline.draw(canvas,(position.0+1,position.1+3));
        graphics.stone_outline.draw(canvas,(position.0+2,position.1+3));
        graphics.stone_outline.draw(canvas,(position.0+3,position.1+3));
        let pos = (position.0+2+8,position.1+2+8);
        graphics.stone_blank[self.color_index()].draw(canvas,(position.0+2,position.1+2));
        if self.shape == Shape::Wildcard {
            graphics.wildcard.draw(canvas,pos)
        } else {
            graphics.stones[self.shape as usize][self.fg as usize].draw(canvas,pos)
        }
    }

}
pub struct Table {    
    stones: [[Option<Stone>;10];10],
    queue: VecDeque<Stone>,
    history: Vec<((usize,usize),Stone)>,
    game_won: bool,
    score: i32,
    remaining: usize,
    cursor_location: Option<(usize,usize)>,
    hint_location: Option<(usize,usize)>,
}

impl Table {
    fn fill_queues(&mut self) {
        while self.queue.len() < 6 && self.remaining > 0 {
            let mut stone = Stone::random_stone();
            while self.placeable(stone).is_none() { stone = Stone::random_stone() };
            self.queue.push_back(stone);
            self.remaining -= 1;
        }
        if self.queue.len() == 0 && self.remaining == 0 {
            self.game_won = true;
        }
    }
    fn populate_board(&mut self) {
        for _ in 0..6 {
            let mut x = thread_rng().gen_range(0,10);
            let mut y = thread_rng().gen_range(0,10);
            while self.stones[x][y].is_some() 
               || x > 0 && self.stones[x-1][y].is_some()
               || x < 9 && self.stones[x+1][y].is_some()
               || y > 0 && self.stones[x][y-1].is_some()
               || y > 0 && x > 0 && self.stones[x-1][y-1].is_some()
               || y < 9 && x > 0 && self.stones[x-1][y+1].is_some()
               || y > 0 && x < 9 && self.stones[x+1][y-1].is_some()
               || y < 9 && x < 9 && self.stones[x+1][y+1].is_some()
               || y < 9 && self.stones[x][y+1].is_some() {
                x = thread_rng().gen_range(0,10);
                y = thread_rng().gen_range(0,10);
            }
            self.remaining -= 1;
            self.stones[x][y] = Some(Stone::random_stone());
        }
        self.fill_queues();
    }
    fn hint(&mut self) {
        if let Some(t) = self.queue.front() {
            self.hint_location = self.placeable(*t);
        }
    }
    fn new() -> Table {
        let mut table = Table {
            stones: [[None;10];10],
            queue: VecDeque::new(),
            game_won: false,
            history: Vec::new(),
            cursor_location: None,
            hint_location:None,
            score:0,
            remaining:90,
        };
        table.populate_board();
        table
    }
    fn undo(&mut self) {
        if let Some(((x,y),t)) = self.history.pop() {
            self.stones[x][y] = None;
            self.score -= self.can_place(t, x, y).unwrap() as i32;
            self.score -= 1;
            if self.score < 0 { self.score = 0 };
            self.queue.push_front(t);
            while self.queue.len() > 6 {
                self.queue.pop_back();
                self.remaining += 1;
            }
        }
    }
    fn placeable(&self, t:Stone) -> Option<(usize,usize)> {
        let mut candidates = Vec::new();
        for i in 0..10 {
            for j in 0..10 {
                if let Some(c) = self.can_place(t, i,j) {
                    candidates.push((c,(i,j)));
                }
            }
        }
        candidates.sort_by(|b,a|a.cmp(b));
        candidates.first().map(|(_,y)|*y)
    }
    fn can_place(&self, t: Stone, x:usize,y:usize) -> Option<usize> {
        if self.stones[x][y].is_none() {
            let mut found_link = false;
            let mut score = 2;
            if x > 0 && self.stones[x-1][y].is_some() {
                if let Some(u) = self.stones[x-1][y] {
                    if Stone::matches(t, u) {
                        found_link = true;
                        score *= 2;
                    } else {
                        return None
                    }
                }
            }
            if x < 9 && self.stones[x+1][y].is_some() {
                if let Some(u) = self.stones[x+1][y] {
                    if Stone::matches(t, u) {
                        found_link = true;
                        score *= 2;
                    } else {
                        return None
                    }
                }
            }
            if y > 0 && self.stones[x][y-1].is_some() {
                if let Some(u) = self.stones[x][y-1] {
                    if Stone::matches(t, u) {
                        found_link = true;
                        score *= 2;
                    } else {
                        return None
                    }
                }
            }
            if y < 9 && self.stones[x][y+1].is_some() {
                if let Some(u) = self.stones[x][y+1] {
                    if Stone::matches(t, u) {
                        found_link = true;
                        score *= 2;
                    } else {
                        return None
                    }
                }
            }
            if found_link == false { return None };
            if t.shape == Shape::Wildcard && score > 2 { score = 2 }
            return Some(score)
        } else { return None }
    }
    fn clicked(&mut self) {
        self.hint_location = None;
        if let Some((x,y)) = self.cursor_location {
            if let Some(t) = self.queue.front() {
                if let Some(c) = self.can_place(*t, x,y)  {
                    self.stones[x][y] = Some(*t);
                    self.history.push(((x,y),*t));
                    self.queue.pop_front();
                    self.score += c as i32;
                    self.fill_queues();
                }
            }
        }
    }
    fn draw<'t>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'t>>) {
        canvas.set_draw_color(rgba(238,238,236,255));
        canvas.clear();

        //canvas.fill_rect(rect: R)
        for y in 0..10 {
            for x in 0..10 {
                let _ = if x % 2 == 0 && y % 2 == 0 || x % 2 == 1 && y % 2 == 1 { 
                    canvas.set_draw_color(rgba(221,215,217,255));
                    canvas.fill_rect(Rect::new(x *22+4,y*22+16+5,22,22)).ok() 
                } else { 
                    canvas.set_draw_color(rgba(211,205,207,255));
                    canvas.fill_rect(Rect::new(x *22+4,y*22+16+5,22,22)).ok() 
                };
            }
        }
        for y in (0..10).rev() {
            for x in (0..10).rev() {
                if let Some(t) = self.stones[x][y] {
                    t.draw_base(canvas,graphics,(x as i32*22-1,y as i32*22+16));
                }
            }
        }
        for y in (0..10).rev() {
            for x in (0..10).rev() {
                if let Some(t) = self.stones[x][y] {
                    t.draw(canvas, graphics, (x as i32*22-1,y as i32*22+16));
                }
            }
        }
        if let Some((x,y)) = self.hint_location {
            graphics.cursor_outline.draw(canvas,(x as i32 * 22-1,y as i32 *22+16));
            graphics.cursor_outline.draw(canvas,(x as i32 * 22+1,y as i32 *22+16));
            graphics.cursor_outline.draw(canvas,(x as i32 * 22-1,y as i32 *22+16-1));
            graphics.cursor_outline.draw(canvas,(x as i32 * 22+1,y as i32 *22+16-1));
            graphics.cursor_outline.draw(canvas,(x as i32 * 22,y as i32 *22+16-1));
            graphics.cursor_outline.draw(canvas,(x as i32 * 22-1,y as i32 *22+16+1));
            graphics.cursor_outline.draw(canvas,(x as i32 * 22+1,y as i32 *22+16+1));
            graphics.cursor_outline.draw(canvas,(x as i32 * 22,y as i32 *22+16+1));
            graphics.cursor_hint.draw(canvas,(x as i32 * 22,y as i32 *22+16));
        }
        if let Some((x,y)) = self.cursor_location {
            graphics.cursor_outline.draw(canvas,(x as i32 * 22-1,y as i32 *22+16));
            graphics.cursor_outline.draw(canvas,(x as i32 * 22+1,y as i32 *22+16));
            graphics.cursor_outline.draw(canvas,(x as i32 * 22-1,y as i32 *22+16-1));
            graphics.cursor_outline.draw(canvas,(x as i32 * 22+1,y as i32 *22+16-1));
            graphics.cursor_outline.draw(canvas,(x as i32 * 22,y as i32 *22+16-1));
            graphics.cursor_outline.draw(canvas,(x as i32 * 22-1,y as i32 *22+16+1));
            graphics.cursor_outline.draw(canvas,(x as i32 * 22+1,y as i32 *22+16+1));
            graphics.cursor_outline.draw(canvas,(x as i32 * 22,y as i32 *22+16+1));
            graphics.cursor.draw(canvas,(x as i32 * 22,y as i32 *22+16));
        }
        canvas.set_draw_color(UI_LIGHT);
        let _ = canvas.fill_rect(Rect::new(229,0,50,245));
        let _ = canvas.fill_rect(Rect::new(0,246,280,20));
        canvas.set_draw_color(UI_DARK);
        let _ = canvas.fill_rect(Rect::new(228,0,1,245));
        let _ = canvas.fill_rect(Rect::new(0,245,280,1));
        for i in 0..self.queue.len() {
            let t = self.queue[i];
            t.draw(canvas,graphics,(226,i as i32 *22+16 + if i > 0 {6} else {0}));
        }
        let dx = 239;
        let dy = 200;
        graphics.dragon_outline.draw(canvas,(dx-1,dy-1));
        graphics.dragon_outline.draw(canvas,(dx-1,dy+1));
        graphics.dragon_outline.draw(canvas,(dx+1,dy-1));
        graphics.dragon_outline.draw(canvas,(dx+1,dy+1));
        graphics.dragon.draw(canvas,(dx,dy));
    }

    fn collides_location(&self, position: (i32,i32)) -> Option<(usize, usize)> {
        let bx = (position.0 - 6) / 22;
        let by = (position.1 - 6 - 16) / 22;
        if bx >= 10 || by >= 10 { return None };
        if bx < 0 || by < 0 { return None };
        return Some((bx as usize,by as usize))
    }
}
const WIDTH:u32=259;
const HEIGHT:u32=245+18;
fn main_loop(mut window:Window, sdl_context: &Sdl) {
    window.set_size(WIDTH,HEIGHT).unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    
    canvas.set_logical_size(WIDTH,HEIGHT).unwrap();
    canvas.set_blend_mode(BlendMode::Blend);
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let graphics_set = GraphicsSet::new(&texture_creator);
    let mut table = Table::new();

    let mut score_gfx = Graphic::blank(4,1).textured(&texture_creator);
    let mut remaining_gfx = Graphic::blank(4,1).textured(&texture_creator);
    let mut remaining_gfx_shadow = Graphic::blank(4,1).textured(&texture_creator);
    let mut menu = MenuBar::new(WIDTH)
                    .add(Menu::new("GAME",72+16,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Hint",9, Keycode::H,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Undo",27, Keycode::Z,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(72, &texture_creator, &graphics_set.tile_set))
                            .add(MenuItem::new("Restart",15, Keycode::N,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(72, &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Quit",363, Keycode::F12,&texture_creator,&graphics_set.tile_set)))
                    .add(Menu::new("VIEW",72+(5*8),&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Micro-mode",360, Keycode::F9,&texture_creator, &graphics_set.tile_set)));
    let mut rate_limiter = FPSManager::new();
    rate_limiter.set_framerate(60).unwrap();
    let mut micro_mode = false;
    loop {
        canvas.set_draw_color(CHARCOAL);
        canvas.clear();
        table.draw(&mut canvas, &graphics_set);
        let gx = 235 + if table.remaining < 10 { 4 } else { 0 };
        let gy = 240 - 70;
        remaining_gfx.draw_rect(0, 0, 4, 1, Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:0});
        score_gfx.draw_rect(0, 0, 4, 1, Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:0});
        remaining_gfx.draw_text(&table.remaining.to_string(), &graphics_set.tile_set , 0, 0, WHITE, TRANSPARENT);
        score_gfx.draw_text(&table.score.to_string(), &graphics_set.tile_set , 0, 0, DARK_CHARCOAL, TRANSPARENT);
        remaining_gfx_shadow.draw_rect(0, 0, 4, 1, Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:0});
        remaining_gfx_shadow.draw_text(&table.remaining.to_string(), &graphics_set.tile_set , 0, 0, BLACK, TRANSPARENT);
        remaining_gfx.update_texture(&graphics_set.tile_set);
        remaining_gfx_shadow.update_texture(&graphics_set.tile_set);
        score_gfx.update_texture(&graphics_set.tile_set);
        remaining_gfx_shadow.draw(&mut canvas, (gx,gy-1));
        remaining_gfx_shadow.draw(&mut canvas, (gx,gy+1));
        remaining_gfx_shadow.draw(&mut canvas, (gx-1,gy));
        remaining_gfx_shadow.draw(&mut canvas, (gx+1,gy));
        remaining_gfx.draw(&mut canvas, (gx,gy));

        let won = table.game_won;
        if won {
            let x = WIDTH as i32 / 2 - (21 * 4);
            let y = HEIGHT as i32 / 2 - (21 * 4);
            graphics_set.win.draw(&mut canvas, (x,y));
        }
        menu.draw(&mut canvas);
        score_gfx.draw(&mut canvas, (4 as i32,HEIGHT as i32-12));
        canvas.present();
        let mut md = false;
        let mut event_happened = false;
        while !event_happened {
            for event in event_pump.poll_iter() {
                event_happened = true;
                let h = menu.handle_event(event.clone(), &mut event_subsystem);
                match event {
                    _ if h => {},
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::F12), .. } => {
                        return
                    },
                    Event::KeyDown { keycode: Some(Keycode::H), ..} => {
                        table.hint();
                    },
                    Event::KeyDown { keycode: Some(Keycode::Z), ..} => {
                        table.undo();
                    },
                    Event::KeyDown { keycode: Some(Keycode::F9), ..} => {
                        if micro_mode {
                            micro_mode = false;
                            canvas.window_mut().set_size(WIDTH,HEIGHT).unwrap_or_default();
                        } else {
                            canvas.window_mut().set_size(WIDTH/2,HEIGHT/2).unwrap_or_default();
                            micro_mode = true;
                        }
                    }
                    Event::KeyDown {..} if won => {
                        table = Table::new();
                    },
                    Event::MouseButtonDown { ..} if won => {
                        md = true;
                        table = Table::new();
                    },
                    Event::MouseButtonUp { ..} if won && md => {
                        md= false;
                        table = Table::new();
                    },
                    Event::KeyDown { keycode: Some(Keycode::N), ..} => {
                        table = Table::new();
                    },
                    Event::MouseButtonUp { ..} if !won => {
                        md = false ;
                    }
                    Event::MouseButtonDown { mouse_btn: _, x, y, ..} if !won => {
                        let loc = table.collides_location((x,y));
                        table.cursor_location = loc;
                        table.clicked();
                    }
                    Event::MouseMotion { x, y, ..} if !won => {
                        let loc = table.collides_location((x,y));
                        table.cursor_location = loc;
                    }

                    _ => {},
                }
            }
            rate_limiter.delay();
        }
    }
}


fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("ishi", WIDTH, HEIGHT)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    main_loop(window, &sdl_context);
}