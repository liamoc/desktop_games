
extern crate tesserae;
extern crate sdl2;
extern crate rand;
extern crate utils;

use utils::menu::{*};

mod rules;
use std::env;
use rules::{Klondike,ThreeDraw,OneDraw,Golf,Spider,OneSuit,TwoSuit,FourSuit,FreeCell,TriPeaks,Cruel, Pyramid};
use std::ops::{Index};
use tesserae::{Graphic,Tile,TileSet};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::render::{Texture, BlendMode, TextureCreator};
use sdl2::video::Window;
use sdl2::Sdl; 
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::rect::Rect;
use sdl2::gfx::framerate::FPSManager;
use std::io::Cursor;
use rand::{thread_rng};
use rand::seq::SliceRandom;
use utils::color::{*};

struct GraphicsSet<T> {
    table: Graphic<T>,
    back_card: Graphic<T>,
    highlight_card: Graphic<T>,
    cards: Vec<Graphic<T>>,
    well: Graphic<T>,
    emblem: Graphic<T>,
    emblem_shadow: Graphic<T>,

    black_border: Graphic<T>,
    win: Graphic<T>,
    tile_set: TileSet,
}
impl <'r> GraphicsSet<Texture<'r>> {
    const BG_DARK : Color = rgba(78,125,6,255);
    const BG_LIGHT : Color = rgba(115,155,22,255);
    const BACK_DARK : Color = rgba(32,74,135,255);
    const BACK_LIGHT : Color = rgba(52,101,164,255);
    const TRANSLUCENT : Color = rgba(255,255,255,128);
    const HIGHLIGHT   : Color = rgba(255,255,0,128);
    fn blank_card(contents:Tile, border: Color) -> Graphic<()> {
        let mut ret = Graphic::blank(5, 6);
        ret[(0,0)] = Tile{fg:border,bg:TRANSPARENT,index:168};
        ret[(1,0)] = Tile{fg:border,bg:TRANSPARENT,index:169};
        ret[(2,0)] = Tile{fg:border,bg:TRANSPARENT,index:169};
        ret[(3,0)] = Tile{fg:border,bg:TRANSPARENT,index:169};
        ret[(4,0)] = Tile{fg:border,bg:TRANSPARENT,index:170};
        ret[(0,1)] = Tile{fg:border,bg:TRANSPARENT,index:184};
        ret[(1,1)] = contents;
        ret[(2,1)] = contents;
        ret[(3,1)] = contents;
        ret[(4,1)] = Tile{fg:border,bg:TRANSPARENT,index:186}; 
        ret[(0,2)] = Tile{fg:border,bg:TRANSPARENT,index:184};
        ret[(1,2)] = contents;
        ret[(2,2)] = contents;
        ret[(3,2)] = contents;
        ret[(4,2)] = Tile{fg:border,bg:TRANSPARENT,index:186}; 
        ret[(0,3)] = Tile{fg:border,bg:TRANSPARENT,index:184};
        ret[(1,3)] = contents;
        ret[(2,3)] = contents;
        ret[(3,3)] = contents;
        ret[(4,3)] = Tile{fg:border,bg:TRANSPARENT,index:186}; 
        ret[(0,4)] = Tile{fg:border,bg:TRANSPARENT,index:184};
        ret[(1,4)] = contents;
        ret[(2,4)] = contents;
        ret[(3,4)] = contents;
        ret[(4,4)] = Tile{fg:border,bg:TRANSPARENT,index:186}; 
        ret[(0,5)] = Tile{fg:border,bg:TRANSPARENT,index:200};
        ret[(1,5)] = Tile{fg:border,bg:TRANSPARENT,index:201};
        ret[(2,5)] = Tile{fg:border,bg:TRANSPARENT,index:201};
        ret[(3,5)] = Tile{fg:border,bg:TRANSPARENT,index:201};
        ret[(4,5)] = Tile{fg:border,bg:TRANSPARENT,index:202};
        ret
    }
    fn card_for(card: Card) -> Graphic<()> {
        let mut g = Self::blank_card(Tile{fg: WHITE, bg: WHITE, index:0}, WHITE);
        let fg = match card.suit {
            Suit::Hearts | Suit::Diamonds => DARK_RED,
            _ => BLACK,
        };
        g[(1,1)] = Tile{fg: fg, bg: WHITE, index: match card.value {
            1 => 2,
            2..=9 => 51 + card.value as usize,
            10 => 113,
            11 => 11,
            12 => 18,
            13 => 12,
            _  => 30
        }};
        g[(2,1)] = Tile{fg: fg, bg: WHITE, index: match card.suit {
            Suit::Hearts => 147,
            Suit::Diamonds => 144,
            Suit::Clubs => 145,
            Suit::Spades => 146
        }};
        g[(2,4)] = g[(2,1)];
        g[(3,4)] = g[(1,1)];
        let offset = match card.suit {
            Suit::Diamonds => 0,
            Suit::Hearts => 3,
            Suit::Spades => 6,
            Suit::Clubs => 9
        };
        g[(1,2)] = Tile{fg:fg, bg: WHITE, index: 288 + offset};
        g[(2,2)] = Tile{fg:fg, bg: WHITE, index: 289 + offset};
        g[(3,2)] = Tile{fg:fg, bg: WHITE, index: 290 + offset};
        g[(1,3)] = Tile{fg:fg, bg: WHITE, index: 304 + offset};
        g[(2,3)] = Tile{fg:fg, bg: WHITE, index: 305 + offset};
        g[(3,3)] = Tile{fg:fg, bg: WHITE, index: 306 + offset};
        g
    }
    fn new<T>(texture_creator: &'r TextureCreator<T>) -> GraphicsSet<Texture<'r>> {

        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
        let mut table = Graphic::solid(640/8, 480/8, Tile {fg: Self::BG_LIGHT, bg:Self::BG_DARK, index:255}).textured(texture_creator);
        let mut back_card = Self::blank_card(Tile {fg: Self::BACK_LIGHT, bg:Self::BACK_DARK, index:192}, Self::BACK_LIGHT).textured(texture_creator);
        let mut highlight_card = Self::blank_card(Tile {fg: Self::HIGHLIGHT, bg:Self::BACK_DARK, index:1}, Self::HIGHLIGHT).textured(texture_creator);
        let mut cell = Self::blank_card(Tile{fg:Self::TRANSLUCENT, bg:Self::TRANSLUCENT, index:1}, WHITE).textured(texture_creator);
        let mut emblem_shadow = Graphic::solid(2,1,Tile {bg: TRANSPARENT, fg: BLACK, index:316}).textured(texture_creator);
        let mut emblem  = Graphic::solid(2,1,Tile {bg: TRANSPARENT, fg: WHITE, index:316}).textured(texture_creator);
        emblem[(1,0)].index += 1;
        emblem_shadow[(1,0)].index += 1;
        emblem.update_texture(&tile_set);
        emblem_shadow.update_texture(&tile_set);
        table.update_texture(&tile_set);
        back_card.update_texture(&tile_set);
        highlight_card.update_texture(&tile_set);
        cell.update_texture(&tile_set);
        let mut cards = Vec::new();
        for i in 1..=13 {
            let mut card = Self::card_for(Card { suit: Suit::Hearts , value: i}).textured(texture_creator);
            card.update_texture(&tile_set);
            cards.push(card)
        }
        for i in 1..=13 {
            let mut card = Self::card_for(Card { suit: Suit::Diamonds, value: i}).textured(texture_creator);
            card.update_texture(&tile_set);
            cards.push(card)
        }
        for i in 1..=13 {
            let mut card = Self::card_for(Card { suit: Suit::Spades, value: i}).textured(texture_creator);
            card.update_texture(&tile_set);
            cards.push(card)
        }
        for i in 1..=13 {
            let mut card = Self::card_for(Card { suit: Suit::Clubs, value: i}).textured(texture_creator);
            card.update_texture(&tile_set);
            cards.push(card)
        }
        let mut white_border = Self::blank_card(Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:1}, WHITE).textured(texture_creator);
        let mut black_border = Self::blank_card(Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:1}, BLACK).textured(texture_creator);
        white_border.update_texture(&tile_set);
        black_border.update_texture(&tile_set);
        let mut win = Graphic::load_from(Cursor::new(&include_bytes!("../win")[..])).unwrap().textured(&texture_creator);
        win.update_texture(&tile_set);

