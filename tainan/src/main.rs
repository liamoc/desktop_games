
extern crate tesserae;
extern crate sdl2;
extern crate rand;
extern crate utils;

use std::collections::HashMap;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use utils::menu::{*};

use tesserae::{Graphic,Tile,TileSet};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::render::{Texture, BlendMode, TextureCreator};
use sdl2::video::Window;
use sdl2::Sdl; 
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::gfx::framerate::FPSManager;
use std::io::Cursor;
use rand::{thread_rng};
use rand::seq::SliceRandom;
use utils::color::{*};

struct GraphicsSet<T> {
    tile_set: TileSet,
    tiles: [Graphic<T>;42],
    tile_outline: Graphic<T>,
    tile_blank: Graphic<T>,
    tile_depth: Graphic<T>,
    tile_highlight: Graphic<T>,
    tile_selected: Graphic<T>,
    tile_hint: Graphic<T>,
    tile_hover: Graphic<T>,
    table: Graphic<T>,
    win: Graphic<T>,
}
impl <'r> GraphicsSet<Texture<'r>> {
    const BG_DARK : Color = rgba(143, 89, 2, 255);
    const BG_LIGHT : Color = rgba(193, 125, 17, 255);
    fn tile<T>(texture_creator: &'r TextureCreator<T>, fg1: usize, fg_color1: Color, fg2: usize, fg_color2: Color, tile_set: &TileSet) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(1,2).textured(texture_creator);
        ret[(0,0)] = Tile { index: fg1, fg: fg_color1, bg: TRANSPARENT };
        ret[(0,1)] = Tile { index: fg2, fg: fg_color2, bg: TRANSPARENT };
        ret.update_texture(tile_set);
        ret
    }
    fn blank_tile<T>(texture_creator: &'r TextureCreator<T>, color: Color, tile_set: &TileSet) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(3,4).textured(texture_creator);
        ret[(0,0)] = Tile { index: 168, bg: TRANSPARENT, fg: color };
        ret[(1,0)] = Tile { index: 169, bg: TRANSPARENT, fg: color };
        ret[(2,0)] = Tile { index: 170, bg: TRANSPARENT, fg: color };
        ret[(0,1)] = Tile { index: 184, bg: TRANSPARENT, fg: color };
        ret[(1,1)] = Tile { index: 0, fg: TRANSPARENT, bg: color };
        ret[(2,1)] = Tile { index: 186, bg: TRANSPARENT, fg: color };
        ret[(0,2)] = Tile { index: 184, bg: TRANSPARENT, fg: color };
        ret[(1,2)] = Tile { index: 0, fg: TRANSPARENT, bg: color };
        ret[(2,2)] = Tile { index: 186, bg: TRANSPARENT, fg: color };
        ret[(0,3)] = Tile { index: 200, bg: TRANSPARENT, fg: color };
        ret[(1,3)] = Tile { index: 201, bg: TRANSPARENT, fg: color };
        ret[(2,3)] = Tile { index: 202, bg: TRANSPARENT, fg: color };
        ret.update_texture(tile_set);
        ret
    }
    
