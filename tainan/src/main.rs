
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
        for i in 0..34 {
            ret.push(MTile{value:i});
            ret.push(MTile{value:i});
            ret.push(MTile{value:i});
            ret.push(MTile{value:i});
        }
        for i in 34..42 {
            ret.push(MTile{value:i});
        }
        ret.shuffle(&mut thread_rng());
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
pub struct Table {    
    tiles: [[[Option<MTile>;32];32];8],
    frees: [[[bool;32];32];8],
    selected: Option<(usize,usize,usize)>,
    mouseover: Option<(usize,usize,usize)>,
    hint_cells: Vec<(usize,usize,usize)>,
    display_frees: bool,
    game_won: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Layout {
    Turtle, Cube, Bridge, Castle, Pyramid
}
impl Table {
    fn clear_table(&mut self) {
        for z in 0..8 {
            for x in 0..32 {
                for y in 0..32 {
                    self.tiles[z][y][x] = None;
                }
            }
        }
    }
    fn cube_layout(&mut self) {
        self.clear_table();
        let mut tiles = MTile::deck();
        for x in 0..6 {
            for y in 0..6 {
                for z in 0..4 {
                    self.tiles[z][4+y*2][10+x*2] = tiles.pop();
                }
            }
        }

    }
    fn pyramid_layout(&mut self) {
        self.clear_table();
        let mut tiles = MTile::deck();
        let midpoint = 8;
        for z in 0..4 {
            let mut descending = false;   
            let mut current_height = 1;     
            for x in 0..(7-z)*2-1 {
                for y in 0..current_height {
                    self.tiles[z][midpoint-current_height+(y*2)+3][(z+x)*2+3] = tiles.pop();
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
        self.tiles[4][9][14] = tiles.pop();
        self.tiles[4][9][16] = tiles.pop();
        self.tiles[4][11][14] = tiles.pop();
        self.tiles[4][11][16] = tiles.pop();
        self.tiles[4][7][15] = tiles.pop();
        self.tiles[4][13][15] = tiles.pop();
        for z in 0..3 {
            self.tiles[z][4][2] = tiles.pop();
            self.tiles[z][16][2] = tiles.pop();
            self.tiles[z][4][28] = tiles.pop();
            self.tiles[z][16][28] = tiles.pop();
        }
    }
    fn bridge_layout(&mut self) {
        self.clear_table();
        let mut tiles = MTile::deck();
        for (x,y) in vec![ (2,8),(4,8),(6,8),(20,8),(22,8),(24,8)] {
            self.tiles[0][y-1][x+2] = tiles.pop();
            self.tiles[0][y+1][x+2] = tiles.pop();
            self.tiles[0][y+3][x+2] = tiles.pop();
            self.tiles[0][y+5][x+2] = tiles.pop();
        }
        for (x,y) in vec![ (4,8),(6,8),(8,8),(18,8),(20,8),(22,8)] {
            self.tiles[1][y-1][x+2] = tiles.pop();
            self.tiles[1][y+1][x+2] = tiles.pop();
            self.tiles[1][y+3][x+2] = tiles.pop();
            self.tiles[1][y+5][x+2] = tiles.pop();
        }
        for (x,y) in vec![ (6,8),(8,8),(10,8),(16,8),(18,8),(20,8)] {
            self.tiles[2][y-1][x+2] = tiles.pop();
            self.tiles[2][y+1][x+2] = tiles.pop();
            self.tiles[2][y+3][x+2] = tiles.pop();
            self.tiles[2][y+5][x+2] = tiles.pop();
        }
        for (x,y) in vec![ (8,8),(10,8),(12,8),(14,8),(16,8),(18,8)] {
            self.tiles[3][y-1][x+2] = tiles.pop();
            self.tiles[3][y+1][x+2] = tiles.pop();
            self.tiles[3][y+3][x+2] = tiles.pop();
            self.tiles[3][y+5][x+2] = tiles.pop();
        }
        for z in 1..6 {
            self.tiles[z][3+4][4] = tiles.pop();
            self.tiles[z][3+4][26] = tiles.pop();
            self.tiles[z][9+4][4] = tiles.pop();
            self.tiles[z][9+4][26] = tiles.pop();
        }
        self.tiles[5][5+4][4] = tiles.pop();
        self.tiles[5][7+4][4] = tiles.pop();
        self.tiles[5][5+4][26] = tiles.pop();
        self.tiles[5][7+4][26] = tiles.pop();
        for (z,x) in vec![(2,4),(3,6),(4,8),(4,10),(4,12),(4,14),(4,16),(4,18),(3,20),(2,22)] {
            self.tiles[z][7][x+2] = tiles.pop();
            self.tiles[z][13][x+2] = tiles.pop();
        }
        self.tiles[5][7][15] = tiles.pop();
        self.tiles[6][7][15] = tiles.pop();
        self.tiles[5][13][15] = tiles.pop();
        self.tiles[6][13][15] = tiles.pop();
    }
    fn castle_layout(&mut self) {
        self.clear_table();
        let mut tiles = MTile::deck();
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
            self.tiles[0+offset][y-1][x+6] = tiles.pop();
            self.tiles[1+offset][y-1][x+6] = tiles.pop();
            self.tiles[2+offset][y-1][x+6] = tiles.pop();
        }
        for (x,y) in vec![
            (2,4),(6,4),(14,4),(18,4),
            (2,16),(6,16),(14,16),(18,16),
            (2,8),(2,12),(18,8),(18,12),
        ] {
            self.tiles[3][y-1][x+6] = tiles.pop();
            self.tiles[4][y-1][x+6] = tiles.pop();
        }
        for (x,y) in vec![
            (10,8),(10,12),(8,10),(12,10),
            (7,7),(13,7),(13,13),(7,13)
        ] {
            self.tiles[3][y-1][x+6] = tiles.pop();
            if x % 2 != 0 { self.tiles[4][y-1][x+6] = tiles.pop() }
        }
    }
    fn turtle_layout(&mut self) {
        self.clear_table();
        let mut tiles = MTile::deck();
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
            self.tiles[0][y-1][x+2] = tiles.pop();
        }
        for y in 0..6 {
            for x in 0..6 {
                self.tiles[1][5+y*2][10+x*2] = tiles.pop();
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                self.tiles[2][7+y*2][12+x*2] = tiles.pop();
            }
        }
        for y in 0..2 {
            for x in 0..2 {
                self.tiles[3][9+y*2][14+x*2] = tiles.pop();
            }
        }
        self.tiles[4][10][15] = tiles.pop();
    } 
    fn is_free_right(&self,z:usize,y:usize,x:usize) -> bool {
        if x < 30 {
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
    fn find_match_for(&self, zz: usize,yy:usize,xx:usize) -> Option<(usize,usize,usize)> {
        let t = self.tiles[zz][yy][xx].unwrap();
        for z in (0..8).rev() {
            for y in 0..32 {
                for x in 0..32 {
                    if (z,y,x) != (zz,yy,xx) {
                        if self.frees[z][y][x] {
                            if let Some(u) = self.tiles[z][y][x] {
                                if MTile::matches(t, u) {
                                    return Some((z,y,x))
                                }
                            }
                        }
                    }
                }
            }
        }
        return None
    }
    fn hint(&mut self) {
        for z in (0..8).rev() {
            for y in 0..32 {
                for x in 0..32 {
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
                if x < 31 && self.tiles[zz][y][x+1].is_some() { return false }
                if y > 0 && x > 0 && self.tiles[zz][y-1][x].is_some() { return false }
                if y > 0 && x < 31 && self.tiles[zz][y+1][x].is_some() { return false }
                if y > 0 && x > 0 && self.tiles[zz][y-1][x-1].is_some() { return false }
                if y > 0 && x < 31 && self.tiles[zz][y-1][x+1].is_some() { return false }
                if y < 31 && x > 0 && self.tiles[zz][y+1][x-1].is_some() { return false }
                if y < 31 && x < 31 && self.tiles[zz][y+1][x+1].is_some() { return false }
            }
        }
        return true
    }
    fn is_free(&self, z:usize,y:usize,x:usize) -> bool {
        self.is_free_top(z,y,x) && (self.is_free_left(z,y,x) || self.is_free_right(z,y,x))
    }
    fn recalculate_frees(&mut self) {
        self.hint_cells = Vec::new();
        self.game_won = true;
        for z in 0..8 {
            for y in 0..32 {
                for x in 0..32 {
                    if self.tiles[z][y][x].is_some() {
                        self.game_won = false;
                        self.frees[z][y][x] = self.is_free(z,y,x);
                    }
                }
            }
        }
    }
    
    fn new(layout:Layout) -> Table {
        let mut table = Table {
            tiles: [[[None;32];32];8],
            frees: [[[false;32];32];8],
            selected: None,
            mouseover: None,
            hint_cells: Vec::new(),
            game_won: false,
            display_frees: true,
        };
        match layout {
            Layout::Cube => table.cube_layout(),
            Layout::Turtle => table.turtle_layout(),
            Layout::Bridge => table.bridge_layout(),
            Layout::Castle => table.castle_layout(),
            Layout::Pyramid => table.pyramid_layout(),
        };
        table.recalculate_frees();
        table
    }
    fn clicked(&mut self, position:Option<(usize,usize,usize)>) {
        if let Some((z,y,x)) = position {        
            if position != self.selected {
                if self.frees[z][y][x] {
                    if let Some((zz,yy,xx)) = self.selected {
                        self.selected = position;
                        if let Some(t) = self.tiles[z][y][x] {
                            if let Some(u) = self.tiles[zz][yy][xx] {
                                if MTile::matches(t,u) {
                                    self.tiles[z][y][x] = None;
                                    self.tiles[zz][yy][xx] = None;
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
        for z in 0..8 {
            for y in (0..32).rev() {
                for x in (0..32).rev() {
                    if let Some(t) = self.tiles[z][y][x] {
                        t.draw_base(canvas,graphics,(x as i32*7+(z as i32*2),y as i32*11+(z as i32*2)));
                    }
                }
            }
            for y in (0..32).rev() {
                for x in (0..32).rev() {
                    if let Some(t) = self.tiles[z][y][x] {
                        let hl = 
                            if self.selected == Some((z,y,x)) { Some(HighlightType::Selected) } 
                            else if self.hint_cells.contains(&(z,y,x)) { Some (HighlightType::Hint) }
                            else if self.display_frees && self.frees[z][y][x] { if self.mouseover == Some((z,y,x)) { Some(HighlightType::Hover) } else { Some(HighlightType::Free) } } 
                            else if !self.display_frees && self.mouseover == Some((z,y,x)) { Some(HighlightType::Hover) } else  { None };
                        t.draw(canvas, hl, graphics, (x as i32*7+(z as i32*2),y as i32*11+(z as i32*2)));
                    }
                }
            }

        }
    }

    fn collides_tile(&self, position: (i32,i32)) -> Option<(usize, usize, usize)> {
        for z in (0..8).rev() {
            let bx = ((position.0 - 6 - z as i32 * 2) / 7).max(0) as usize;
            let by = ((position.1 - 6 - z as i32 * 2) / 11).max(0) as usize;
            if bx > 31 || by > 31 { continue };
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

    let mut menu = MenuBar::new(WIDTH)
                    .add(Menu::new("GAME",152,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Turtle", 352, Keycode::F1,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Cube", 353, Keycode::F2,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Bridge", 354, Keycode::F3,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Castle", 355, Keycode::F4,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Pyramid", 356, Keycode::F5,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(136, &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Quit",363, Keycode::F12,&texture_creator,&graphics_set.tile_set)))
                    .add(Menu::new("ACTION",88,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Hint",9, Keycode::H,&texture_creator,&graphics_set.tile_set))
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
                    Event::KeyDown { keycode: Some(Keycode::F8), ..} => {
                        table.display_frees = !table.display_frees;
                    },
                    Event::KeyDown { keycode: Some(Keycode::H), ..} => {
                        table.hint();
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