        GraphicsSet {
            table: table,
            back_card: back_card,
            cards: cards,
            well: cell,
            black_border: black_border,
            win: win,
            tile_set: tile_set,
            emblem: emblem,
            emblem_shadow: emblem_shadow,
            highlight_card: highlight_card
        }
    }

}

impl <T> Index<Card> for GraphicsSet<T> {
    type Output = Graphic<T>;
    fn index(&self, index: Card) -> &Graphic<T> {
        match index.suit {
            Suit::Hearts => &self.cards[index.value as usize -1],
            Suit::Diamonds => &self.cards[index.value as usize +12],
            Suit::Spades => &self.cards[index.value as usize +25],
            Suit::Clubs => &self.cards[index.value as usize +38],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Suit {
    Hearts, Clubs, Diamonds, Spades
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Card {
    pub value : u8,
    pub suit : Suit
}
impl Card {
    pub fn deck() -> Vec<Card> {
        let mut ret = Vec::new();
        for i in 1..=13 {
            ret.push(Card{suit:Suit::Hearts, value:i})
        }
        for i in 1..=13 {
            ret.push(Card{suit:Suit::Diamonds, value:i})
        }
        for i in 1..=13 {
            ret.push(Card{suit:Suit::Clubs, value:i})
        }
        for i in 1..=13 {
            ret.push(Card{suit:Suit::Spades, value:i})
        }
        ret.shuffle(&mut thread_rng());
        ret
    }
    fn draw_well<'r>(canvas:&mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>, position: (i32,i32)) {
        graphics.black_border.draw(canvas,(position.0-5,position.1-5));
        graphics.black_border.draw(canvas,(position.0-7,position.1-5));
        graphics.black_border.draw(canvas,(position.0-7,position.1-7));
        graphics.black_border.draw(canvas,(position.0-5,position.1-7));
        graphics.well.draw(canvas,(position.0-6,position.1-6));
    }
    fn draw_back<'r>(canvas:&mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>, position: (i32,i32)) {
        graphics.black_border.draw(canvas,(position.0-5,position.1-5));
        graphics.black_border.draw(canvas,(position.0-7,position.1-5));
        graphics.black_border.draw(canvas,(position.0-7,position.1-7));
        graphics.black_border.draw(canvas,(position.0-5,position.1-7));
        graphics.back_card.draw(canvas,(position.0-6,position.1-6));
    }
    fn draw<'r>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>, position : (i32,i32)) {
        graphics.black_border.draw(canvas,(position.0-5,position.1-5));
        graphics.black_border.draw(canvas,(position.0-7,position.1-5));
        graphics.black_border.draw(canvas,(position.0-7,position.1-7));
        graphics.black_border.draw(canvas,(position.0-5,position.1-7));
        graphics[*self].draw(canvas,(position.0-6,position.1-6));
    }

}

pub struct Well {
    pub id: usize,
    pub position: (i32, i32),
    pub cards: Vec<Card>,
    trail: usize,
    current_trail:usize
}
impl Well {
    fn draw<'r>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>) {        
        let cards : &[Card] = &self.cards[self.cards.len() - (1 + self.current_trail).min(self.cards.len())..];