    fn new<T>(texture_creator: &'r TextureCreator<T>) -> GraphicsSet<Texture<'r>> {
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
        let mut win = Graphic::load_from(Cursor::new(&include_bytes!("../win")[..])).unwrap().textured(&texture_creator);
        win.update_texture(&tile_set);
        let tile_outline = Self::blank_tile(texture_creator, CHARCOAL, &tile_set);
        let tile_depth = Self::blank_tile(texture_creator, NEUTRAL_GRAY, &tile_set);
        let tile_blank = Self::blank_tile(texture_creator, WHITE, &tile_set);
        let tile_highlight = Self::blank_tile(texture_creator, rgba(255,255,200,255), &tile_set);
        let tile_hover= Self::blank_tile(texture_creator, rgba(230,230,255,255), &tile_set);
        let tile_selected = Self::blank_tile(texture_creator, rgba(200,200,255,255), &tile_set);
        let tile_hint = Self::blank_tile(texture_creator, rgba(255,200,255,255), &tile_set);
        let mut table = Graphic::solid(640/8, 480/8, Tile {fg: Self::BG_LIGHT, bg:Self::BG_DARK, index:255}).textured(texture_creator);
        table.update_texture(&tile_set);
        let tiles : [Graphic<Texture<'r>>;42] = [
            Self::tile(texture_creator, 378, TEAL, 394, TEAL, &tile_set),
            Self::tile(texture_creator, 379, TEAL, 379, TEAL, &tile_set),
            Self::tile(texture_creator, 380, TEAL, 396, TEAL, &tile_set),
            Self::tile(texture_creator, 395, TEAL, 395, TEAL, &tile_set),
            Self::tile(texture_creator, 381, TEAL, 397, TEAL, &tile_set),
            Self::tile(texture_creator, 382, TEAL, 398, TEAL, &tile_set),
            Self::tile(texture_creator, 383, TEAL, 398, TEAL, &tile_set),
            Self::tile(texture_creator, 192, TEAL, 192, TEAL, &tile_set),
            Self::tile(texture_creator, 399, TEAL, 415, TEAL, &tile_set),
            Self::tile(texture_creator, 372, GREEN, 388, GREEN, &tile_set),
            Self::tile(texture_creator, 373, GREEN, 373, GREEN, &tile_set),
            Self::tile(texture_creator, 373, GREEN, 389, GREEN, &tile_set),
            Self::tile(texture_creator, 389, GREEN, 389, GREEN, &tile_set),
            Self::tile(texture_creator, 375, GREEN, 391, GREEN, &tile_set),
            Self::tile(texture_creator, 390, GREEN, 390, GREEN, &tile_set),
            Self::tile(texture_creator, 376, GREEN, 392, GREEN, &tile_set),
            Self::tile(texture_creator, 377, GREEN, 393, GREEN, &tile_set),
            Self::tile(texture_creator, 374, GREEN, 392, GREEN, &tile_set),
            Self::tile(texture_creator, 409, BLACK, 428, DARK_RED, &tile_set),
            Self::tile(texture_creator, 410, BLACK, 428, DARK_RED, &tile_set),
            Self::tile(texture_creator, 411, BLACK, 428, DARK_RED, &tile_set),
            Self::tile(texture_creator, 412, BLACK, 428, DARK_RED, &tile_set),
            Self::tile(texture_creator, 413, BLACK, 428, DARK_RED, &tile_set),
            Self::tile(texture_creator, 414, BLACK, 428, DARK_RED, &tile_set),
            Self::tile(texture_creator, 425, BLACK, 428, DARK_RED, &tile_set),
            Self::tile(texture_creator, 426, BLACK, 428, DARK_RED, &tile_set),
            Self::tile(texture_creator, 427, BLACK, 428, DARK_RED, &tile_set),
            Self::tile(texture_creator, 429, BLUE, 445, BLUE, &tile_set),
            Self::tile(texture_creator, 430, CRIMSON, 446, CRIMSON, &tile_set),
            Self::tile(texture_creator, 431, GREEN, 447, GREEN, &tile_set),
            Self::tile(texture_creator, 460, PURPLE, 476, PURPLE, &tile_set),
            Self::tile(texture_creator, 461, PURPLE, 477, PURPLE, &tile_set),
            Self::tile(texture_creator, 462, PURPLE, 478, PURPLE, &tile_set),
            Self::tile(texture_creator, 463, PURPLE, 479, PURPLE, &tile_set),
            Self::tile(texture_creator, 491, BROWN, 509, BROWN, &tile_set),
            Self::tile(texture_creator, 507, rgba(17,15,164,255), 509, BROWN, &tile_set),
            Self::tile(texture_creator, 493, GREEN, 509, BROWN, &tile_set),
            Self::tile(texture_creator, 494, ORANGE, 509, BROWN, &tile_set),
            Self::tile(texture_creator, 492, ORANGE, 508, ORANGE, &tile_set),
            Self::tile(texture_creator, 492, BROWN, 508, BROWN, &tile_set),
            Self::tile(texture_creator, 492, NEUTRAL_GRAY, 508, NEUTRAL_GRAY, &tile_set),
            Self::tile(texture_creator, 492, GREEN, 508, GREEN, &tile_set),
        ];
        GraphicsSet {
            tile_set: tile_set,table,
            win, tile_blank, tile_depth, tile_outline,tiles,tile_highlight,tile_selected,tile_hover, tile_hint
        }
    }

}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HighlightType {
    Free, Selected, Hover, Hint
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MTile {
    pub value : u8
}
impl MTile {
    pub fn deck() -> Vec<MTile> {
        let mut ret = Vec::new();
        let mut suits : Vec<u8> = Vec::new();
        for i in 0..36 {
            suits.push(i);
        }
        suits.shuffle(&mut thread_rng());
        for i in suits {
            if i == 34 {
                ret.push(MTile{value:34});
                ret.push(MTile{value:35});
                ret.push(MTile{value:36});
                ret.push(MTile{value:37});
            } else if i == 35 {
                ret.push(MTile{value:38});
                ret.push(MTile{value:39});
                ret.push(MTile{value:40});
                ret.push(MTile{value:41});
            } else {
                ret.push(MTile{value:i});
                ret.push(MTile{value:i});
                ret.push(MTile{value:i});
                ret.push(MTile{value:i});
            }
        }
        ret
    }
    pub fn matches(t : MTile, u : MTile) -> bool {
        t.value == u.value || (38..=41).contains(&t.value) && (38..=41).contains(&u.value)
                           || (34..=37).contains(&t.value) && (34..=37).contains(&u.value)
    }
    fn draw_base<'r>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>, position : (i32,i32)) {
        for offset in 0..2 {
            graphics.tile_outline.draw(canvas,(position.0+offset-1,position.1+offset-1));
            graphics.tile_outline.draw(canvas,(position.0+offset-1,position.1+offset+1));
            graphics.tile_outline.draw(canvas,(position.0+offset+1,position.1+offset-1));
        }
        for offset in 0..2 {
            graphics.tile_depth.draw(canvas,(position.0+offset,position.1+offset));
        }
    }
    fn draw<'r>(&self, canvas: &mut Canvas<Window>, highlight: Option<HighlightType>, graphics: &GraphicsSet<Texture<'r>>, position : (i32,i32)) {
        graphics.tile_outline.draw(canvas,(position.0+1,position.1+1));
        graphics.tile_outline.draw(canvas,(position.0+2,position.1+1));
        graphics.tile_outline.draw(canvas,(position.0+3,position.1+1));
        graphics.tile_outline.draw(canvas,(position.0+1,position.1+2));
        graphics.tile_outline.draw(canvas,(position.0+3,position.1+2));
        graphics.tile_outline.draw(canvas,(position.0+1,position.1+3));
        graphics.tile_outline.draw(canvas,(position.0+2,position.1+3));
        graphics.tile_outline.draw(canvas,(position.0+3,position.1+3));
        match highlight {
            None => graphics.tile_blank.draw(canvas,(position.0+2,position.1+2)),
            Some(HighlightType::Free) => graphics.tile_highlight.draw(canvas,(position.0+2,position.1+2)),
            Some(HighlightType::Selected) => graphics.tile_selected.draw(canvas,(position.0+2,position.1+2)),
            Some(HighlightType::Hover) => graphics.tile_hover.draw(canvas,(position.0+2,position.1+2)),
            Some(HighlightType::Hint) => graphics.tile_hint.draw(canvas,(position.0+2,position.1+2)),
        }
        graphics.tiles[self.value as usize].draw(canvas,(position.0+2+8,position.1+2+8));
    }

}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum SenDirection {
    N, S, E, W
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Sen {
    direction: SenDirection,
    x: i32,
    y: i32, 
    turns: usize,
    destination: (i32,i32),
}
impl Sen {
    fn cost(&self) -> (i32,i32) {
        (self.turns as i32,(self.destination.0 - self.x).abs() + (self.destination.1 - self.y).abs())
    }
    fn straight_on(&self) -> Sen {
        Sen {
            direction: self.direction,
            x: match self.direction {
                SenDirection::E => self.x + 2,
                SenDirection::W => self.x - 2,
                _ => self.x,
            },
            y: match self.direction {
                SenDirection::N => self.y - 2,
                SenDirection::S => self.y + 2,
                _ => self.y
            },
            turns: self.turns,
            destination: self.destination
        }
    }
    fn turn_left(&self) -> Sen {
        Sen {
            direction: match self.direction {
                SenDirection::N => SenDirection::W,
                SenDirection::W => SenDirection::S,
                SenDirection::S => SenDirection::E,
                SenDirection::E => SenDirection::N
            },
            x: self.x, 
            y: self.y,
            turns: self.turns+1,
            destination: self.destination
        }.straight_on()
    }
    fn turn_right(&self) -> Sen {
        Sen {
            direction: match self.direction {
                SenDirection::S => SenDirection::W,
                SenDirection::E => SenDirection::S,
                SenDirection::N => SenDirection::E,
                SenDirection::W => SenDirection::N
            },
            x: self.x, 
            y: self.y,
            turns: self.turns+1,
            destination: self.destination
        }.straight_on()
    }
}
impl PartialOrd for Sen {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.cost().cmp(&self.cost()))
    }
}
impl Ord for Sen {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost().cmp(&self.cost())
    }
}
pub struct Table {    
    tiles: [[[Option<MTile>;36];32];8],
    frees: [[[bool;36];32];8],
    layout: [[[bool;36];32];8],
    selected: Option<(usize,usize,usize)>,
    mouseover: Option<(usize,usize,usize)>,
    hint_cells: Vec<(usize,usize,usize)>,
    history: Vec<((usize,usize,usize),(usize,usize,usize),MTile,MTile)>,
    gravity_history: Vec<Vec<(usize,usize)>>,
    display_frees: bool,
    game_won: bool,
    move_count: usize,
    shisen: bool,
    gravity: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Layout {
    Turtle, Cube, Bridge, Castle, Pyramid, ShisenSho, ShisenGravity
}
impl Table {
    fn clear_layout(&mut self) {
        self.shisen = false;
        self.gravity = false;
        for z in 0..8 {
            for x in 0..36 {
                for y in 0..32 {
                    self.layout[z][y][x] = false;
                    self.tiles[z][y][x] = None;
                }
            }
        }
    }
    fn cube_layout(&mut self) {
        self.clear_layout();
        for x in 0..6 {
            for y in 0..6 {
                for z in 0..4 {
                    self.layout[z][4+y*2][10+x*2] = true;
                }
            }
        }

    }
    fn pyramid_layout(&mut self) {
        let midpoint = 8;
        for z in 0..4 {
            let mut descending = false;   
            let mut current_height = 1;     
            for x in 0..(7-z)*2-1 {
                for y in 0..current_height {
                    self.layout[z][midpoint-current_height+(y*2)+3][(z+x)*2+3] = true;
                }
                if descending {
                    current_height -= 1;
                } else {
                    if current_height == 7-z {
                        descending = true;
                        current_height -= 1;
                    } else {
                        current_height += 1;
                    }
                }
            }
        }
        self.layout[4][9][14] = true;
        self.layout[4][9][16] =  true;
        self.layout[4][11][14] = true;
        self.layout[4][11][16] = true;
        self.layout[4][7][15] = true;
        self.layout[4][13][15] = true;
        for z in 0..3 {
            self.layout[z][4][2] = true;
            self.layout[z][16][2] = true;
            self.layout[z][4][28] = true;
            self.layout[z][16][28] = true;
        }
    }
    fn bridge_layout(&mut self) {
        self.clear_layout();
        for (x,y) in vec![ (2,8),(4,8),(6,8),(20,8),(22,8),(24,8)] {
            self.layout[0][y-1][x+2] = true;
            self.layout[0][y+1][x+2] = true;
            self.layout[0][y+3][x+2] = true;
            self.layout[0][y+5][x+2] = true;
        }
        for (x,y) in vec![ (4,8),(6,8),(8,8),(18,8),(20,8),(22,8)] {
            self.layout[1][y-1][x+2] = true;
            self.layout[1][y+1][x+2] = true;
            self.layout[1][y+3][x+2] = true;
            self.layout[1][y+5][x+2] = true;
        }
        for (x,y) in vec![ (6,8),(8,8),(10,8),(16,8),(18,8),(20,8)] {
            self.layout[2][y-1][x+2] = true;
            self.layout[2][y+1][x+2] = true;
            self.layout[2][y+3][x+2] = true;
            self.layout[2][y+5][x+2] = true;
        }
        for (x,y) in vec![ (8,8),(10,8),(12,8),(14,8),(16,8),(18,8)] {
            self.layout[3][y-1][x+2] = true;
            self.layout[3][y+1][x+2] = true;
            self.layout[3][y+3][x+2] = true;
            self.layout[3][y+5][x+2] = true;
        }
        for z in 1..6 {
            self.layout[z][3+4][4] = true;
            self.layout[z][3+4][26] = true;
            self.layout[z][9+4][4] = true;
            self.layout[z][9+4][26] = true;
        }
        self.layout[5][5+4][4] = true;
        self.layout[5][7+4][4] = true;
        self.layout[5][5+4][26] = true;
        self.layout[5][7+4][26] = true;
        for (z,x) in vec![(2,4),(3,6),(4,8),(4,10),(4,12),(4,14),(4,16),(4,18),(3,20),(2,22)] {
            self.layout[z][7][x+2] = true;
            self.layout[z][13][x+2] = true;
        }
        self.layout[5][7][15] = true;
        self.layout[6][7][15] = true;
        self.layout[5][13][15] = true;
        self.layout[6][13][15] = true;
    }
    fn shisen_sho(&mut self) {
        self.clear_layout();
        for y in 0..8 {
            for x in 0..18 {
                self.layout[0][y*2+3][x*2] = true;
            }
        }
        self.shisen = true;
    }
    fn castle_layout(&mut self) {
        self.clear_layout();
        for (x,y) in vec![
            (2,4),(4,4),(6,4),(8,4),(10,4),(12,4),(14,4),(16,4),(18,4),
            (2,6),(18,6),
            (2,8),(18,8),
            (2,10),(18,10),
            (2,12),(18,12),
            (2,14),(18,14),
            (2,16),(4,16),(6,16),(8,16),(10,16),(12,16),(14,16),(16,16),(18,16),
            (8,8),(10,8),(12,8),
            (8,12),(10,12),(12,12),
            (8,10),(12,10),
        ] {
            let offset = if x == 10 && (y == 4 || y == 16) { 2 } else { 0 };
            self.layout[0+offset][y-1][x+6] = true;
            self.layout[1+offset][y-1][x+6] = true;
            self.layout[2+offset][y-1][x+6] = true
        }
        for (x,y) in vec![
            (2,4),(6,4),(14,4),(18,4),
            (2,16),(6,16),(14,16),(18,16),
            (2,8),(2,12),(18,8),(18,12),
        ] {
            self.layout[3][y-1][x+6] = true;
            self.layout[4][y-1][x+6] = true;
        }
        for (x,y) in vec![
            (10,8),(10,12),(8,10),(12,10),
            (7,7),(13,7),(13,13),(7,13)
        ] {
            self.layout[3][y-1][x+6] = true;
            if x % 2 != 0 { self.layout[4][y-1][x+6] = true }
        }
    }
    fn turtle_layout(&mut self) {
        self.clear_layout();
        for (x,y) in vec![
            (2,4),(4,4),(6,4),(8,4),(10,4),(12,4),(14,4),(16,4),(18,4),(20,4),(22,4),(24,4),
            (6,6),(8,6),(10,6),(12,6),(14,6),(16,6),(18,6),(20,6),
            (4,8),(6,8),(8,8),(10,8),(12,8),(14,8),(16,8),(18,8),(20,8),(22,8),
            (2,10),(4,10),(6,10),(8,10),(10,10),(12,10),(14,10),(16,10),(18,10),(20,10),(22,10),(24,10),
            (2,12),(4,12),(6,12),(8,12),(10,12),(12,12),(14,12),(16,12),(18,12),(20,12),(22,12),(24,12),
            (0,11),(26,11),(28,11),
            (4,14),(6,14),(8,14),(10,14),(12,14),(14,14),(16,14),(18,14),(20,14),(22,14),
            (6,16),(8,16),(10,16),(12,16),(14,16),(16,16),(18,16),(20,16),
            (2,18),(4,18),(6,18),(8,18),(10,18),(12,18),(14,18),(16,18),(18,18),(20,18),(22,18),(24,18)
        ] {
            self.layout[0][y-1][x+2] = true;
        }
        for y in 0..6 {
            for x in 0..6 {
                self.layout[1][5+y*2][10+x*2] = true;
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                self.layout[2][7+y*2][12+x*2] = true;
            }
        }
        for y in 0..2 {
            for x in 0..2 {
                self.layout[3][9+y*2][14+x*2] = true;
            }
        }
        self.layout[4][10][15] = true;
    } 
    fn is_free_right(&self,z:usize,y:usize,x:usize) -> bool {
        if x < 34 {
            let xx = x + 2;
            if self.tiles[z][y][xx].is_some() { return false }
            if y < 31 && self.tiles[z][y+1][xx].is_some() { return false }
            if y > 0 && self.tiles[z][y-1][xx].is_some() { return false }
        }
        return true
    }
    fn is_free_left(&self,z:usize,y:usize,x:usize) -> bool {
        if x > 1 {
            let xx = x - 2;
            if self.tiles[z][y][xx].is_some() { return false }
            if y < 31 && self.tiles[z][y+1][xx].is_some() { return false }
            if y > 0 && self.tiles[z][y-1][xx].is_some() { return false }
        }
        return true
    }
    fn shisen_apply_gravity(&mut self) {
        let mut shifts = Vec::new();
        let mut changed = true;
        while changed {
            changed = false;
            for y in 0..7 {
                for x in 0..18 {
                    let yy = y*2+3;
                    let xx = x*2;
                    if self.tiles[0][yy][xx].is_some() && self.tiles[0][yy+2][xx].is_none() {
                        changed = true;
                        self.tiles[0][yy+2][xx] = Some(self.tiles[0][yy][xx].unwrap());
                        self.tiles[0][yy][xx] = None;
                        shifts.push((yy+2,xx))
                    }
                }
            }
        }
        shifts.reverse();
        self.gravity_history.push(shifts);
    }
    fn shisen_free(&self,y:i32,x:i32) -> bool {
        if y < -2 || x < -2 || x >= 38 || y >= 34 { false } 
        else { 
        if y < 0 || x < 0 || x >= 36 || y >= 32 { true } else { self.tiles[0][y as usize][x as usize].is_none() }
        }
    }
    fn shisen_path(&self,yy:i32,xx:i32,y:i32,x:i32) -> bool {
        let mut heap = BinaryHeap::new();
        let mut visited = HashMap::new();
        if self.shisen_free(yy-2,xx) || (yy-2,xx) == (y,x) { 
            let x = Sen {x: xx, y: yy-2, direction: SenDirection::N, destination: (x,y), turns: 0};
            heap.push(x);
            visited.insert((x.y,x.x,x.direction),x.cost());
        }
        if self.shisen_free(yy+2,xx) || (yy+2,xx) == (y,x)  { 
            let x = Sen {x: xx, y: yy+2, direction: SenDirection::S, destination: (x,y), turns: 0};
            heap.push(x);
            visited.insert((x.y,x.x,x.direction),x.cost());
        }
        if self.shisen_free(yy,xx+2) || (yy,xx+2) == (y,x)  { 
            let x = Sen {x: xx+2, y: yy, direction: SenDirection::E, destination: (x,y), turns: 0};
            heap.push(x);
            visited.insert((x.y,x.x,x.direction),x.cost());
        }
        if self.shisen_free(yy,xx-2) || (yy,xx-2) == (y,x) { 
            let x = Sen {x: xx-2, y: yy, direction: SenDirection::W, destination: (x,y), turns: 0};
            heap.push(x);
            visited.insert((x.y,x.x,x.direction),x.cost());
        }
        while let Some(st) = heap.pop() {
            if (st.x,st.y) == (x,y) {
                return true
            }
            for neighbour in [st.straight_on(),st.turn_left(),st.turn_right()].iter() {
                if (self.shisen_free(neighbour.y, neighbour.x) || (neighbour.y == y && neighbour.x == x)) && visited.get(&(neighbour.y,neighbour.x,neighbour.direction)).unwrap_or(&(999,0)) > &neighbour.cost() && neighbour.turns < 3 {
                    heap.push(*neighbour);
                    visited.insert((neighbour.y,neighbour.x,neighbour.direction),neighbour.cost());
                }
            }
        }
        false
        //if self.shisen_free(yy+2,xx) { destinations.push((yy+2,xx))};
        //if self.shisen_free(yy,xx+2) { destinations.push((yy,xx+2))};
        //if self.shisen_free(yy,xx-2) { destinations.push((yy,xx-2))};

    }
    fn check_match(&self, zz:usize,yy:usize,xx:usize,z:usize,y:usize,x:usize) -> bool {
        if let Some(t) = self.tiles[zz][yy][xx] {
            if (z,y,x) != (zz,yy,xx) {
                if self.frees[z][y][x] {
                    if let Some(u) = self.tiles[z][y][x] {
                        if MTile::matches(t, u) {
                            return !self.shisen || self.shisen_path(yy as i32,xx as i32,y as i32,x as i32); //
                        }
                    }
                }
            }
            return false;
        } else { false }
    }
    fn find_match_for(&self, zz: usize,yy:usize,xx:usize) -> Option<(usize,usize,usize)> {
        for z in (0..8).rev() {
            for y in 0..32 {
                for x in 0..36 {
                    if self.check_match(zz,yy,xx,z,y,x) {
                        return Some((z,y,x))
                    }
                }
            }
        }
        return None
    }
    fn populate_board(&mut self) {        
        for z in 0..8 {
            for y in 0..32 {
                for x in 0..36 {
                    if self.layout[z][y][x] {
                        self.tiles[z][y][x] = Some(MTile {value: (((y-3)/2)*18+(x/2)) as u8 });
                    } else {
                        self.tiles[z][y][x] = None;
                    }
                }
            }
        }
        let mut moves = Vec::new();
        loop {
            if self.gravity { self.shisen_apply_gravity() }
            self.recalculate_frees();
            let mut locations = Vec::new();
            for z in 0..8 {
                for y in 0..32 {
                    for x in 0..36 {
                        if self.tiles[z][y][x].is_some() && self.frees[z][y][x] {
                            locations.push((z,y,x));
                        }
                    }
                }
            }
            if locations.len() < 2 { break } 
            locations.shuffle(&mut thread_rng());
            let (z1,y1,x1) = locations.pop().unwrap();
            let (z2,y2,x2) = if self.shisen {
                let mut result = None;
                for i in 0..locations.len() {
                    if self.shisen_path(y1 as i32, x1 as i32, locations[i].1 as i32, locations[i].2 as i32) {
                        result = Some(locations[i]);
                        locations.remove(i);  
                        break;                      
                    }
                }
                result.unwrap()
            } else {
                locations.pop().unwrap()
            };
            let (yy1,xx1,yy2,xx2) = if self.gravity {
                let v1 = self.tiles[z1][y1][x1].unwrap().value as usize;
                let v2 = self.tiles[z2][y2][x2].unwrap().value as usize;
                (v1/18*2+3,v1%18*2,v2/18*2+3,v2%18*2)
            } else {
                (y1,x1,y2,x2)
            };
            self.tiles[z1][y1][x1] = None;
            self.tiles[z2][y2][x2] = None;
            moves.push(((z1,yy1,xx1),(z2,yy2,xx2)));
        }
        if self.game_won {
            self.game_won = false; 
            let mut deck = MTile::deck();
            for ((z1,y1,x1),(z2,y2,x2)) in moves {
                self.tiles[z1][y1][x1] = deck.pop();
                self.tiles[z2][y2][x2] = deck.pop();
            }
            self.gravity_history = vec![];
        } else {
            self.populate_board();
        }
    }
    fn hint(&mut self) {
        for z in (0..8).rev() {
            for y in 0..32 {
                for x in 0..36 {
                    if self.tiles[z][y][x].is_some() && self.frees[z][y][x] {
                        if let Some((zz,yy,xx)) = self.find_match_for(z,y,x) {
                            self.hint_cells = Vec::new();
                            self.hint_cells.push((zz,yy,xx));
                            self.hint_cells.push((z,y,x));
                            return
                        }
                    }
                }
            }
        }
    }
    fn is_free_top(&self, z:usize,y:usize,x:usize) -> bool {
        if z < 7 { 
            for zz in z+1..8 {
                if self.tiles[zz][y][x].is_some() { return false }
                if x > 0 && self.tiles[zz][y][x-1].is_some() { return false }
                if x < 35 && self.tiles[zz][y][x+1].is_some() { return false }
                if y > 0 && self.tiles[zz][y-1][x].is_some() { return false }
                if y < 31 && self.tiles[zz][y+1][x].is_some() { return false }
                if y > 0 && x > 0 && self.tiles[zz][y-1][x-1].is_some() { return false }
                if y > 0 && x < 35 && self.tiles[zz][y-1][x+1].is_some() { return false }
                if y < 31 && x > 0 && self.tiles[zz][y+1][x-1].is_some() { return false }
                if y < 31 && x < 35 && self.tiles[zz][y+1][x+1].is_some() { return false }
            }
        }
        return true
    }
    fn is_free(&self, z:usize,y:usize,x:usize) -> bool {
        if self.shisen { return true };
        self.is_free_top(z,y,x) && (self.is_free_left(z,y,x) || self.is_free_right(z,y,x))
    }
    fn recalculate_frees(&mut self) {
        self.hint_cells = Vec::new();
        self.game_won = true;
        for z in 0..8 {
            for y in 0..32 {
                for x in 0..36 {
                    if self.tiles[z][y][x].is_some() {
                        self.game_won = false;
                        self.frees[z][y][x] = self.is_free(z,y,x);
                    } else {
                        self.frees[z][y][x] = false;
                    }
                }
            }
        }
    }
    
