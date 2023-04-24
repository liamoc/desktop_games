
extern crate tesserae;
extern crate sdl2;
extern crate rand;
extern crate utils;

use utils::menu::{*};

mod rules;
use std::env;
use rules::{TetraVex,Two,Three,Four,Five};
use tesserae::{Graphic,Tile,TileSet};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::render::{Texture, BlendMode, TextureCreator};
use sdl2::video::Window;
use sdl2::Sdl; 
use sdl2::keyboard::Keycode;
use rand::Rng;
use sdl2::render::Canvas;
use utils::framerate::FPSManager;
use std::io::Cursor;
use rand::{thread_rng};
use rand::seq::SliceRandom;
use utils::color::{*};

struct ColorSet {
    bright_border_color: [Color;9],
    dark_border_color : [Color;9],
    face_color : [Color;9],
}
struct GraphicsSet<T> {
    tile_set: TileSet,
    top_wedge: [Graphic<T>;9],
    left_wedge: [Graphic<T>;9],
    right_wedge: [Graphic<T>;9],
    bottom_wedge: [Graphic<T>;9],
    cell: Graphic<T>,
    win: Graphic<T>,
}
impl <'r> GraphicsSet<Texture<'r>> {
    fn left_wedge<T>(texture_creator: &'r TextureCreator<T>, tile_set: &TileSet, color_set: &ColorSet, index: usize) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(7,7).textured(texture_creator);
        ret[(0,0)] = Tile { index: 168, fg: color_set.bright_border_color[index], bg: TRANSPARENT };
        ret[(0,1)] = Tile { index: 184, fg: color_set.bright_border_color[index], bg: TRANSPARENT };
        ret[(0,2)] = Tile { index: 184, fg: color_set.bright_border_color[index], bg: TRANSPARENT };
        ret[(0,3)] = Tile { index: 184, fg: color_set.bright_border_color[index], bg: TRANSPARENT };
        ret[(0,4)] = Tile { index: 184, fg: color_set.bright_border_color[index], bg: TRANSPARENT };
        ret[(0,5)] = Tile { index: 184, fg: color_set.bright_border_color[index], bg: TRANSPARENT };
        ret[(0,6)] = Tile { index: 200, fg: color_set.bright_border_color[index], bg: TRANSPARENT };
        ret[(1,1)] = Tile { index: 207, bg: TRANSPARENT, fg: color_set.face_color[index] };
        ret[(1,2)] = Tile { index: 0, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(1,3)] = Tile { index: 0, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(1,4)] = Tile { index: 0, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(1,5)] = Tile { index: 206, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(2,2)] = Tile { index: 207, bg: TRANSPARENT, fg: color_set.face_color[index] };
        ret[(2,3)] = Tile { index: 184, fg: color_set.dark_border_color[index], bg: color_set.face_color[index] };
        ret[(2,4)] = Tile { index: 206, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret.update_texture(tile_set);
        ret
    }
    fn right_wedge<T>(texture_creator: &'r TextureCreator<T>, tile_set: &TileSet, color_set: &ColorSet, index: usize) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(7,7).textured(texture_creator);
        ret[(6,0)] = Tile { index: 170, fg: color_set.dark_border_color[index], bg: TRANSPARENT };
        ret[(6,1)] = Tile { index: 186, fg: color_set.dark_border_color[index], bg: TRANSPARENT };
        ret[(6,2)] = Tile { index: 186, fg: color_set.dark_border_color[index], bg: TRANSPARENT };
        ret[(6,3)] = Tile { index: 186, fg: color_set.dark_border_color[index], bg: TRANSPARENT };
        ret[(6,4)] = Tile { index: 186, fg: color_set.dark_border_color[index], bg: TRANSPARENT };
        ret[(6,5)] = Tile { index: 186, fg: color_set.dark_border_color[index], bg: TRANSPARENT };
        ret[(6,6)] = Tile { index: 202, fg: color_set.dark_border_color[index], bg: TRANSPARENT };
        ret[(5,1)] = Tile { index: 206, bg: TRANSPARENT, fg: color_set.face_color[index] };
        ret[(5,2)] = Tile { index: 0, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(5,3)] = Tile { index: 0, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(5,4)] = Tile { index: 0, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(5,5)] = Tile { index: 207, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(4,2)] = Tile { index: 206, bg: TRANSPARENT, fg: color_set.face_color[index] };
        ret[(4,3)] = Tile { index: 186, fg: color_set.bright_border_color[index], bg: color_set.face_color[index] };
        ret[(4,4)] = Tile { index: 207, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret.update_texture(tile_set);
        ret
    }
    fn bottom_wedge<T>(texture_creator: &'r TextureCreator<T>, tile_set: &TileSet, color_set: &ColorSet, index: usize) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(7,7).textured(texture_creator);
        ret[(0,6)] = Tile { index: 200, fg: color_set.dark_border_color[index], bg: TRANSPARENT };
        ret[(1,6)] = Tile { index: 201, fg: color_set.dark_border_color[index], bg: TRANSPARENT };
        ret[(2,6)] = Tile { index: 201, fg: color_set.dark_border_color[index], bg: TRANSPARENT };
        ret[(3,6)] = Tile { index: 201, fg: color_set.dark_border_color[index], bg: TRANSPARENT };
        ret[(4,6)] = Tile { index: 201, fg: color_set.dark_border_color[index], bg: TRANSPARENT };
        ret[(5,6)] = Tile { index: 201, fg: color_set.dark_border_color[index], bg: TRANSPARENT };
        ret[(6,6)] = Tile { index: 202, fg: color_set.dark_border_color[index], bg: TRANSPARENT };
        ret[(1,5)] = Tile { index: 206, bg: TRANSPARENT, fg: color_set.face_color[index] };
        ret[(2,5)] = Tile { index: 0, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(3,5)] = Tile { index: 0, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(4,5)] = Tile { index: 0, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(5,5)] = Tile { index: 207, bg: TRANSPARENT, fg: color_set.face_color[index] };
        ret[(2,4)] = Tile { index: 206, bg: TRANSPARENT, fg: color_set.face_color[index] };
        ret[(3,4)] = Tile { index: 201, fg: color_set.bright_border_color[index], bg: color_set.face_color[index] };
        ret[(4,4)] = Tile { index: 207, bg: TRANSPARENT, fg: color_set.face_color[index] };
        ret.update_texture(tile_set);
        ret
    }
    fn top_wedge<T>(texture_creator: &'r TextureCreator<T>, tile_set: &TileSet, color_set: &ColorSet, index: usize) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(7,7).textured(texture_creator);
        ret[(0,0)] = Tile { index: 168, fg: color_set.bright_border_color[index], bg: TRANSPARENT };
        ret[(1,0)] = Tile { index: 169, fg: color_set.bright_border_color[index], bg: TRANSPARENT };
        ret[(2,0)] = Tile { index: 169, fg: color_set.bright_border_color[index], bg: TRANSPARENT };
        ret[(3,0)] = Tile { index: 169, fg: color_set.bright_border_color[index], bg: TRANSPARENT };
        ret[(4,0)] = Tile { index: 169, fg: color_set.bright_border_color[index], bg: TRANSPARENT };
        ret[(5,0)] = Tile { index: 169, fg: color_set.bright_border_color[index], bg: TRANSPARENT };
        ret[(6,0)] = Tile { index: 170, fg: color_set.bright_border_color[index], bg: TRANSPARENT };
        ret[(1,1)] = Tile { index: 207, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(2,1)] = Tile { index: 0, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(3,1)] = Tile { index: 0, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(4,1)] = Tile { index: 0, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(5,1)] = Tile { index: 206, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(2,2)] = Tile { index: 207, fg: TRANSPARENT, bg: color_set.face_color[index] };
        ret[(3,2)] = Tile { index: 169, fg: color_set.dark_border_color[index], bg: color_set.face_color[index] };
        ret[(4,2)] = Tile { index: 206, fg: TRANSPARENT, bg: color_set.face_color[index] };
        //ret[(3,3)] = Tile { index: 254, fg: CHARCOAL, bg: TRANSPARENT };
        ret.update_texture(tile_set);
        ret
    }
    fn new<T>(texture_creator: &'r TextureCreator<T>) -> GraphicsSet<Texture<'r>> {
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
        let color_set = ColorSet{
            face_color: [rgba(204,0,0,255),rgba(115,210,22,255),rgba(52,101,164,255),
                         rgba(245,121,0,255), rgba(117,80,123,255),rgba(78,165,177,255),
                         rgba(237,212,0,255), rgba(193,125,17,255), rgba(186,189,182,255)],
            dark_border_color: 
                        [rgba(164,0,0,255),rgba(78,154,6,255),rgba(5,32,74,255),
                         rgba(206,92,0,255),rgba(92,52,102,255),rgba(33,140,141,255),
                         rgba(196,160,0,255),rgba(143,89,2,255),rgba(115,120,118,255)],
            bright_border_color:
                        [rgba(239,41,41,255),rgba(138,226,52,255),rgba(114,159,207,255),
                         rgba(252,175,62,255),rgba(173,127,168,255),rgba(108,206,203,255),
                         rgba(252,233,79,255),rgba(233,185,110,255),rgba(211,215,207,255)] 

        };
        let top_wedge = [
            Self::top_wedge(texture_creator, &tile_set, &color_set, 0),
            Self::top_wedge(texture_creator, &tile_set, &color_set, 1),
            Self::top_wedge(texture_creator, &tile_set, &color_set, 2),
            Self::top_wedge(texture_creator, &tile_set, &color_set, 3),
            Self::top_wedge(texture_creator, &tile_set, &color_set, 4),
            Self::top_wedge(texture_creator, &tile_set, &color_set, 5),
            Self::top_wedge(texture_creator, &tile_set, &color_set, 6),
            Self::top_wedge(texture_creator, &tile_set, &color_set, 7),
            Self::top_wedge(texture_creator, &tile_set, &color_set, 8),
        ];
        let bottom_wedge = [
            Self::bottom_wedge(texture_creator, &tile_set, &color_set, 0),
            Self::bottom_wedge(texture_creator, &tile_set, &color_set, 1),
            Self::bottom_wedge(texture_creator, &tile_set, &color_set, 2),
            Self::bottom_wedge(texture_creator, &tile_set, &color_set, 3),
            Self::bottom_wedge(texture_creator, &tile_set, &color_set, 4),
            Self::bottom_wedge(texture_creator, &tile_set, &color_set, 5),
            Self::bottom_wedge(texture_creator, &tile_set, &color_set, 6),
            Self::bottom_wedge(texture_creator, &tile_set, &color_set, 7),
            Self::bottom_wedge(texture_creator, &tile_set, &color_set, 8),
        ];
        let left_wedge = [
            Self::left_wedge(texture_creator, &tile_set, &color_set, 0),
            Self::left_wedge(texture_creator, &tile_set, &color_set, 1),
            Self::left_wedge(texture_creator, &tile_set, &color_set, 2),
            Self::left_wedge(texture_creator, &tile_set, &color_set, 3),
            Self::left_wedge(texture_creator, &tile_set, &color_set, 4),
            Self::left_wedge(texture_creator, &tile_set, &color_set, 5),
            Self::left_wedge(texture_creator, &tile_set, &color_set, 6),
            Self::left_wedge(texture_creator, &tile_set, &color_set, 7),
            Self::left_wedge(texture_creator, &tile_set, &color_set, 8),
        ];
        let right_wedge = [
            Self::right_wedge(texture_creator, &tile_set, &color_set, 0),
            Self::right_wedge(texture_creator, &tile_set, &color_set, 1),
            Self::right_wedge(texture_creator, &tile_set, &color_set, 2),
            Self::right_wedge(texture_creator, &tile_set, &color_set, 3),
            Self::right_wedge(texture_creator, &tile_set, &color_set, 4),
            Self::right_wedge(texture_creator, &tile_set, &color_set, 5),
            Self::right_wedge(texture_creator, &tile_set, &color_set, 6),
            Self::right_wedge(texture_creator, &tile_set, &color_set, 7),
            Self::right_wedge(texture_creator, &tile_set, &color_set, 8),
        ];
        let mut cell = Graphic::load_from(Cursor::new(&include_bytes!("../tetravex")[..])).unwrap().textured(texture_creator);
        cell.update_texture(&tile_set);
        let mut win = Graphic::load_from(Cursor::new(&include_bytes!("../win")[..])).unwrap().textured(&texture_creator);
        win.update_texture(&tile_set);
        GraphicsSet {
            tile_set: tile_set, top_wedge,bottom_wedge,left_wedge,right_wedge,
            cell ,win
        }
    }

}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Card {
    pub values : [u8;4],    
}
impl Card {
    pub fn deck(size: usize) -> Vec<Card> {
        let mut ret = Vec::new();
        let mut map = vec![vec![[0;4];size];size];
        for x in 0..size {
            for y in 0..size {
                let left = if x > 0 { map[x-1][y][3] } else { thread_rng().gen_range(1,10)};
                let top = if y > 0 { map[x][y-1][1] } else { thread_rng().gen_range(1,10)};
                let right = thread_rng().gen_range(1,10);
                let bottom = thread_rng().gen_range(1,10);
                map[x][y][0] = top;
                map[x][y][1] = bottom;
                map[x][y][2] = left;
                map[x][y][3] = right;
            }
        }
        for i in 0..size {
            for j in 0..size {
                ret.push(Card {values: map[i][j]});
            }
        }
        ret.shuffle(&mut thread_rng());
        ret
    }
    fn draw_well<'r>(canvas:&mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>, position: (i32,i32)) {
        graphics.cell.draw(canvas,position)
    }
    fn draw<'r>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>, position : (i32,i32)) {
        graphics.top_wedge[self.values[0] as usize-1].draw(canvas,position);
        graphics.bottom_wedge[self.values[1] as usize-1].draw(canvas,position);
        graphics.left_wedge[self.values[2] as usize-1].draw(canvas,position);
        graphics.right_wedge[self.values[3] as usize-1].draw(canvas,position);
    }

}
#[derive(Debug)]
pub struct Well {
    pub id: usize,
    pub position: (i32, i32),
    pub card: Option<Card>,
}
impl Well {
    fn draw<'r>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>) {        
        Card::draw_well(canvas,graphics,self.position);
        if let Some(c) = &self.card {
            c.draw(canvas,graphics,self.position);
        }
    }
    fn top_position(&self) -> (i32,i32) {
        self.position
    }
    fn skim(&mut self) -> Option<Card> {
        if let Some(_) = self.card {
            let ret = self.card;
            self.card = None;
            ret
        } else { None }
    }
}
struct Animation {
    time: u32,
    card: Card,
    start: (i32,i32),
    end: (i32,i32),
    and_then: Box<dyn FnOnce(&mut Table)>
}
impl Animation {
    fn draw<'r>(&self, canvas: &mut Canvas<Window>, graphics: & GraphicsSet<Texture<'r>>, frame: u32) {
        let (sx,sy) = self.start;
        let (ex,ey) = self.end;
        let x = sx + ((ex - sx) * frame as i32 / self.time as i32); 
        let y = sy + ((ey - sy) * frame as i32 / self.time as i32); 
        self.card.draw(canvas, graphics, (x,y));
    }
    
}
pub struct Table {    
    wells: Vec<Well>,
    history: Vec<Vec<Move>>,
    animations: Vec<(Animation,u32)>, 
    move_count: u32,   
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Move {
    Shift(GameObject, GameObject),
}


impl Table {
    pub fn add_well(&mut self, position: (i32, i32), card: Option<Card>) -> usize {
        let id = self.wells.len();
        self.wells.push(Well { id: id, position: position, card: card});
        id
    }
    pub fn wells(&self) -> &[Well] {
        &self.wells
    }
    pub fn well(&self, index: usize) -> &Well {
        &self.wells[index]
    }
    fn shift_then_raw(&mut self, src: GameObject, dest: GameObject, and_then: Box<dyn FnOnce(&mut Table)>) {
        let cards = match src {
            //TODO this should use functions on the objects
            GameObject::Well(i) => {
                let c = self.well(i).card.unwrap();
                self.wells[i].card = None;
                c
            }
        };
        let start_pos = match src {
            GameObject::Well(i) => self.well(i).top_position()
        };
        self.animate_add_card_to(dest,cards, start_pos, and_then)

    }
    pub fn shift_then(&mut self, src: GameObject, dest: GameObject, and_then: Box<dyn FnOnce(&mut Table)>) {
        if src != dest {
            self.shift_then_raw(src, dest, and_then);
            if let Some(h) = self.history.last_mut() {
                h.push(Move::Shift(src,dest))
            }
        }
    }
    pub fn shift(&mut self, src: GameObject, dest: GameObject) {
            self.shift_then(src,dest,Box::new(|_| {}));
    }
    pub fn end_move(&mut self) {
        if if let Some(h) = self.history.last() {
                h.len() > 0
            } else { true }  {
            self.history.push(Vec::new());
            self.move_count += 1
        }
    }
    pub fn can_undo(&self) -> bool {
        if self.animations.len() > 0 { return false; }
        if self.history.len() > 1 {
            if let Some(h) = self.history.last() {
                h.len() == 0
            } else { false }
        } else { false }
    }
    pub fn undo(&mut self) {
        if self.can_undo() {
            self.history.truncate(self.history.len() - 1);
            let mut moves = if let Some(h) = self.history.last_mut() {                    
                h.drain(..).rev().collect()                     
            } else { Vec::new() };
            for i in moves.drain(..) {
                match i {
                    Move::Shift(s,d) => { 
                        self.shift_then_raw(d, s, Box::new(move |_tbl|{
                        }));
                    },
                }
            }
        }
    }
    fn animate_move_stack(&mut self, start: (i32,i32), end: (i32,i32), card: Card, time: u32, and_then: Box<dyn FnOnce(&mut Table)>) {
        self.animations.push((Animation { time: time, card: card, start: start, end:end, and_then: and_then},0))
    }
    pub fn add_card_to(&mut self, dest: GameObject, card: Card) {
        //TODO use functions on the objects
        match dest {
            GameObject::Well(i) => { if self.wells[i].card.is_none() { self.wells[i].card = Some(card) } },
        }
    }
    pub fn animate_add_card_to(&mut self, dest:GameObject, card :Card, start_pos: (i32,i32), and_then: Box<dyn FnOnce(&mut Table)>) {
        let dest_pos = match dest {
            GameObject::Well(i) => self.well(i).top_position()
        };
        let f : Box<dyn FnOnce(&mut Table)> = Box::new(move |tbl| {
            tbl.add_card_to(dest,card);
            and_then(tbl)
        });
        self.animate_move_stack(start_pos, dest_pos, card, 14, f);
    } 
    fn new<T:Rules>() -> Table {
        let mut table = Table {
            animations: Vec::new(),
            wells: Vec::new(),
            history: Vec::new(),
            move_count: 0
        };        
        T::new_game(&mut table);
        table.history.push(Vec::new());
        table
    }