        if cards.len() > 0 {
            let mut x = self.position.0;
            for i in cards {
                i.draw(canvas,graphics,(x,self.position.1));
                x += 14;
            }
        } else {
            Card::draw_well(canvas,graphics,self.position);
        }
    }
    fn top_position(&self) -> (i32,i32) {
        if self.cards.len() == 0 {
            self.position
        } else {
            let trail = self.current_trail.min(self.cards.len() - 1);
            (self.position.0 + trail as i32 * 14, self.position.1)
        }
    }
    fn top_position_with(&self, num: usize) -> (i32,i32) {
        if self.cards.len() > 0 {
            let trail = self.current_trail.min(self.cards.len() - 1);
            (self.position.0  + (trail + num).min(self.trail) as i32 * 14, self.position.1)
        } else {
            (self.position.0 + (num-1.min(num)).min(self.trail) as i32 * 14, self.position.1)
        }
    }
    fn skim(&mut self) -> Vec<Card> {
        let mut  ret = Vec::new();
        if let Some(x) = self.cards.last() {
            ret.push(*x);
            self.cards.truncate(self.cards.len()-1);
            if self.current_trail > 0 { self.current_trail -= 1 };
        }
        
        ret
    }
}
pub struct Stack {
    pub id: usize,
    pub position: (i32, i32),
    pub cards: Vec<Card>,
    pub hidden_point: usize,
    pub frame_visible: bool,
}
impl Stack {
    fn split_at(&mut self, index: usize) -> Vec<Card> {
        let ret = self.cards[index..].to_vec();
        self.cards.truncate(index);
        ret
    }
    pub fn top_position(&self) -> (i32,i32) {
        let length = self.cards.len() as i32;
        let hidden = self.hidden_point as i32;
        let y = self.position.1 + hidden.min(length) * 6 + (length - hidden).max(0) * 10;
        (self.position.0,y)
    } 
    fn draw_cards<'r>(cards: &Vec<Card>, hidden_point: usize, frame_visible: bool,  position: (i32,i32), canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>) {
        Self::draw_cards_anim(cards, hidden_point, frame_visible, position, canvas, graphics, 0)
    }
    fn draw_cards_anim<'r>(cards: &Vec<Card>, hidden_point: usize, frame_visible: bool, position: (i32,i32), canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>, reduction: i32) {
        if cards.len() == 0 && frame_visible {
            Card::draw_well(canvas, graphics, position);
        } else {
            let x = position.0;
            let mut red = reduction;
            let mut y = position.1;
            let mut hide = hidden_point;
            for i in cards {
                if hide > 0 {
                    Card::draw_back(canvas, graphics, (x,y));
                    hide -= 1;
                    y += (6 - red.min(6)).max(0);
                    red = (red - 6).max(0);
                } else {                
                    i.draw(canvas,graphics,(x,y));
                    y += (10 - red.min(10)).max(0);
                    red = (red - 10).max(0);
                }
            }
        }
    }
    fn draw<'r>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>) {
        Self::draw_cards(&self.cards, self.hidden_point, self.frame_visible, self.position, canvas, graphics)
    }
    
}

pub struct Deck {
    pub id: usize,
    pub position: (i32, i32),
    pub cards: Vec<Card>,
    pub emblem: bool,
}
impl Deck {
    fn draw<'r>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>) {
        if self.cards.len() > 0 {
            Card::draw_back(canvas, graphics, self.position);
        } else {
            Card::draw_well(canvas, graphics, self.position);
            if self.emblem {
                graphics.emblem_shadow.draw(canvas, (self.position.0 + 6 -1, self.position.1 + 14 -1));
                graphics.emblem_shadow.draw(canvas, (self.position.0 + 6 +1, self.position.1 + 14 -1 ));
                graphics.emblem_shadow.draw(canvas, (self.position.0 + 6 -1, self.position.1 + 14 +1 ));
                graphics.emblem_shadow.draw(canvas, (self.position.0 + 6 +1, self.position.1 + 14 +1 ));
                graphics.emblem_shadow.draw(canvas, (self.position.0 + 6, self.position.1 + 14 +1 ));
                graphics.emblem_shadow.draw(canvas, (self.position.0 + 6, self.position.1 + 14 -1 ));
                graphics.emblem_shadow.draw(canvas, (self.position.0 + 6 +1, self.position.1 + 14));
                graphics.emblem_shadow.draw(canvas, (self.position.0 + 6 -1, self.position.1 + 14 ));
                graphics.emblem.draw(canvas, (self.position.0 + 6, self.position.1 + 14));
            }
        }
    }
}
enum AnimationType {
    MoveStack { cards: Vec<Card>, start: (i32,i32), end: (i32,i32) },
    MoveHidden { start: (i32,i32), end: (i32,i32) },
    CollapseStack { cards: Vec<Card>, position: (i32,i32) },
    Highlight { position: (i32,i32), height: u32, color: (u8,u8,u8) },
}
struct Animation {
    time: u32,
    anim_type: AnimationType,
    and_then: Box<dyn FnOnce(&mut Table)>
}
impl Animation {
    fn draw<'r>(&self, canvas: &mut Canvas<Window>, graphics: & GraphicsSet<Texture<'r>>, frame: u32) {
        match &self.anim_type {
            AnimationType::MoveStack{ cards, start: (sx,sy), end: (ex,ey) } => {
                let x = sx + ((ex - sx) * frame as i32 / self.time as i32); 
                let y = sy + ((ey - sy) * frame as i32 / self.time as i32); 
                Stack::draw_cards(&cards, 0, false, (x,y), canvas, graphics);
            },
            AnimationType::MoveHidden{ start: (sx,sy), end: (ex,ey) } => {
                let x = sx + ((ex - sx) * frame as i32 / self.time as i32); 
                let y = sy + ((ey - sy) * frame as i32 / self.time as i32); 
                Card::draw_back(canvas, graphics, (x,y));
            },
            AnimationType::CollapseStack{ cards, position } => {
                let reduction = 10 * cards.len() as i32 * frame as i32 / self.time as i32;
                Stack::draw_cards_anim(cards, 0, false, *position, canvas, graphics, reduction);
            },
            AnimationType::Highlight{ position, color: (r,g,b), height } => {
                let alpha = frame * 255 * 2 / self.time;
                let color = rgba(*r,*g,*b, (if alpha > 255 {255 - (alpha - 256) } else {alpha}) as u8);
                canvas.set_draw_color(color);
                canvas.draw_rect(Rect::new(position.0,position.1,28,*height)).unwrap();
                canvas.draw_rect(Rect::new(position.0-1,position.1,28,*height)).unwrap();
                canvas.draw_rect(Rect::new(position.0+1,position.1,28,*height)).unwrap();
                canvas.draw_rect(Rect::new(position.0,position.1-1,28,*height)).unwrap();
                canvas.draw_rect(Rect::new(position.0,position.1+1,28,*height)).unwrap();
            }
        }
    }
    
}
pub struct Table {    
    wells: Vec<Well>,
    stacks: Vec<Stack>,
    decks: Vec<Deck>,
    history: Vec<Vec<Move>>,
    animations: Vec<(Animation,u32)>, 
    move_count: u32,   
    selection: Option<GameObject>
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Move {
    Shift(GameObject, GameObject, usize),
    Reveal(usize, i32)
}


impl Table {