    fn new(layout:Layout) -> Table {
        let mut table = Table {
            tiles: [[[None;36];32];8],
            frees: [[[false;36];32];8],
            layout: [[[false;36];32];8],
            selected: None,
            mouseover: None,
            hint_cells: Vec::new(),
            game_won: false,
            display_frees: true,
            history: Vec::new(),
            gravity_history: Vec::new(),
            move_count:0,
            shisen: false,
            gravity: false,
        };
        match layout {
            Layout::Cube => table.cube_layout(),
            Layout::Turtle => table.turtle_layout(),
            Layout::Bridge => table.bridge_layout(),
            Layout::Castle => table.castle_layout(),
            Layout::Pyramid => table.pyramid_layout(),
            Layout::ShisenSho => table.shisen_sho(),
            Layout::ShisenGravity => { table.shisen_sho(); table.gravity = true}
        };
        table.populate_board();
        table.recalculate_frees();
        table
    }
    fn undo(&mut self) {
        self.deselect();
        if let Some(((z,y,x),(zz,yy,xx),t,u)) = self.history.pop() {
            if self.gravity {
                if let Some(ls) = self.gravity_history.pop() {
                    for (y,x) in ls {
                        self.tiles[0][y-2][x] = Some(self.tiles[0][y][x].unwrap());
                        self.tiles[0][y][x] = None;
                    }
                }
            }
            self.tiles[z][y][x] = Some(t);
            self.tiles[zz][yy][xx] = Some(u);
            self.recalculate_frees();        
        }
    }
    fn clicked(&mut self, position:Option<(usize,usize,usize)>) {
        if let Some((z,y,x)) = position {        
            if position != self.selected {
                if self.frees[z][y][x] {
                    if let Some((zz,yy,xx)) = self.selected {
                        self.selected = position;
                        if let Some(t) = self.tiles[z][y][x] {
                            if let Some(u) = self.tiles[zz][yy][xx] {
                                if self.check_match(zz, yy, xx, z, y, x)  {
                                    self.tiles[z][y][x] = None;
                                    self.tiles[zz][yy][xx] = None;
                                    self.move_count+= 1;
                                    self.history.push(((z,y,x),(zz,yy,xx),t,u));
                                    self.selected = None;
                                    if self.gravity { self.shisen_apply_gravity(); }
                                    self.recalculate_frees();
                                } else {
                                    self.selected = position;
                                }
                            }
                        }
                    } else {
                        self.selected = position;
                    }
                }
            }
        } else {
            self.selected = None;
        }
    }
    fn deselect(&mut self) {
        self.selected = None;
    }
    fn draw<'t>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'t>>) {
        graphics.table.draw(canvas,(0,0));
        let xshift = if self.shisen { -9 } else {0};
        for z in 0..8 {
            for y in (0..32).rev() {
                for x in (0..36).rev() {
                    if let Some(t) = self.tiles[z][y][x] {
                        t.draw_base(canvas,graphics,(x as i32*7+(z as i32*2)+xshift,y as i32*11+(z as i32*2)));
                    }
                }
            }
            for y in (0..32).rev() {
                for x in (0..36).rev() {
                    if let Some(t) = self.tiles[z][y][x] {
                        let hl = 
                            if self.selected == Some((z,y,x)) { Some(HighlightType::Selected) } 
                            else if self.hint_cells.contains(&(z,y,x)) { Some (HighlightType::Hint) }
                            else if self.display_frees && self.frees[z][y][x] { if self.mouseover == Some((z,y,x)) { Some(HighlightType::Hover) } else { Some(HighlightType::Free) } } 
                            else if !self.display_frees && self.mouseover == Some((z,y,x)) { Some(HighlightType::Hover) } else  { None };
                        t.draw(canvas, hl, graphics, (x as i32*7+(z as i32*2)+xshift,y as i32*11+(z as i32*2)));
                    }
                }
            }

        }
    }