    fn draw<'t>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'t>>) {
        //graphics.table.draw(canvas,(0,0));
        for i in &self.wells {
            i.draw(canvas,graphics)
        }
        for (x,i) in &self.animations {
            x.draw(canvas,graphics, *i)
        }
    }

    fn tick(&mut self) -> Vec<(Animation,u32)> {
        for (x,i) in &mut self.animations {
            *i+=1;            
            if *i == x.time {
            
            }
        }
        let (done, v): (Vec<_>, Vec<_>) = self.animations.drain(..).partition(|(x,i)| *i >= x.time );
        self.animations = v;
        done
    }

    fn collides_well(&self, position: (i32,i32)) -> Option<(usize, (i32,i32))> {
        for i in &self.wells {
            if position.0 >= i.top_position().0 + 6 && position.0 < i.top_position().0 + 50 {
                let offset_x = i.top_position().0 - position.0;
                if position.1 >= i.top_position().1 + 6 && position.1 < i.top_position().1 + 50 {
                    let offset_y = i.top_position().1 - position.1;
                    return Some((i.id,(offset_x,offset_y)));
                }
            }
        };
        None
    }
}

pub trait Rules {
    fn table_size() -> (u32,u32);
    fn new_game(table: &mut Table);

    fn can_place_well(table: &Table, well_id: usize, card: Card) -> bool;
    fn can_skim_well(well : &Well) -> bool;
    fn game_won(table: &Table) -> bool;
    fn well_clicked(tbl: &mut Table, well_id: usize);
    fn placed_in_well(table: &mut Table, well_id : usize);
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameObject {
    Well(usize)
}
fn main_loop<RULES:Rules>(mut window:Window, sdl_context: &Sdl) -> (Option<u32>,Window) {

    
    window.set_size(RULES::table_size().0,RULES::table_size().1).unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    
    canvas.set_logical_size(RULES::table_size().0,RULES::table_size().1).unwrap();
    canvas.set_blend_mode(BlendMode::Blend);
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let graphics_set = GraphicsSet::new(&texture_creator);
    let mut table = Table::new::<RULES>();

    let mut mx = 0;
    let mut my = 0;
    let mut md = false;
    let mut move_count_gfx = Graphic::blank(4,1).textured(&texture_creator);
    let mut move_count_gfx_shadow = Graphic::blank(4,1).textured(&texture_creator);
    let mut attached_cards : Option<(Card, GameObject)> = None;

    let mut grab_offset : (i32,i32) = (0,0);
    let wwh = RULES::table_size().1;
    let mut menu = MenuBar::new(RULES::table_size().0)
                    .add(Menu::new("GAME",152,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("2x2 (Easy)", 352, Keycode::F1,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("3x3 (Normal)", 353, Keycode::F2,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("4x4 (Hard)", 354, Keycode::F3,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("5x5 (Extreme)", 355, Keycode::F4,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(136, &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Quit",363, Keycode::F12,&texture_creator,&graphics_set.tile_set)))
                    .add(Menu::new("ACTION",88,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Undo",27, Keycode::Z,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(72, &texture_creator, &graphics_set.tile_set))
                            .add(MenuItem::new("Restart",15, Keycode::N,&texture_creator,&graphics_set.tile_set)))
                    .add(Menu::new("VIEW",112,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Micro-mode",360, Keycode::F9,&texture_creator, &graphics_set.tile_set)));
    let mut rate_limiter = FPSManager::new();
    rate_limiter.set_framerate(60).unwrap();
    let mut micro_mode = false;
    loop {
        canvas.set_draw_color(CHARCOAL);
        canvas.clear();
        table.draw(&mut canvas, &graphics_set);
        if let Some((cards,_)) = &attached_cards {
            cards.draw(&mut canvas, &graphics_set, (mx + grab_offset.0,my + grab_offset.1));
        }
        move_count_gfx.draw_rect(0, 0, 4, 1, Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:0});
        move_count_gfx.draw_text(&table.move_count.to_string(), &graphics_set.tile_set , 0, 0, WHITE, TRANSPARENT);
        move_count_gfx_shadow.draw_rect(0, 0, 4, 1, Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:0});
        move_count_gfx_shadow.draw_text(&table.move_count.to_string(), &graphics_set.tile_set , 0, 0, BLACK, TRANSPARENT);
        move_count_gfx.update_texture(&graphics_set.tile_set);
        move_count_gfx_shadow.update_texture(&graphics_set.tile_set);
        move_count_gfx_shadow.draw(&mut canvas, (10,wwh as i32-9-8));
        move_count_gfx_shadow.draw(&mut canvas, (10,wwh as i32-11-8));
        move_count_gfx_shadow.draw(&mut canvas, (9,wwh as i32-10-8));
        move_count_gfx_shadow.draw(&mut canvas, (11,wwh as i32-10-8));
        move_count_gfx.draw(&mut canvas, (10,wwh as i32-10-8));
        let won = RULES::game_won(&table);
        if won {
            let x = (RULES::table_size().0 as i32) / 2 - (21 * 4);
            let y = (RULES::table_size().1 as i32) / 2 - (21 * 4);
            graphics_set.win.draw(&mut canvas, (x,y));
        }
        menu.draw(&mut canvas);
        canvas.present();
        if table.animations.len() > 0 {
            rate_limiter.delay();
            let animations = table.tick();
            for i in animations {
                (i.0.and_then)(&mut table);
            }
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
                            return (None, canvas.into_window())
                        },
                        Event::KeyDown { keycode: Some(Keycode::F1), ..} => {
                            return (Some(2), canvas.into_window());
                        },
                        Event::KeyDown { keycode: Some(Keycode::F2), ..} => {
                            return (Some(3), canvas.into_window());
                        },
                        Event::KeyDown { keycode: Some(Keycode::F3), ..} => {
                            return (Some(4), canvas.into_window());
                        },
                        Event::KeyDown { keycode: Some(Keycode::F4), ..} => {
                            return (Some(5), canvas.into_window());
                        },
                        Event::KeyDown { keycode: Some(Keycode::F9), ..} => {
                            if micro_mode {
                                micro_mode = false;
                                canvas.window_mut().set_size(RULES::table_size().0,RULES::table_size().1).unwrap_or_default();
                            } else {
                                canvas.window_mut().set_size(RULES::table_size().0/2,RULES::table_size().1/2).unwrap_or_default();
                                micro_mode = true;
                            }
                        }
                        Event::KeyDown {..} if won => {
                            table = Table::new::<RULES>();
                            attached_cards = None;
                            md = false;
                        },
                        Event::MouseButtonUp { ..} if won => {
                            table = Table::new::<RULES>();
                            attached_cards = None;
                            md = false;
                        },
                        Event::KeyDown { keycode: Some(Keycode::Z), ..} => {
                            table.undo()
                        },
                        Event::KeyDown { keycode: Some(Keycode::N), ..} => {
                            table = Table::new::<RULES>();
                            attached_cards = None;
                            md = false;
                        },
                        Event::MouseButtonDown { mouse_btn: _, x, y, ..} if !won => {
                            let (sx,sy) = (x,y);//(x * dw as i32 / ww as i32, y * dh as i32 / wh as i32);
                            md = true;
                            mx = sx; my = sy;
                            if let Some((idx,offset)) = table.collides_well((mx,my)) {
                                if RULES::can_skim_well(&table.wells[idx]) {
                                    attached_cards = Some((table.wells[idx].skim().unwrap(), GameObject::Well(idx)));
                                    grab_offset = offset;
                                }
                            }
                        }
                        Event::MouseMotion { x, y, ..} if !won => {
                            let (sx,sy) = (x,y); //(x * dw as i32 / ww as i32, y * dh as i32 / wh as i32);
                            if (mx,my) != (sx,sy) {
                                mx = sx; my = sy;
                                md = false;
                            }
                        }

                        Event::MouseButtonUp { mouse_btn: _, x, y, ..} if !won => {
                            
                            let (sx,sy) = (x,y);//(x * dw as i32 / ww as i32, y * dh as i32 / wh as i32);
                            mx = sx; my = sy;
                            let mut placed = false;
                            if let Some((cards,origin)) = attached_cards {
                                if !md {
                                    if let Some((idx,_)) = table.collides_well((mx,my)) {
                                        if origin != GameObject::Well(idx) {
                                            if RULES::can_place_well(&table,idx, cards) {
                                                table.animate_add_card_to(GameObject::Well(idx), cards,(mx + grab_offset.0,my + grab_offset.1),Box::new(move |tbl : & mut Table| {
                                                    RULES::placed_in_well(tbl, idx);
                                                    tbl.end_move();
                                                }));
                                                placed = true;
                                                if let Some(h) = table.history.last_mut() {
                                                    h.push(Move::Shift(origin, GameObject::Well(idx)));
                                                }
                                            }
                                        }
                                    }
                                } 
                                if !placed {
                                    if md {
                                        md = false;
                                        table.add_card_to(origin, cards);
                                        match origin {
                                            GameObject::Well(d) => RULES::well_clicked(&mut table,d),
                                        }
                                    } else {
                                        table.animate_add_card_to(origin, cards, (mx + grab_offset.0,my + grab_offset.1),Box::new(|_| {}));
                                    }
                                }
                                attached_cards = None;
                            }
                        }
                        _ => {},
                    }
                }
                rate_limiter.delay();
            }
        }
    }
}

fn choose(window : Window, sdl_context:&Sdl, size : u32) {
    if let (Some(v), w) = match size {
        2 => main_loop::<TetraVex<Two>>(window,sdl_context),
        3 => main_loop::<TetraVex<Three>>(window,sdl_context),
        4 => main_loop::<TetraVex<Four>>(window,sdl_context),
        _ => main_loop::<TetraVex<Five>>(window,sdl_context),        
    } {
        choose(w, sdl_context,v);
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("vexation", 320, 240)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    let arg = env::args().nth(1).unwrap_or(String::from("3"));
    match &arg[..] {
        "2" => choose(window,&sdl_context,2),
        "3" => choose(window,&sdl_context,3),
        "4"   => choose(window,&sdl_context,4),
        "5"   => choose(window,&sdl_context,5),
        _ => println!("Available sizes: 2,3,4,5")
    }
    //cards::main_loop::<Spider<OneSuit>>();
    
}