    pub fn add_deck(&mut self, position: (i32,i32), cards: &[Card]) -> usize {
        let id = self.decks.len();
        self.decks.push(Deck { id: id, position: position, cards: cards.to_vec(), emblem: false });
        id
    }
    pub fn add_deck_with_emblem(&mut self, position: (i32,i32), cards: &[Card]) -> usize {
        let id = self.decks.len();
        self.decks.push(Deck { id: id, position: position, cards: cards.to_vec(), emblem: true });
        id
    }
    pub fn add_well(&mut self, position: (i32, i32), trail: usize, cards: &[Card]) -> usize {
        let id = self.wells.len();
        self.wells.push(Well { id: id, position: position, trail:trail, current_trail: trail.min(cards.len()), cards: cards.to_vec() });
        id
    }
    pub fn add_stack(&mut self, position: (i32, i32), cards: &[Card], hidden_point: usize ) -> usize {
        let id = self.stacks.len();
        self.stacks.push(Stack { id: id, position: position, cards: cards.to_vec(), hidden_point: hidden_point, frame_visible: true });
        id
    }
    pub fn add_stack_nobase(&mut self, position: (i32, i32), cards: &[Card], hidden_point: usize ) -> usize {
        let id = self.stacks.len();
        self.stacks.push(Stack { id: id, position: position, cards: cards.to_vec(), hidden_point: hidden_point, frame_visible: false });
        id
    }
    pub fn stacks(&self) -> &[Stack] {
        &self.stacks
    }
    pub fn wells(&self) -> &[Well] {
        &self.wells
    }
    #[allow(dead_code)]
    pub fn decks(&self) -> &[Deck] {
        &self.decks
    }
    pub fn stack(&self, index: usize) -> &Stack {
        &self.stacks[index]
    }
    pub fn deck(&self, index: usize) -> &Deck {
        &self.decks[index]
    }
    pub fn well(&self, index: usize) -> &Well {
        &self.wells[index]
    }
    pub fn reveal(&mut self, stack_id: usize, num: i32) {
        self.reveal_then(stack_id, num, Box::new(|_| {}))
    }
    fn reveal_then_raw(&mut self, stack_id: usize, num: i32, and_then: Box<dyn FnOnce(&mut Table)>) {
        self.stacks[stack_id].hidden_point = (self.stacks[stack_id].hidden_point as i32 - num).max(0) as usize;
        and_then(self)   
    }
    pub fn reveal_then(&mut self, stack_id: usize, num: i32, and_then: Box<dyn FnOnce(&mut Table)>) {
        if self.stacks[stack_id].hidden_point != (self.stacks[stack_id].hidden_point as i32 - num).max(0) as usize {
            if let Some(h) = self.history.last_mut() {
                h.push(Move::Reveal(stack_id,num))
            }
            self.reveal_then_raw(stack_id, num, and_then)
        }
    }
    fn shift_then_raw(&mut self, src: GameObject, dest: GameObject, hidden: bool, num_cards:usize, and_then: Box<dyn FnOnce(&mut Table)>) {
        
        let cards = match src {
            //TODO this should use functions on the objects
            GameObject::Deck(i) => {
                let len = self.deck(i).cards.len();
                let cards = self.deck(i).cards[len - num_cards.min(len)..].to_vec().drain(..).rev().collect();
                self.decks[i].cards.truncate(len - num_cards.min(len));
                cards
            }
            GameObject::Stack(i) => { 
                let len = self.stack(i).cards.len();
                let cards = self.stack(i).cards[len - num_cards.min(len)..].to_vec();
                self.stacks[i].cards.truncate(len - num_cards.min(len));
                cards
            }
            GameObject::Well(i) => {
                let len = self.well(i).cards.len();
                let cards = self.well(i).cards[len - num_cards.min(len)..].to_vec();
                self.wells[i].cards.truncate(len - num_cards.min(len));
                self.wells[i].current_trail -= num_cards.min(len).min(self.wells[i].current_trail);
                cards
            }
        };
        let start_pos = match src {
            GameObject::Deck(i) => self.deck(i).position,
            GameObject::Stack(i) => self.stack(i).top_position(),
            GameObject::Well(i) => self.well(i).top_position_with(cards.len()),
        };
        self.animate_add_cards_to(dest,&cards, start_pos, hidden, and_then)

    }
    pub fn shift_then(&mut self, src: GameObject, dest: GameObject, num_cards:usize, and_then: Box<dyn FnOnce(&mut Table)>) {
        if src != dest {
            self.shift_then_raw(src, dest, src.is_deck() || dest.is_deck(), num_cards, and_then);
            if let Some(h) = self.history.last_mut() {
                h.push(Move::Shift(src,dest,num_cards))
            }
        }
    }
    fn multi_shift_helper(&mut self, srcs: Vec<(GameObject, usize)>,i: usize,  dest: GameObject, and_then: Box<dyn FnOnce(&mut Table)>) {
        if i < srcs.len() {
            let (src,num_cards) = srcs[i];
            self.shift_then_raw(src, dest, src.is_deck() || dest.is_deck(), num_cards, Box::new(move |tbl| tbl.multi_shift_helper(srcs,i+1,dest, and_then)));
        } else {
            and_then(self);
        }
    }
    pub fn multi_shift_then(&mut self, srcs: Vec<(GameObject, usize)>, dest: GameObject, and_then: Box<dyn FnOnce(&mut Table)>) {
        for (src,num_cards) in &srcs {
            if let Some(h) = self.history.last_mut() {
                h.push(Move::Shift(*src,dest,*num_cards))
            }
        }
        self.multi_shift_helper(srcs,0,dest,and_then)
    }
    pub fn shift(&mut self, src: GameObject, dest: GameObject, num_cards:usize) {
            self.shift_then(src,dest,num_cards,Box::new(|_| {}));
    }
    pub fn end_move(&mut self) {
        self.deselect();
        if if let Some(h) = self.history.last() {
                h.len() > 0
            } else { true }  {
            self.history.push(Vec::new());
            self.move_count += 1
        }
    }
    pub fn can_undo(&self) -> bool {
        if self.animations.len() > 0 { return false };
        if self.history.len() > 1 {
            if let Some(h) = self.history.last() {
                h.len() == 0
            } else { false }
        } else { false }
    }
    pub fn undo(&mut self) {
        self.deselect();
        if self.can_undo() {
            self.history.truncate(self.history.len() - 1);
            let mut moves = if let Some(h) = self.history.last_mut() {                    
                h.drain(..).rev().collect()                     
            } else { Vec::new() };
            for i in moves.drain(..) {
                match i {
                    Move::Reveal(s,i) => self.reveal_then_raw(s, -i, Box::new(|_|{})),
                    Move::Shift(s,d,i) => { 
                        self.shift_then_raw(d, s, s.is_deck() ||d.is_deck(), i, Box::new(move |tbl|{
                            //hack
                            match d { GameObject::Well(w) => tbl.wells[w].current_trail = tbl.wells[w].trail, _ => {} }
                        }));
                        match d { GameObject::Well(w) => self.wells[w].current_trail = self.wells[w].trail, _ => {} }
                    },
                }
            }
        }
    }
    fn animate_highlight(&mut self, pos: (i32,i32), time: u32, color: (u8,u8,u8), height: u32, and_then: Box<dyn FnOnce(&mut Table)>) {
        self.animations.push((Animation { time: time, anim_type: AnimationType::Highlight{position: pos, color:color, height:height},and_then: and_then},0))
    }
    fn animate_move_stack(&mut self, start: (i32,i32), end: (i32,i32), cards: Vec<Card>, time: u32, and_then: Box<dyn FnOnce(&mut Table)>) {
        self.animations.push((Animation { time: time, anim_type: AnimationType::MoveStack{cards: cards, start: start, end:end}, and_then: and_then},0))
    }
    fn animate_move_hidden(&mut self, start: (i32,i32), end: (i32,i32), time: u32, and_then: Box<dyn FnOnce(&mut Table)>) {
        self.animations.push((Animation { time: time, anim_type: AnimationType::MoveHidden{start: start, end:end}, and_then: and_then},0))
    }
    fn animate_collapse_stack(&mut self, position: (i32,i32), cards: Vec<Card>, time: u32, and_then: Box<dyn FnOnce(&mut Table)>) {
        self.animations.push((Animation { time: time, anim_type: AnimationType::CollapseStack{cards: cards, position:position}, and_then: and_then},0))
    }
    pub fn add_cards_to(&mut self, dest: GameObject, cards: &[Card]) {
        //TODO use functions on the objects
        match dest {
            GameObject::Deck(i) => self.decks[i].cards.append(&mut cards.to_vec().drain(..).rev().collect()),
            GameObject::Stack(i) =>  self.stacks[i].cards.append(&mut cards.to_vec()),
            GameObject::Well(i) => { self.wells[i].cards.append(&mut cards.to_vec()); self.wells[i].current_trail = (self.wells[i].current_trail + cards.len()).min(self.wells[i].trail)},
        }
    }
    pub fn animate_highlight_stack(&mut self, stack_id: usize, position:usize, color: (u8,u8,u8)) {
        self.animate_highlight_stack_then(stack_id, position, color, Box::new(|_| {}));
    }
    pub fn animate_highlight_well(&mut self, well_id: usize, color: (u8,u8,u8)) {
        self.animate_highlight_well_then(well_id, color, Box::new(|_| {}));
    }
    pub fn animate_highlight_deck(&mut self, deck_id: usize, color: (u8,u8,u8)) {
        self.animate_highlight_deck_then(deck_id, color, Box::new(|_| {}));
    }
    pub fn animate_highlight_well_then(&mut self, well_id: usize, color: (u8,u8,u8),and_then:Box<dyn FnOnce(&mut Table)>) {
        self.animate_highlight(self.well(well_id).top_position(), 30, color, 36, and_then);
    }
    pub fn animate_highlight_deck_then(&mut self, deck_id: usize, color: (u8,u8,u8),and_then:Box<dyn FnOnce(&mut Table)>) {
        self.animate_highlight(self.deck(deck_id).position, 30, color, 36, and_then);
    }
    pub fn animate_highlight_stack_then(&mut self, stack_id: usize, position:usize, color: (u8,u8,u8),and_then:Box<dyn FnOnce(&mut Table)>) {
        let mut offset_y = 0;
        let len = self.stacks[stack_id].cards.len();
        for i in 0..len.min(position) {
            if i != len - 1 {
                if i < self.stacks[stack_id].hidden_point {
                    offset_y += 6;
                } else {
                    offset_y += 10;
                }
            }
        }
        let mut height = 36;
        if len > 0 {
            for i in len.min(position)..len {
                if i != len - 1 {
                    if i < self.stacks[stack_id].hidden_point {
                        height += 6;
                    } else {
                        height += 10;
                    }
                }
            }
        }
        let (x,y) = self.stacks[stack_id].position;
        self.animate_highlight((x,y+offset_y), 30, color, height, and_then);
    }
    pub fn animate_add_cards_to(&mut self, dest:GameObject, cards :&[Card], start_pos: (i32,i32), hidden: bool, and_then: Box<dyn FnOnce(&mut Table)>) {
        let dest_pos = match dest {
            GameObject::Deck(i) => self.deck(i).position,
            GameObject::Stack(i) => self.stack(i).top_position(),
            GameObject::Well(i) => self.well(i).top_position_with(cards.len())
        };
        let len = cards.len();
        let cardso = cards.to_vec();
        let f : Box<dyn FnOnce(&mut Table)> = Box::new(move |tbl| {
            if len > 1 && !dest.is_stack() && !hidden {
                let cardso1 = cardso.to_vec();
                tbl.animate_collapse_stack(dest_pos, cardso, 28, Box::new(move |tbl| {
                    tbl.add_cards_to(dest, &cardso1);
                    and_then(tbl)
                }));
            } else {
                tbl.add_cards_to(dest,&cardso);
                and_then(tbl)
            }
        });
        if hidden { 
            self.animate_move_hidden(start_pos, dest_pos, 14, f);
        } else {
            self.animate_move_stack(start_pos, dest_pos, cards.to_vec(), 14, f);
        }
    } 
    fn new<T:Rules>() -> Table {
        let mut table = Table {
            decks: Vec::new(),
            animations: Vec::new(),
            wells: Vec::new(),
            stacks: Vec::new(),
            history: Vec::new(),
            move_count: 0,
            selection: None
        };        
        T::new_game(&mut table);
        table.history.push(Vec::new());
        table
    }