    fn collides_tile(&self, position: (i32,i32)) -> Option<(usize, usize, usize)> {
        let xshift = if self.shisen { 9 } else {0};
        for z in (0..8).rev() {
            let bx = ((position.0 + xshift - 6 - z as i32 * 2) / 7).max(0) as usize;
            let by = ((position.1 - 6 - z as i32 * 2) / 11).max(0) as usize;
            if bx > 35 || by > 31 { continue };
            if self.tiles[z][by][bx].is_some() {
                return Some((z,by,bx))
            }
            if by > 0 && self.tiles[z][by - 1][bx].is_some() {
                return Some((z,by-1,bx))
            }
            if bx > 0 && self.tiles[z][by][bx-1].is_some() {
                return Some((z,by,bx-1))
            }
            if by > 0 && bx > 0 && self.tiles[z][by - 1][bx-1].is_some() {
                return Some((z,by-1,bx-1))
            }
        }
        None
    }
}
const WIDTH:u32=248;
fn main_loop(mut window:Window, sdl_context: &Sdl) {
    window.set_size(WIDTH,240).unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    
    canvas.set_logical_size(WIDTH,240).unwrap();
    canvas.set_blend_mode(BlendMode::Blend);
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let graphics_set = GraphicsSet::new(&texture_creator);
    let mut layout = Layout::Turtle;
    let mut table = Table::new(layout);

    let mut move_count_gfx = Graphic::blank(4,1).textured(&texture_creator);
    let mut move_count_gfx_shadow = Graphic::blank(4,1).textured(&texture_creator);
    let mut menu = MenuBar::new(WIDTH)
                    .add(Menu::new("GAME",152,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Turtle", 352, Keycode::F1,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Cube", 353, Keycode::F2,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Bridge", 354, Keycode::F3,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Castle", 355, Keycode::F4,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Pyramid", 356, Keycode::F5,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(136, &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Shisen-sho", 357, Keycode::F6,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Shisen Gravity", 358, Keycode::F7,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(136, &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Quit",363, Keycode::F12,&texture_creator,&graphics_set.tile_set)))
                    .add(Menu::new("ACTION",88,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Hint",9, Keycode::H,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Undo",27, Keycode::Z,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(72, &texture_creator, &graphics_set.tile_set))
                            .add(MenuItem::new("Restart",15, Keycode::N,&texture_creator,&graphics_set.tile_set)))
                    .add(Menu::new("VIEW",96+32,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Toggle Frees",359, Keycode::F8,&texture_creator, &graphics_set.tile_set))
                            .add(MenuItem::separator(96+32-16, &texture_creator, &graphics_set.tile_set))
                            .add(MenuItem::new("Micro-mode",360, Keycode::F9,&texture_creator, &graphics_set.tile_set)));
    let mut rate_limiter = FPSManager::new();
    rate_limiter.set_framerate(60).unwrap();
    let mut micro_mode = false;
    loop {
        canvas.set_draw_color(CHARCOAL);
        canvas.clear();
        table.draw(&mut canvas, &graphics_set);
        move_count_gfx.draw_rect(0, 0, 4, 1, Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:0});
        move_count_gfx.draw_text(&table.move_count.to_string(), &graphics_set.tile_set , 0, 0, WHITE, TRANSPARENT);
        move_count_gfx_shadow.draw_rect(0, 0, 4, 1, Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:0});
        move_count_gfx_shadow.draw_text(&table.move_count.to_string(), &graphics_set.tile_set , 0, 0, BLACK, TRANSPARENT);
        move_count_gfx.update_texture(&graphics_set.tile_set);
        move_count_gfx_shadow.update_texture(&graphics_set.tile_set);
        move_count_gfx_shadow.draw(&mut canvas, (10,240-9-8));
        move_count_gfx_shadow.draw(&mut canvas, (10,240-11-8));
        move_count_gfx_shadow.draw(&mut canvas, (9,240-10-8));
        move_count_gfx_shadow.draw(&mut canvas, (11,240-10-8));
        move_count_gfx.draw(&mut canvas, (10,240-10-8));
        let won = table.game_won;
        if won {
            let x = WIDTH as i32 / 2 - (21 * 4);
            let y = 240 / 2 - (21 * 4);
            graphics_set.win.draw(&mut canvas, (x,y));
        }
        menu.draw(&mut canvas);
        canvas.present();
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
                    Event::KeyDown { keycode: Some(Keycode::F1), ..} => {
                        layout = Layout::Turtle;
                        let tog = table.display_frees;
                        table = Table::new(layout);
                        table.display_frees = tog;
                    },
                    Event::KeyDown { keycode: Some(Keycode::F2), ..} => {
                        layout = Layout::Cube;
                        let tog = table.display_frees;
                        table = Table::new(layout);
                        table.display_frees = tog;
                    },
                    Event::KeyDown { keycode: Some(Keycode::F3), ..} => {
                        layout = Layout::Bridge;
                        let tog = table.display_frees;
                        table = Table::new(layout);
                        table.display_frees = tog;
                    },
                    Event::KeyDown { keycode: Some(Keycode::F4), ..} => {
                        layout = Layout::Castle;
                        let tog = table.display_frees;
                        table = Table::new(layout);
                        table.display_frees = tog;
                    },
                    Event::KeyDown { keycode: Some(Keycode::F5), ..} => {
                        layout = Layout::Pyramid;
                        let tog = table.display_frees;
                        table = Table::new(layout);
                        table.display_frees = tog;
                    },
                    Event::KeyDown { keycode: Some(Keycode::F6), ..} => {
                        layout = Layout::ShisenSho;
                        let tog = table.display_frees;
                        table = Table::new(layout);
                        table.display_frees = tog;
                    },
                    Event::KeyDown { keycode: Some(Keycode::F7), ..} => {
                        layout = Layout::ShisenGravity;
                        let tog = table.display_frees;
                        table = Table::new(layout);
                        table.display_frees = tog;
                    },
                    Event::KeyDown { keycode: Some(Keycode::F8), ..} => {
                        table.display_frees = !table.display_frees;
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
                            canvas.window_mut().set_size(WIDTH,240).unwrap_or_default();
                        } else {
                            canvas.window_mut().set_size(WIDTH/2,240/2).unwrap_or_default();
                            micro_mode = true;
                        }
                    }
                    Event::KeyDown {..} if won => {
                        let tog = table.display_frees;
                        table = Table::new(layout);
                        table.display_frees = tog;
                    },
                    Event::MouseButtonUp { ..} if won => {
                        let tog = table.display_frees;
                        table = Table::new(layout);
                        table.display_frees = tog;
                    },
                    Event::KeyDown { keycode: Some(Keycode::N), ..} => {
                        let tog = table.display_frees;
                        table = Table::new(layout);
                        table.display_frees = tog;
                    },
                    Event::MouseButtonDown { mouse_btn: _, x, y, ..} if !won => {
                        table.mouseover = table.collides_tile((x,y));
                    }
                    Event::MouseMotion { x, y, ..} if !won => {
                        table.mouseover = table.collides_tile((x,y));
                    }

                    Event::MouseButtonUp { mouse_btn: _, x, y, ..} if !won => {                            
                        if table.mouseover == table.collides_tile((x,y)) {
                            table.clicked(table.mouseover);
                        } else {
                            table.deselect();
                        }
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
    let window = video_subsystem.window("tainan", WIDTH, 240)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    main_loop(window, &sdl_context);
}