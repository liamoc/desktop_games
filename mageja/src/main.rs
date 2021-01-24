
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
use sdl2::rect::Rect;
use utils::color::{*};

struct GraphicsSet<T> {
    tile_set: TileSet,
    tile_top: [Graphic<T>;4],
    tile_bottom: [Graphic<T>;4],
    tile_left: [Graphic<T>;4],
    tile_right: [Graphic<T>;4],
    tile_base: [Graphic<T>;4],
    tile_bg: [Graphic<T>;4],
    tile_hover: [Graphic<T>;4],
    tile_unmoved: [Graphic<T>;4],
    table: Graphic<T>,
    win: Graphic<T>,
}
impl <'r> GraphicsSet<Texture<'r>> {
    const BROWN_DARK : Color = rgba(143, 89, 2, 255);
    const BROWN_LIGHT : Color = rgba(193, 125, 17, 255);
    fn one2<T>(texture_creator: &'r TextureCreator<T>, tile: usize, fg_color: Color, bg_color : Color, tile_set: &TileSet) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(1,1).textured(texture_creator);
        ret[(0,0)] = Tile { index: tile, fg: fg_color, bg: bg_color };        
        ret.update_texture(tile_set);
        ret
    }
    fn base_set<T>(texture_creator: &'r TextureCreator<T>, tile_set: &TileSet) -> [Graphic<Texture<'r>>;4] {
        [
            Self::one2(texture_creator, 193, CHARCOAL, CHARCOAL, &tile_set),
            Self::one2(texture_creator, 193, rgba(213,165,90,255), Self::BROWN_LIGHT, &tile_set),
            Self::one2(texture_creator, 193, PALE_ORANGE, ORANGE, &tile_set),
            Self::one2(texture_creator, 0, rgba(108,206,203,255), rgba(78,165,177,255), &tile_set),
        ]
    }
    fn bg_set<T>(texture_creator: &'r TextureCreator<T>, tile_set: &TileSet) -> [Graphic<Texture<'r>>;4] {
        [
            Self::one2(texture_creator, 255, CHARCOAL, CHARCOAL, &tile_set),
            Self::one2(texture_creator, 255, rgba(213,165,90,255), Self::BROWN_LIGHT, &tile_set),
            Self::one2(texture_creator, 255, rgba(206,92,0,255), ORANGE, &tile_set),
            Self::one2(texture_creator, 255, rgba(33,140,141,255), rgba(78,165,177,255), &tile_set),
        ]
    }
    fn hover_set<T>(texture_creator: &'r TextureCreator<T>,  tile_set: &TileSet) -> [Graphic<Texture<'r>>;4] {
        [
            Self::one2(texture_creator, 0, rgba(106,114,118,255), CHARCOAL, &tile_set),
            Self::one2(texture_creator, 193, WHITE, Self::BROWN_LIGHT, &tile_set),
            Self::one2(texture_creator, 193, WHITE, ORANGE, &tile_set),
            Self::one2(texture_creator, 193, rgba(108,206,203,255), rgba(78,165,177,255), &tile_set),
        ]
    }
    fn unmoved_set<T>(texture_creator: &'r TextureCreator<T>,  tile_set: &TileSet) -> [Graphic<Texture<'r>>;4] {
        [
            Self::one2(texture_creator, 254, CHARCOAL,WHITE, &tile_set),
            Self::one2(texture_creator, 254, Self::BROWN_LIGHT,WHITE, &tile_set),
            Self::one2(texture_creator, 254, ORANGE, WHITE, &tile_set),
            Self::one2(texture_creator, 254, rgba(78,165,177,255),WHITE, &tile_set),
        ]
    }
    fn new<T>(texture_creator: &'r TextureCreator<T>) -> GraphicsSet<Texture<'r>> {
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
        let mut win = Graphic::load_from(Cursor::new(&include_bytes!("../win")[..])).unwrap().textured(&texture_creator);
        win.update_texture(&tile_set);
        let mut table = Graphic::solid(640/8, 480/8, Tile {fg: rgba(127,90,133,255), bg:rgba(117,80,123,255), index:255}).textured(texture_creator);
        table.update_texture(&tile_set);
        let tile_unmoved = Self::unmoved_set(texture_creator, &tile_set);
        let tile_base = Self::base_set(texture_creator,  &tile_set);
        let tile_hover = Self::hover_set(texture_creator, &tile_set);
        let tile_bg = Self::bg_set(texture_creator, &tile_set);
        let tile_left = [
            Self::one2(texture_creator, 186, rgba(106,114,118,255), TRANSPARENT, &tile_set),
            Self::one2(texture_creator, 186,  rgba(213,165,90,255), TRANSPARENT, &tile_set),
            Self::one2(texture_creator, 186, PALE_ORANGE, TRANSPARENT, &tile_set),
            Self::one2(texture_creator, 186,  rgba(108,206,203,255), TRANSPARENT, &tile_set),
        ];
        let tile_right = [
            Self::one2(texture_creator, 184, DARK_CHARCOAL, TRANSPARENT, &tile_set),
            Self::one2(texture_creator, 184, Self::BROWN_DARK, TRANSPARENT, &tile_set),
            Self::one2(texture_creator, 184, rgba(206,92,0,255), TRANSPARENT, &tile_set),
            Self::one2(texture_creator, 184, rgba(33,140,141,255), TRANSPARENT, &tile_set),
        ];
        let tile_top = [
            Self::one2(texture_creator, 201, rgba(106,114,118,255), TRANSPARENT, &tile_set),
            Self::one2(texture_creator, 201,  rgba(213,165,90,255), TRANSPARENT, &tile_set),
            Self::one2(texture_creator, 201, PALE_ORANGE, TRANSPARENT, &tile_set),
            Self::one2(texture_creator, 201, rgba(108,206,203,255), TRANSPARENT, &tile_set),
        ];
        let tile_bottom = [
            Self::one2(texture_creator, 169, DARK_CHARCOAL, TRANSPARENT, &tile_set),
            Self::one2(texture_creator, 169, Self::BROWN_DARK, TRANSPARENT, &tile_set),
            Self::one2(texture_creator, 169, rgba(206,92,0,255), TRANSPARENT, &tile_set),
            Self::one2(texture_creator, 169, rgba(33,140,141,255), TRANSPARENT, &tile_set),
        ];
        GraphicsSet {
            tile_set: tile_set,table,
            win, tile_base, tile_left, tile_right, tile_top, tile_bottom, 
            tile_unmoved, tile_hover, tile_bg
        }
    }

}

pub struct Table {    
    tiles: [[u16;28];28],
    backdrop: [[u8;28];28],
    anim_tiles: [[u16;28];28],
    anim_direction: (i32,i32),
    anim_offset: (i32,i32),
    anim_left: usize,
    level: usize,
    blocked: Vec<u16>,
    collapsed: Vec<(usize,usize)>,
    collapseable: Vec<u16>,
    special: u16,
    hovering: Option<u16>,
    selected: Option<u16>,    
    origin: (usize,usize),
    over: (usize,usize),
    won_cell_offset: (i16,i16),
    game_won: bool,
    moved: bool,
    move_count: u32,
}

impl Table {
    fn level_from_graphic<T>(&mut self, g:&Graphic<T>)  {
        self.collapseable = Vec::new();
        self.special = 0;
        let mut dest_pos = (0,0);
        let mut special_pos = None;
        self.won_cell_offset = (0,0);
        for y in 0..28 {
            for x in 0..28 {
                let t = g[(x as u32,y as u32)];
                self.tiles[y][x] = t.index as u16;
                if t.fg.g > 200 {
                    // normal block
                } else if t.fg.b > 200  {
                    if !self.collapseable.contains(&(t.index as u16)) { self.collapseable.push (t.index as u16); }
                } else if t.fg.r > 200 {
                    if special_pos.is_none() {
                        self.special = t.index as u16;
                        special_pos = Some((x as i16,y as i16));
                    }
                } else if !self.blocked.contains(&(t.index as u16)) && t.index != 0 {
                    self.blocked.push(t.index as u16);
                }
                if t.bg.r == 255 {
                    dest_pos = (x as i16,y as i16);
                }
            }
        }
        if let Some((x,y)) = special_pos {
            self.won_cell_offset = (dest_pos.0 - x,dest_pos.1 - y);
        }
    }
    fn next_level(&self) -> usize {
        if self.level == 24 {
            self.level
        } else {
            self.level + 1
        }
    }
    fn previous_level(&self) -> usize {
        if self.level == 1 {
            self.level
        } else {
            self.level - 1
        }
    }
    fn level(&mut self,index:usize) {
        let bytes = match index {
            1 => include_bytes!("../levels/1"),
            2 => include_bytes!("../levels/2"),
            3 => include_bytes!("../levels/3"),
            4 => include_bytes!("../levels/4"),
            5 => include_bytes!("../levels/5"),
            6 => include_bytes!("../levels/6"),
            7 => include_bytes!("../levels/7"),
            8 => include_bytes!("../levels/8"),
            9 => include_bytes!("../levels/9"),
            10 => include_bytes!("../levels/10"),
            11 => include_bytes!("../levels/11"),
            12 => include_bytes!("../levels/12"),
            13 => include_bytes!("../levels/13"),
            14 => include_bytes!("../levels/14"),
            15 => include_bytes!("../levels/15"),
            16 => include_bytes!("../levels/16"),
            17 => include_bytes!("../levels/17"),
            18 => include_bytes!("../levels/18"),
            19 => include_bytes!("../levels/19"),
            20 => include_bytes!("../levels/20"),
            21 => include_bytes!("../levels/21"),
            22 => include_bytes!("../levels/22"),
            23 => include_bytes!("../levels/23"),
            _ => include_bytes!("../levels/24")
        };
        self.level_from_graphic(&Graphic::load_from(Cursor::new(&bytes[..])).unwrap());
    }
    fn new(index: usize) -> Table {
        let mut table = Table {
            tiles: [[0;28];28],
            backdrop: [[0;28];28],
            blocked: Vec::new(),
            collapseable: Vec::new(),
            special: 1,
            won_cell_offset: (1,1),
            game_won: false,
            collapsed: Vec::new(),
            hovering: None,
            origin: (0,0),
            over: (0,0),
            moved: false,
            selected: None,
            anim_offset: (0,0),
            anim_direction: (0,0),
            anim_left: 0,
            anim_tiles: [[0;28];28],
            move_count: 0,
            level: index,
        };
        table.level(index);
        table.set_up_goal();
        table
    }
    fn set_up_goal(&mut self) {
        for y in 0..28 {
            for x in 0..28 {
                if self.tiles[y][x] == self.special {
                    let ny = (self.won_cell_offset.1 + y as i16).max(0) as usize;
                    let nx = (self.won_cell_offset.0 + x as i16).max(0) as usize;
                    if ny < 28 && nx < 28 {
                        self.backdrop[ny][nx] = 2;
                    }
                }
            }
        }
    }
    fn can_move_left(&self) -> bool {
        for y in 0..28 {
            for x in 0..28 {
                if Some(self.tiles[y][x]) == self.selected {
                    if x == 0 { return false }
                    if self.tiles[y][x-1] != 0 && Some(self.tiles[y][x-1]) != self.selected {
                        return false;
                    }
                }
            }
        }
        true
    }
    fn can_move_right(&self) -> bool {
        for y in 0..28 {
            for x in 0..28 {
                if Some(self.tiles[y][x]) == self.selected {
                    if x == 27 { return false }
                    if self.tiles[y][x+1] != 0 && Some(self.tiles[y][x+1]) != self.selected {
                        return false;
                    }
                }
            }
        }
        true
    }
    fn can_move_up(&self) -> bool {
        for y in 0..28 {
            for x in 0..28 {
                if Some(self.tiles[y][x]) == self.selected {
                    if y == 0 { return false }
                    if self.tiles[y-1][x] != 0 && Some(self.tiles[y-1][x]) != self.selected {
                        return false;
                    }
                }
            }
        }
        true
    }
    fn can_move_down(&self) -> bool {
        for y in 0..28 {
            for x in 0..28 {
                if Some(self.tiles[y][x]) == self.selected {
                    if y == 27 { return false }
                    if self.tiles[y+1][x] != 0 && Some(self.tiles[y+1][x]) != self.selected {
                        return false;
                    }
                }
            }
        }
        true
    }
    fn can_move_x(&self, xdiff : i32) -> bool {
        if xdiff < 0 { self.can_move_left() } else { self.can_move_right() }
    }
    fn can_move_y(&self, ydiff : i32) -> bool {
        if ydiff < 0 { self.can_move_up() } else { self.can_move_down() }
    }
    fn transfer_to_anim_buffer(&mut self, id:u16 ) {
        for y in 0..28 {
            for x in 0..28 {
                if self.tiles[y][x] == id {
                    self.tiles[y][x] = 0;
                    self.anim_tiles[y][x] = id;
                } else {
                    self.anim_tiles[y][x] = 0;
                }
            }
        }
    }
    fn all_collapsed(&self, i : u16) -> bool {
        for y in 0..28 {
            for x in 0..28 {
                if self.tiles[y][x] == i && !self.collapsed.contains(&(x,y)) {
                    return false;
                }
            }
        }
        true
    }
    fn destroy_collapsed(&mut self, i : u16) {
        let mut to_remove = Vec::new();
        for y in 0..28 {
            for x in 0..28 {
                if self.tiles[y][x] == i {
                    self.tiles[y][x] = 0;
                    self.backdrop[y][x] = 3;
                    to_remove.push((x,y));
                }
            }
        }
        self.collapsed = self.collapsed.clone().into_iter().filter(|x| !to_remove.contains(&x)).collect();
        self.collapseable = self.collapseable.clone().into_iter().filter(|x| *x != i).collect();
    }
    fn transfer_from_anim_buffer(&mut self, offset:(i32,i32) ) {
        for y in 0..28 {
            for x in 0..28 {
                if self.anim_tiles[y][x] != 0 {
                    let ny = (y as i32 + offset.1).max(0) as usize;
                    let nx = (x as i32 + offset.0).max(0) as usize;
                    self.tiles[ny][nx] = self.anim_tiles[y][x];
                    self.anim_tiles[y][x] = 0;
                }
            }
        }
    }
    fn add_collapses(&mut self) {
        let mut c = true;
        while c {
            c = false;
            for y in 0..28 {
                for x in 0..28 {
                    if self.collapseable.contains(&self.tiles[y][x]) && !self.collapsed.contains(&(x,y)) {
                        let mut matched_neighbours_nw = 0;
                        let mut matched_neighbours_ne = 0;
                        let mut matched_neighbours_sw = 0;
                        let mut matched_neighbours_se = 0;
                        if x == 0 || self.tiles[y][x-1] == self.special || self.collapsed.contains(&(x-1,y)) || self.blocked.contains(&self.tiles[y][x-1]) {
                            matched_neighbours_nw += 1;
                            matched_neighbours_sw += 1;
                        } 
                        if y == 0 || self.tiles[y-1][x] == self.special || self.collapsed.contains(&(x,y-1)) || self.blocked.contains(&self.tiles[y-1][x]) {
                            matched_neighbours_nw += 1;
                            matched_neighbours_ne += 1;
                        }
                        if x == 27 || self.tiles[y][x+1] == self.special || self.collapsed.contains(&(x+1,y)) || self.blocked.contains(&self.tiles[y][x+1]) {
                            matched_neighbours_se += 1;
                            matched_neighbours_ne += 1;
                        }
                        if y == 27 || self.tiles[y+1][x] == self.special || self.collapsed.contains(&(x,y+1)) || self.blocked.contains(&self.tiles[y+1][x]) {
                            matched_neighbours_sw += 1;
                            matched_neighbours_se += 1;
                        }
                        if matched_neighbours_nw >= 2 || matched_neighbours_ne >= 2 || matched_neighbours_se >= 2 || matched_neighbours_sw >= 2 {
                            c = true;
                            self.collapsed.push((x,y));
                        }
                    }
                }
            }
        }
    }
    fn queue_moves(&mut self) {
        let x_raw = self.over.0 as i32 - self.origin.0 as i32;
        let y_raw = self.over.1 as i32 - self.origin.1 as i32;
        let xdiff = if x_raw.abs() > 0 { if self.can_move_x(x_raw) { x_raw } else { 0 } } else { 0 };
        let ydiff = if y_raw.abs() > 0 { if self.can_move_y(y_raw) { y_raw } else { 0 } } else { 0 };
        if xdiff.abs() > 0 || ydiff.abs() > 0 {
            if xdiff.abs() > ydiff.abs() {
                if xdiff > 0 {
                    self.anim_direction = (1,0);
                } else  {
                    self.anim_direction = (-1,0);
                }
                self.anim_left = 8;
                self.moved = true;
                self.anim_offset = (0,0);
                self.transfer_to_anim_buffer(self.selected.unwrap()); //scary
            } else if ydiff.abs() > 0 {
                if ydiff > 0 {
                    self.anim_direction = (0,1);
                } else  {
                    self.anim_direction = (0,-1);
                }
                self.anim_left = 8;
                self.moved = true;
                self.anim_offset = (0,0);
                self.transfer_to_anim_buffer(self.selected.unwrap()); //scary
            }
        }
    }
    fn mouse_moved(&mut self, x: i32, y:i32) {
        let coords_x = (x + 2) / 8;
        let coords_y = (y -14) / 8;
        if coords_x < 28 && coords_x >= 0 &&
           coords_y < 28 && coords_y >= 0 {
            self.over = (coords_x as usize, coords_y as usize);
            if self.selected.is_some() {
                if self.anim_left == 0 { self.queue_moves() }
            } else {
                if self.tiles[coords_y as usize][coords_x as usize] != 0 {
                    self.hovering = Some(self.tiles[coords_y as usize][coords_x as usize])
                } else {
                    self.hovering = None;
                }
            }
        } else {
            self.hovering = None;
        }
    }
    fn mouse_down(&mut self, x: i32, y: i32) {
        self.mouse_moved(x, y);
        if let Some(x) = self.hovering {
            if !self.blocked.contains(&x) && !self.collapseable.contains(&x) {
                self.origin = self.over;
                self.selected = self.hovering;
                self.hovering = None;
            }
        }
    }
    fn mouse_up(&mut self, x: i32, y:i32) {
        self.selected = None;
        if self.anim_left == 0 && self.moved { self.end_move() }
        self.mouse_moved(x, y);
    }
    fn check_won(&mut self) {
        for y in 0..28 {
            for x in 0..28 {
                if self.backdrop[y][x] == 2 && self.tiles[y][x] != self.special {
                    self.game_won = false;
                    return;
                }
            }
        }
        self.game_won = true;
    }
    fn tick(&mut self) {
        if self.anim_left > 0 {
            self.anim_left -= 1;
            self.anim_offset.0 += self.anim_direction.0;
            self.anim_offset.1 += self.anim_direction.1;
            if self.anim_left == 0 {
                self.origin.0 = (self.origin.0 as i32 + self.anim_direction.0).max(0) as usize;
                self.origin.1 = (self.origin.1 as i32 + self.anim_direction.1).max(0) as usize;
                self.transfer_from_anim_buffer(self.anim_direction);
                if self.selected.is_some() { self.queue_moves() } else { self.end_move() };
            }
        }
    }
    fn end_move(&mut self) {
        self.check_won(); self.moved = false; self.move_count += 1;
        self.add_collapses();
        let collapseable = self.collapseable.clone();
        for i in collapseable {
            if self.all_collapsed(i) {
                self.destroy_collapsed(i);
            }
        }
    }
    fn draw_backdrop<'t>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'t>>) {
        for x in 0..28 {
            for y in 0..28 {
                if self.backdrop[y][x] != 0 {
                    let pos = (x as i32 *8-2,y as i32 *8 + 14);
                    let col = self.backdrop[y][x] as usize;
                    graphics.tile_bg[col].draw(canvas,pos);
                    if x == 0 || self.backdrop[y][x-1] != self.backdrop[y][x] {
                        graphics.tile_right[col].draw(canvas,(pos.0-6,pos.1));
                    }
                    if y == 0 || self.backdrop[y-1][x] != self.backdrop[y][x] {
                        graphics.tile_bottom[col].draw(canvas,(pos.0,pos.1-6));
                    }
                    if x == 27 || self.backdrop[y][x+1] != self.backdrop[y][x] {
                        graphics.tile_left[col].draw(canvas,(pos.0+6,pos.1));
                    }
                    if y == 27 || self.backdrop[y+1][x] != self.backdrop[y][x] {
                        graphics.tile_top[col].draw(canvas,(pos.0,pos.1+6));
                    }

                } 
            }
        }
    }
    fn draw_tiles<'t>(&self, tiles: &[[u16;28];28], offset: (i32,i32), canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'t>>) {
        for x in 0..28 {
            for y in 0..28 {
                if tiles[y][x] != 0 {
                    let pos = (x as i32 *8-2+offset.0,y as i32 *8 + 14 + offset.1);
                    let col = if self.blocked.contains(&tiles[y][x]) { 0 }
                              else if self.collapseable.contains(&tiles[y][x]) { 3}
                              else if self.special == tiles[y][x] { 2 }
                              else { 1 };
                    if (Some(tiles[y][x]) == self.hovering && !self.collapseable.contains(&tiles[y][x])) || self.collapsed.contains(&(x,y)) {
                        graphics.tile_hover[col].draw(canvas,pos);
                    } else if Some(tiles[y][x]) == self.selected {
                        graphics.tile_unmoved[col].draw(canvas,pos);
                    } else {
                        graphics.tile_base[col].draw(canvas,pos);
                    }
                    if x == 0 || tiles[y][x-1] != tiles[y][x] {
                        graphics.tile_left[col].draw(canvas,pos);
                    }
                    if y == 0 || tiles[y-1][x] != tiles[y][x] {
                        graphics.tile_top[col].draw(canvas,pos);
                    }
                    if x == 27 || tiles[y][x+1] != tiles[y][x] {
                        graphics.tile_right[col].draw(canvas,pos);
                    }
                    if y == 27 || tiles[y+1][x] != tiles[y][x] {
                        graphics.tile_bottom[col].draw(canvas,pos);
                    }

                } 
            }
        }
    }
    fn draw<'t>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'t>>) {
        graphics.table.draw(canvas,(-2,14));
        self.draw_backdrop(canvas,graphics);
        self.draw_tiles(&self.tiles,(0,0),canvas,graphics);
        self.draw_tiles(&self.anim_tiles,self.anim_offset,canvas,graphics);
    }

}
const WIDTH:u32=220;
const HEIGHT:u32=252;
fn main_loop(mut window:Window, sdl_context: &Sdl) {
    window.set_size(WIDTH,HEIGHT).unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    
    canvas.set_logical_size(WIDTH,HEIGHT).unwrap();
    canvas.set_blend_mode(BlendMode::Blend);
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let graphics_set = GraphicsSet::new(&texture_creator);
    let mut table = Table::new(1);

    let mut move_count_gfx = Graphic::blank(5+6,1).textured(&texture_creator);
    move_count_gfx.draw_text("Move: ",&graphics_set.tile_set,0,0,BLACK,TRANSPARENT);
    let mut level_gfx = Graphic::blank(2+6,1).textured(&texture_creator);
    level_gfx.draw_text("Level ",&graphics_set.tile_set,0,0,BLACK,TRANSPARENT);
    let mut menu = MenuBar::new(WIDTH)
                    .add(Menu::new("GAME",152,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Next Level", 352, Keycode::F1,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Previous Level", 353, Keycode::F2,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(136, &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Restart",15, Keycode::N,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(136, &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Quit",363, Keycode::F12,&texture_creator,&graphics_set.tile_set)))
                    .add(Menu::new("VIEW",96+16,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Micro-mode",360, Keycode::F9,&texture_creator, &graphics_set.tile_set)));
    let mut rate_limiter = FPSManager::new();
    rate_limiter.set_framerate(60).unwrap();
    let mut micro_mode = false;
    loop {
        canvas.set_draw_color(CHARCOAL);
        canvas.clear();
        table.draw(&mut canvas, &graphics_set);
        move_count_gfx.draw_rect(6, 0, 5, 1, Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:0});
        move_count_gfx.draw_text(&table.move_count.to_string(), &graphics_set.tile_set , 6, 0, BLACK, TRANSPARENT);
        move_count_gfx.update_texture(&graphics_set.tile_set);
        level_gfx.draw_rect(6, 0, 2, 1, Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:0});
        level_gfx.draw_text(&table.level.to_string(), &graphics_set.tile_set , 6, 0, BLACK, TRANSPARENT);
        level_gfx.update_texture(&graphics_set.tile_set);
        let won = table.game_won;
        if won {
            let x = WIDTH as i32 / 2 - (21 * 4);
            let y = HEIGHT as i32 / 2 - (21 * 4);
            graphics_set.win.draw(&mut canvas, (x,y));
        }
        menu.draw(&mut canvas);
        canvas.set_draw_color(BLACK);
        canvas.fill_rect(Rect::new(0,HEIGHT as i32 -17,WIDTH,17)).unwrap();
        canvas.set_draw_color(rgba(176,179,172,255));
        canvas.fill_rect(Rect::new(0,HEIGHT as i32 -16,WIDTH,16)).unwrap();
        move_count_gfx.draw(&mut canvas, (8,HEIGHT as i32-12));
        level_gfx.draw(&mut canvas, (WIDTH as i32 - 16 - (6 * 8) - if table.level >= 10 { 8 } else { 0 } ,HEIGHT as i32-12));
        canvas.present();
        if table.anim_left > 0 {
            table.tick();
            table.tick();
            table.tick();
            rate_limiter.delay();
        } else {
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
                            table = Table::new(table.next_level());
                        },
                        Event::KeyDown { keycode: Some(Keycode::F2), ..} => {
                            table = Table::new(table.previous_level());
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
                            table = Table::new(table.next_level());
                        },
                        Event::MouseButtonUp { ..} if won => {
                            table = Table::new(table.next_level());
                        },
                        Event::KeyDown { keycode: Some(Keycode::N), ..} => {
                            table = Table::new(table.level);
                        },
                        Event::MouseButtonDown { mouse_btn: _, x, y, ..} if !won => {
                            table.mouse_down(x, y);
                        }
                        Event::MouseMotion { x, y, ..} if !won => {
                            table.mouse_moved(x,y);
                        }
                        Event::MouseButtonUp { mouse_btn: _, x, y, ..} if !won => {                            
                            table.mouse_up(x, y);
                        }
                        _ => {},
                    }
                }
                rate_limiter.delay();
            }
        }
    }
}


fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("mageja", WIDTH, 240)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    main_loop(window, &sdl_context);
}