    fn draw<'t>(&self, canvas: &mut Canvas<Window>, show_selection: bool, graphics: &GraphicsSet<Texture<'t>>) {
        graphics.table.draw(canvas,(0,0));
        for i in &self.decks {
            i.draw(canvas,graphics)
        }
        for i in &self.wells {
            i.draw(canvas,graphics)
        }
        for i in &self.stacks {
            i.draw(canvas,graphics)
        }
        for (x,i) in &self.animations {
            x.draw(canvas,graphics, *i)
        }
        if show_selection && self.animations.len() == 0 {
            if let Some(n) = self.selection {
                let pos = match n {
                    GameObject::Well(i)  => self.well(i).top_position(),
                    GameObject::Deck(i)  => self.deck(i).position,
                    GameObject::Stack(i) => { let p = self.stack(i).top_position(); (p.0,p.1-10)},
                };
                graphics.highlight_card.draw(canvas,(pos.0-6,pos.1-6));
            }
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

    fn collides_stack(&self, position:(i32,i32)) -> Option<(usize,usize,(i32,i32))> {
        for i in self.stacks.iter().rev() {
            if i.cards.len() != 0 || i.frame_visible {
                if position.0 >= i.position.0 && position.0 < i.position.0 + 26 {
                    let offset_x = i.position.0 - position.0;            
                    if position.1 >= i.position.1 {
                        let mut remaining = position.1 - i.position.1;
                        let mut offset_y = i.position.1 - position.1;
                        for j in 0..i.cards.len() {
                            let delta = if j < i.hidden_point { 6 } else { 10 };
                            remaining -= delta;                        
                            if remaining < 0 {
                                return Some((i.id,j,(offset_x,offset_y)))
                            }
                            if j != i.cards.len()-1 { offset_y += delta; }
                        }
                        if remaining < 36 && i.cards.len() > 0 {
                            return Some((i.id,i.cards.len()-1,(offset_x, offset_y)))
                        } else if remaining < 36 {
                            return Some((i.id,0,(offset_x, offset_y)))
                        }
                    }
                }
            }
        }
        None
    }
    fn collides_deck(&self, position: (i32,i32)) -> Option<usize> {
        for i in &self.decks {
            if position.0 >= i.position.0 && position.0 < i.position.0 + 26 {
                if position.1 >= i.position.1 && position.1 < i.position.1 + 36 {
                    return Some(i.id)
                }
            }
        };
        None
    }
    fn collides_well(&self, position: (i32,i32)) -> Option<(usize, (i32,i32))> {
        for i in &self.wells {
            if position.0 >= i.top_position().0 && position.0 < i.top_position().0 + 26 {
                let offset_x = i.top_position().0 - position.0;
                if position.1 >= i.top_position().1 && position.1 < i.top_position().1 + 36 {
                    let offset_y = i.top_position().1 - position.1;
                    return Some((i.id,(offset_x,offset_y)));
                }
            }
        };
        None
    }
    pub fn select(&mut self, obj: GameObject) {
        self.selection = Some(obj);
    }
    pub fn deselect(&mut self) {
        self.selection = None;
    }

}

pub trait Rules {
    fn table_size() -> (u32,u32);
    fn new_game(table: &mut Table);

    fn can_split_stack(stack : &Stack, position : usize, table: &Table) -> bool;
    fn can_place_stack(stack : &Stack, cards: &[Card]) -> bool;
    fn can_place_well(well : &Well, cards: &[Card]) -> bool;
    fn can_skim_well(well : &Well) -> bool;
    fn game_won(table: &Table) -> bool;

    fn placed_in_stack(table: &mut Table, stack_id : usize, cards: usize); 
    fn placed_in_well(table: &mut Table, well_id : usize, cards: usize);
    fn deal_from_deck(table: &mut Table, deck_id : usize);

    fn stack_clicked(table: &mut Table, stack_id : usize, position:usize);
    fn well_clicked(table: &mut Table, well_id : usize);

    fn hint(table: &mut Table);


}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameObject {
    Stack(usize), Well(usize), Deck(usize)
}
impl GameObject {
    pub fn is_stack(&self) -> bool {
        match *self {
            GameObject::Stack(_) => true, _ => false
        }
    }
    pub fn is_deck(&self) -> bool {
        match *self {
            GameObject::Deck(_) => true, _ => false
        }
    }
    #[allow(dead_code)]
    pub fn is_well(&self) -> bool {
        match *self {
            GameObject::Well(_) => true, _ => false
        }
    }
}
//const RULES : Spider = Spider {};
fn main_loop<RULES:Rules>(mut window:Window, sdl_context: &Sdl) -> (Option<Variant>,Window) {

    
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
    let mut attached_cards : Option<(Vec<Card>, GameObject)> = None;

    let mut grab_offset : (i32,i32) = (0,0);
    let wwh = RULES::table_size().1;
    let mut menu = MenuBar::new(RULES::table_size().0)
                    .add(Menu::new("GAME",152,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Klondike 1-Draw", 52, Keycode::Num1,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Klondike 3-Draw", 53, Keycode::Num2,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(136, &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Spider 1-Suit", 54, Keycode::Num3,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Spider 2-Suit", 55, Keycode::Num4,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Spider 4-Suit", 56, Keycode::Num5,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(136, &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Pyramid 1-Draw", 352, Keycode::F1,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Pyramid 3-Draw", 353, Keycode::F2,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(136, &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Golf", 57, Keycode::Num6,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("TriPeaks", 58, Keycode::Num7,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(136, &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("FreeCell", 59, Keycode::Num8,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Cruel", 60, Keycode::Num9,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(136, &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Quit",363, Keycode::F12,&texture_creator,&graphics_set.tile_set)))
                    .add(Menu::new("ACTION",88,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Hint",9, Keycode::H,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Undo",27, Keycode::Z,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(72, &texture_creator, &graphics_set.tile_set))
                            .add(MenuItem::new("Restart",15, Keycode::N,&texture_creator,&graphics_set.tile_set)))
                    .add(Menu::new("VIEW",112,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Micro-mode",360, Keycode::F9,&texture_creator, &graphics_set.tile_set)));
    let mut rate_limiter = FPSManager::new();
    rate_limiter.set_framerate(60).unwrap();
    let mut micro_mode = false;
    loop {
        if let Some((cards,_)) = &attached_cards {
            table.draw(&mut canvas, false, &graphics_set);
            Stack::draw_cards(&cards, 0, false, (mx + grab_offset.0,my + grab_offset.1), &mut canvas, &graphics_set);
        } else {
            table.draw(&mut canvas,true,&graphics_set);
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
        let won = attached_cards == None && RULES::game_won(&table);
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
                        Event::KeyDown { keycode: Some(Keycode::Num1), ..} => {
                            return (Some(Variant::KlondikeOne), canvas.into_window());
                        },
                        Event::KeyDown { keycode: Some(Keycode::Num2), ..} => {
                            return (Some(Variant::KlondikeThree), canvas.into_window());
                        },
                        Event::KeyDown { keycode: Some(Keycode::Num3), ..} => {
                            return (Some(Variant::SpiderOne), canvas.into_window());
                        },
                        Event::KeyDown { keycode: Some(Keycode::Num4), ..} => {
                            return (Some(Variant::SpiderTwo), canvas.into_window());
                        },
                        Event::KeyDown { keycode: Some(Keycode::Num5), ..} => {
                            return (Some(Variant::SpiderFour), canvas.into_window());
                        },
                        Event::KeyDown { keycode: Some(Keycode::Num6), ..} => {
                            return (Some(Variant::Golf), canvas.into_window());
                        },
                        Event::KeyDown { keycode: Some(Keycode::Num7), ..} => {
                            return (Some(Variant::TriPeaks), canvas.into_window());
                        },
                        Event::KeyDown { keycode: Some(Keycode::Num8), ..} => {
                            return (Some(Variant::FreeCell), canvas.into_window());
                        },
                        Event::KeyDown { keycode: Some(Keycode::Num9), ..} => {
                            return (Some(Variant::Cruel), canvas.into_window());
                        },
                        Event::KeyDown { keycode: Some(Keycode::F1), ..} => {
                            return (Some(Variant::PyramidOne), canvas.into_window());
                        },
                        Event::KeyDown { keycode: Some(Keycode::F2), ..} => {
                            return (Some(Variant::PyramidThree), canvas.into_window());
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
                        Event::KeyDown { keycode: Some(Keycode::H), ..} => {
                            RULES::hint(&mut table);
                        },
                        Event::MouseButtonDown { mouse_btn: _, x, y, ..} if !won => {
                            let (sx,sy) = (x,y);//(x * dw as i32 / ww as i32, y * dh as i32 / wh as i32);
                            md = true;
                            mx = sx; my = sy;
                            if let Some((idx, pos, offset)) = table.collides_stack((mx,my)) {
                                if RULES::can_split_stack(&table.stacks[idx], pos, &table) {
                                    attached_cards = Some((table.stacks[idx].split_at(pos), GameObject::Stack(idx)));
                                    grab_offset = offset;
                                }
                            }
                            if let Some((idx,offset)) = table.collides_well((mx,my)) {
                                if RULES::can_skim_well(&table.wells[idx]) {
                                    attached_cards = Some((table.wells[idx].skim(), GameObject::Well(idx)));
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
                            if let None = attached_cards {
                                if let Some(idx) = table.collides_deck((mx,my)) {                                
                                    RULES::deal_from_deck(&mut table,idx);
                                    table.end_move();
                                }
                            }
                            if let Some((cards,origin)) = attached_cards {
                                if !md {
                                    if let Some((idx,pos,_)) = table.collides_stack((mx,my)) {
                                        if origin != GameObject::Stack(idx) {
                                            let len = table.stacks[idx].cards.len();
                                            if len == 0 || pos == len -1 { 
                                                if RULES::can_place_stack(&table.stacks[idx], &cards) {
                                                    let l = cards.len();
                                                    table.animate_add_cards_to(GameObject::Stack(idx), &cards,(mx + grab_offset.0,my + grab_offset.1), false,Box::new(move |tbl : & mut Table| {
                                                        RULES::placed_in_stack(tbl, idx, l);
                                                        tbl.end_move();
                                                    }));
                                                    placed = true;
                                                    if let Some(h) = table.history.last_mut() {
                                                        h.push(Move::Shift(origin, GameObject::Stack(idx), cards.len()));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if let Some((idx,_)) = table.collides_well((mx,my)) {
                                        if origin != GameObject::Well(idx) {
                                            if RULES::can_place_well(&table.wells[idx], &cards) {
                                                let len = cards.len();
                                                table.animate_add_cards_to(GameObject::Well(idx), &cards,(mx + grab_offset.0,my + grab_offset.1),false, Box::new(move |tbl : & mut Table| {
                                                    RULES::placed_in_well(tbl, idx, len);
                                                    tbl.end_move();
                                                }));
                                                placed = true;
                                                if let Some(h) = table.history.last_mut() {
                                                    h.push(Move::Shift(origin, GameObject::Well(idx), len));
                                                }
                                            }
                                        }
                                    }
                                } 
                                if !placed {
                                    if md {
                                        md = false;
                                        table.add_cards_to(origin, &cards);
                                        match origin {
                                            GameObject::Stack(d) => {let len = table.stack(d).cards.len() - cards.len(); RULES::stack_clicked(&mut table,d, len)}, 
                                            GameObject::Well(d) => RULES::well_clicked(&mut table,d),
                                            _ => {},
                                        }
                                    } else {
                                        table.animate_add_cards_to(origin, &cards, (mx + grab_offset.0,my + grab_offset.1),false, Box::new(|_| {}));
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

enum Variant {
    KlondikeThree,
    KlondikeOne,
    SpiderFour,
    SpiderTwo,
    SpiderOne,
    Golf,
    Cruel,
    TriPeaks,
    FreeCell,
    PyramidOne,
    PyramidThree
}
fn choose(window : Window, sdl_context:&Sdl, variant : Variant) {
    if let (Some(v), w) = match variant {
        Variant::KlondikeOne   => main_loop::<Klondike<OneDraw>>(window,sdl_context),
        Variant::KlondikeThree => main_loop::<Klondike<ThreeDraw>>(window,sdl_context),
        Variant::SpiderOne     => main_loop::<Spider<OneSuit>>(window,sdl_context),
        Variant::SpiderTwo     => main_loop::<Spider<TwoSuit>>(window,sdl_context),
        Variant::SpiderFour    => main_loop::<Spider<FourSuit>>(window,sdl_context),
        Variant::Golf          => main_loop::<Golf>(window,sdl_context),
        Variant::Cruel         => main_loop::<Cruel>(window,sdl_context),
        Variant::TriPeaks      => main_loop::<TriPeaks>(window,sdl_context),
        Variant::FreeCell      => main_loop::<FreeCell>(window,sdl_context),
        Variant::PyramidOne    => main_loop::<Pyramid<OneDraw>>(window,sdl_context),
        Variant::PyramidThree  => main_loop::<Pyramid<ThreeDraw>>(window,sdl_context),
    } {
        choose(w, sdl_context,v);
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("solitude", 320, 240)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    let arg = env::args().nth(1).unwrap_or(String::from("spider2"));
    match &arg[..] {
        "pyramid3" => choose(window,&sdl_context,Variant::PyramidThree),
        "pyramid1" => choose(window,&sdl_context,Variant::PyramidOne),
        "klondike3" => choose(window,&sdl_context,Variant::KlondikeThree),
        "klondike1" => choose(window,&sdl_context,Variant::KlondikeOne),
        "golf"      => choose(window,&sdl_context,Variant::Golf),
        "tripeaks"  => choose(window,&sdl_context,Variant::TriPeaks),
        "spider1"   => choose(window,&sdl_context,Variant::SpiderOne),
        "spider2"   => choose(window,&sdl_context,Variant::SpiderTwo),
        "spider4"   => choose(window,&sdl_context,Variant::SpiderFour),
        "freecell"  => choose(window,&sdl_context,Variant::FreeCell),
        "cruel"     => choose(window,&sdl_context,Variant::Cruel),
        _ => println!("Available games: klondike1, klondike3, spider1, spider2, spider4, pyramid1, pyramid3, cruel, golf, tripeaks, freecell")
    }
    //cards::main_loop::<Spider<OneSuit>>();
    
}