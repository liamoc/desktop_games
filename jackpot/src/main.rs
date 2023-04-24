extern crate tesserae;
extern crate sdl2;

use rand::seq::SliceRandom;

use rand::Rng;
use tesserae::{*};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use utils::framerate::FPSManager;
use sdl2::render::TextureCreator;
use sdl2::render::Texture;
use sdl2::render::RenderTarget;
use sdl2::render::Canvas;
use utils::color::{*};
use sdl2::rect::{Rect};
use utils::menu::{*};
use rand::thread_rng;
use std::io::Cursor;

use std::f32;

struct Emblem<'r> {
    graphic: Graphic<Texture<'r>>,
    shadow: Graphic<Texture<'r>>
}
impl <'r> Emblem<'r> {
    fn new<T>(start: usize, color: Color, shadow_color: Color, tile_set:  &TileSet, texture_creator : &'r TextureCreator<T>) -> Emblem<'r> {
        let mut graphic = Graphic::blank(2,2).textured(texture_creator);
        let mut shadow  = Graphic::blank(2,2).textured(texture_creator);

        if start == 292 {
            graphic[(0,0)] = Tile { fg: color, bg: TRANSPARENT, index: 288};
            graphic[(1,0)] = Tile { fg: color, bg: TRANSPARENT, index: 289};
            shadow[(0,0)] = Tile { fg: shadow_color, bg: TRANSPARENT, index: 288};
            shadow[(1,0)] = Tile { fg: shadow_color, bg: TRANSPARENT, index: 289};
        } else {
            graphic[(0,0)] = Tile { fg: color, bg: TRANSPARENT, index: start};
            graphic[(1,0)] = Tile { fg: color, bg: TRANSPARENT, index: start + 1};
            shadow[(0,0)] = Tile { fg: shadow_color, bg: TRANSPARENT, index: start};
            shadow[(1,0)] = Tile { fg: shadow_color, bg: TRANSPARENT, index: start + 1};
        }
        if start == 288 {
            graphic[(0,1)] = Tile { bg: color, fg: TRANSPARENT, index: 207};
            graphic[(1,1)] = Tile { bg: color, fg: TRANSPARENT, index: 206};
            shadow[(0,1)] = Tile { bg: shadow_color, fg: TRANSPARENT, index: 207};
            shadow[(1,1)] = Tile { bg: shadow_color, fg: TRANSPARENT, index: 206};
        } else {
            graphic[(0,1)] = Tile { fg: color, bg: TRANSPARENT, index: start + 16};
            graphic[(1,1)] = Tile { fg: color, bg: TRANSPARENT, index: start + 17};
            shadow[(0,1)] = Tile { fg: shadow_color, bg: TRANSPARENT, index: start + 16};
            shadow[(1,1)] = Tile { fg: shadow_color, bg: TRANSPARENT, index: start + 17};
        }
        graphic.update_texture(&tile_set);
        shadow.update_texture(&tile_set);
        Emblem {
            graphic, shadow
        }
    }
    fn draw<T:RenderTarget>(&self, canvas : &mut Canvas<T>, pos_unshifted: (i32,i32)) {
        let pos = (pos_unshifted.0 + 4, pos_unshifted.1);
        self.shadow.draw(canvas,(pos.0+1,pos.1+1));
        self.shadow.draw(canvas,(pos.0-1,pos.1+1));
        self.shadow.draw(canvas,(pos.0-1,pos.1-1));
        self.shadow.draw(canvas,(pos.0+1,pos.1-1));
        self.shadow.draw(canvas,(pos.0+1,pos.1));
        self.shadow.draw(canvas,(pos.0-1,pos.1));
        self.shadow.draw(canvas,(pos.0,pos.1+1));
        self.shadow.draw(canvas,(pos.0,pos.1-1));
        self.graphic.draw(canvas,pos);
    }
    
}
struct Emblems<'r> {
    silver_coin: Emblem<'r>,
    gold_coin: Emblem<'r>,
    club: Emblem<'r>,
    diamond: Emblem<'r>,
    heart: Emblem<'r>,
    spade: Emblem<'r>,
}

struct Game<'r,C> {
    chrome : Graphic<Texture<'r>>,
    overlay: Graphic<Texture<'r>>,
    credits_gfx: Graphic<Texture<'r>>,
    bank_gfx: Graphic<Texture<'r>>,
    tile_set: TileSet,
    emblems: Emblems<'r>,
    inserting: Vec<(bool,i32)>,
    vending: Vec<(bool,u32,(f32,f32),f32,f32)>, //pound, delay, position, angle, velocity
    reels: [Reel;3],
    purse: u32,
    bank: u32,
    credits: u32,
    cursor_x: i32,
    cursor_y: i32,
    mdx: i32,
    mdy: i32,
    scrolling_x: i32,
    scrolling_message: Graphic<TileCache<'r,C>>,
    body_parts: Vec<&'static str>,
    posessions: Vec<&'static str>,
    jackpot: JackpotDisplay<'r,C>,
    go_button: GoButton<'r>,
    collect_button : CollectButton<'r>,
    hold_buttons: [HoldButton<'r>;3],
    won_hold: bool 
}
struct JackpotDisplay<'r,C> {
    ticks: u32,
    graphic: Graphic<TileCache<'r,C>>
}
struct GoButton<'r> {
    state: u32,
    graphic_in: Graphic<Texture<'r>>,
    graphic_out: Graphic<Texture<'r>>,
}
impl <'r>GoButton <'r> {
    const HIGHLIGHT_COLOUR: Color = rgba(138,226,52,255);
    const SHADOW_COLOUR:Color = rgba(78,154,6,255);
    const FACE_COLOUR: Color = rgba(115,210,22,255);
    const LIGHT_FACE_COLOUR: Color = Self::FACE_COLOUR; //rgba(165,230,72,255);
    fn new<T>(texture_creator : &'r TextureCreator<T>,tile_set:&TileSet) -> GoButton<'r> {
        let mut graphic_in = Graphic::solid(5,5,Tile{bg:Self::FACE_COLOUR,fg:CHARCOAL,index:0}).textured(texture_creator);
        let mut graphic_out = Graphic::solid(5,5,Tile{bg:Self::LIGHT_FACE_COLOUR,fg:CHARCOAL,index:0}).textured(texture_creator);
        graphic_out[(0,0)] = Tile{index:168,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_in[(0,0)] = Tile{index:168,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_out[(0,4)] = Tile{index:200,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_in[(0,4)] = Tile{index:200,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_out[(4,4)] = Tile{index:202,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_in[(4,4)] = Tile{index:202,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_out[(4,0)] = Tile{index:170,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_in[(4,0)] = Tile{index:170,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        for i in 1..=3 {
            graphic_out[(i,0)] = Tile{index:169,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
            graphic_in[(i,0)] = Tile{index:169,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
            graphic_out[(i,4)] = Tile{index:201,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
            graphic_in[(i,4)] = Tile{index:201,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
            graphic_out[(4,i)] = Tile{index:186,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
            graphic_in[(4,i)] = Tile{index:186,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
            graphic_out[(0,i)] = Tile{index:184,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
            graphic_in[(0,i)] = Tile{index:184,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        }
        graphic_in.draw_text("GO!",tile_set,1,2,WHITE,Self::FACE_COLOUR);
        graphic_out.draw_text("GO!",tile_set,1,2,CHARCOAL,Self::LIGHT_FACE_COLOUR);

        graphic_in.update_texture(tile_set);
        graphic_out.update_texture(tile_set);
        GoButton {
            state:0,
            graphic_in, graphic_out
        }
    }
    fn tick(&mut self, spinning: bool, can_spin: bool) {
        if spinning {
            self.state = 2;
        } else if can_spin {
            self.state = 1;
        } else {
            self.state = 0;
        }
    }
    fn draw<T:RenderTarget>(&self, canvas : &mut Canvas<T>, position:(i32,i32)) {
        match self.state {
            1 => self.graphic_out.draw(canvas,position),
            2 => self.graphic_in.draw(canvas,position),
            _ => {}
        }
    }
}

struct HoldButton<'r> {
    state: u32,
    graphic_in: Graphic<Texture<'r>>,
    graphic_out: Graphic<Texture<'r>>,
}
impl <'r>HoldButton <'r> {
    const HIGHLIGHT_COLOUR: Color = rgba(252,175,62,255);
    const SHADOW_COLOUR:Color = rgba(206,92,0,255);
    const FACE_COLOUR: Color = rgba(245,121,0,255);
    const LIGHT_FACE_COLOUR: Color = Self::FACE_COLOUR; //rgba(165,230,72,255);
    fn new<T>(texture_creator : &'r TextureCreator<T>,tile_set:&TileSet) -> HoldButton<'r> {
        let mut graphic_in = Graphic::solid(6,3,Tile{bg:Self::FACE_COLOUR,fg:CHARCOAL,index:0}).textured(texture_creator);
        let mut graphic_out = Graphic::solid(6,3,Tile{bg:Self::LIGHT_FACE_COLOUR,fg:CHARCOAL,index:0}).textured(texture_creator);
        graphic_out[(0,0)] = Tile{index:168,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_in[(0,0)] = Tile{index:168,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_out[(0,2)] = Tile{index:200,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_in[(0,2)] = Tile{index:200,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_out[(5,2)] = Tile{index:202,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_in[(5,2)] = Tile{index:202,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_out[(5,0)] = Tile{index:170,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_in[(5,0)] = Tile{index:170,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_in[(0,1)] = Tile{index:184,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_in[(5,1)] = Tile{index:186,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_out[(0,1)] = Tile{index:184,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_out[(5,1)] = Tile{index:186,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        for i in 1..=4 {
            graphic_out[(i,0)] = Tile{index:169,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
            graphic_in[(i,0)] = Tile{index:169,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
            graphic_out[(i,2)] = Tile{index:201,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
            graphic_in[(i,2)] = Tile{index:201,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        }
        graphic_in.draw_text("HOLD",tile_set,1,1,WHITE,Self::FACE_COLOUR);
        graphic_out.draw_text("HOLD",tile_set,1,1,CHARCOAL,Self::LIGHT_FACE_COLOUR);
        graphic_in.update_texture(tile_set);
        graphic_out.update_texture(tile_set);
        HoldButton {
            state:0,
            graphic_in, graphic_out
        }
    }
    fn tick(&mut self, holding: bool, can_hold: bool) {
        if holding {
            self.state = 2;
        } else if can_hold {
            self.state = 1;
        } else {
            self.state = 0;
        }
    }
    fn draw<T:RenderTarget>(&self, canvas : &mut Canvas<T>, position:(i32,i32)) {
        match self.state {
            1 => self.graphic_out.draw(canvas,position),
            2 => self.graphic_in.draw(canvas,position),
            _ => {}
        }
    }
}

struct CollectButton<'r> {
    state: u32,
    graphic_in: Graphic<Texture<'r>>,
    graphic_out: Graphic<Texture<'r>>,
}
impl <'r>CollectButton<'r> {
    const HIGHLIGHT_COLOUR: Color = rgba(239,41,41,255);
    const SHADOW_COLOUR:Color = rgba(164,0,0,255);
    const FACE_COLOUR: Color = rgba(204,0,0,255);
    const LIGHT_FACE_COLOUR: Color = Self::FACE_COLOUR; 
    const TEXT_COLOUR : Color = YELLOW;
    fn new<T>(texture_creator : &'r TextureCreator<T>,tile_set:&TileSet) -> CollectButton<'r> {
        let mut graphic_in = Graphic::solid(5,3,Tile{bg:Self::FACE_COLOUR,fg:CHARCOAL,index:0}).textured(texture_creator);
        let mut graphic_out = Graphic::solid(5,3,Tile{bg:Self::LIGHT_FACE_COLOUR,fg:CHARCOAL,index:0}).textured(texture_creator);
        graphic_out[(0,0)] = Tile{index:168,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_in[(0,0)] = Tile{index:168,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_out[(1,0)] = Tile{index:169,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_in[(1,0)] = Tile{index:169,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_out[(2,0)] = Tile{index:169,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_in[(2,0)] = Tile{index:169,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_out[(3,0)] = Tile{index:169,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_in[(3,0)] = Tile{index:169,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_in[(1,2)] = Tile{index:201,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_out[(1,2)] = Tile{index:201,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_in[(2,2)] = Tile{index:201,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_out[(2,2)] = Tile{index:201,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_in[(3,2)] = Tile{index:201,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_out[(3,2)] = Tile{index:201,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_out[(0,2)] = Tile{index:200,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_in[(0,2)] = Tile{index:200,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_out[(4,2)] = Tile{index:202,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_in[(4,2)] = Tile{index:202,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_out[(4,0)] = Tile{index:170,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_in[(4,0)] = Tile{index:170,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_out[(0,1)] = Tile{index:184,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_in[(0,1)] = Tile{index:184,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};

        graphic_out[(1,1)] = Tile{index:176,fg:Self::TEXT_COLOUR,bg:Self::FACE_COLOUR};
        graphic_in[(1,1)] = Tile{index:176,fg:Self::TEXT_COLOUR,bg:Self::FACE_COLOUR};
        graphic_out[(2,1)] = Tile{index:177,fg:Self::TEXT_COLOUR,bg:Self::FACE_COLOUR};
        graphic_in[(2,1)] = Tile{index:177,fg:Self::TEXT_COLOUR,bg:Self::FACE_COLOUR};
        graphic_out[(3,1)] = Tile{index:178,fg:Self::TEXT_COLOUR,bg:Self::FACE_COLOUR};
        graphic_in[(3,1)] = Tile{index:178,fg:Self::TEXT_COLOUR,bg:Self::FACE_COLOUR};
        graphic_in[(4,1)] = Tile{index:186,fg:Self::HIGHLIGHT_COLOUR,bg:TRANSPARENT};
        graphic_out[(4,1)] = Tile{index:186,fg:Self::SHADOW_COLOUR,bg:TRANSPARENT};
        graphic_in.update_texture(tile_set);
        graphic_out.update_texture(tile_set);
        CollectButton {
            state:0,
            graphic_in, graphic_out
        }
    }
    fn tick(&mut self, collecting: bool, can_collect: bool) {
        if collecting {
            self.state = 2;
        } else if can_collect {
            self.state = 1;
        } else {
            self.state = 0;
        }
    }
    fn draw<T:RenderTarget>(&self, canvas : &mut Canvas<T>, position:(i32,i32)) {
        match self.state {
            1 => self.graphic_out.draw(canvas,position),
            2 => self.graphic_in.draw(canvas,position),
            _ => {}
        }
    }
}
impl <'r,C> JackpotDisplay<'r,C> {
    const COLOURS : [Color;16] = [ 
        rgba(85,87,83,255),
        rgba(85,87,83,255),
        rgba(85,87,83,255),
        CRIMSON,
        DARK_RED,
        BROWN,
        AMBER,
        YELLOW,
        DARK_YELLOW,
        PALE_ORANGE,
        BRIGHT_GREEN,
        PURPLE,
        TEAL,
        rgba(85,87,83,255),
        rgba(85,87,83,255),
        rgba(85,87,83,255),
    ];
    fn tick(&mut self, bank: u32,spinning:bool, tile_set: &TileSet) {
        if spinning {
            self.ticks = (self.ticks + 1) % 16;            
            for i in 0..15 {
                self.graphic[((self.ticks + i) % 16, 0)] = Tile {fg: Self::COLOURS[i as usize],bg: TRANSPARENT, index:188};
            }
        } else {
            for i in 0..bank.min(1600) / 100 {
                self.graphic[(i,0)] = Tile {fg:YELLOW,bg:TRANSPARENT,index:188};
            }
            for i in bank.min(1600)/100..16 {
                self.graphic[(i,0)] = Tile {fg:TRANSPARENT,bg:TRANSPARENT,index:188};
            }
        }
        self.graphic.update_texture(tile_set);
    }
    fn draw<T:RenderTarget>(&self, canvas : &mut Canvas<T>, position:(i32,i32)) {
        self.graphic.draw(canvas,position);
    }
}
struct Reel {
    reel_position: f32,
    position: (i32,i32),
    velocity: f32,
    holding: bool,
    highlight: Color,
}
const FULL_SWEEP_COLOR : Color = rgba(252,175,62,255);
const MATCH_COLOR : Color = rgba(213,232,69,255);
#[derive(Eq,PartialEq,Debug,Copy,Clone)]
enum Piece { Penny, Tuppence, Club, Diamond, Heart, Spade, Pound, TwoPound }
impl Piece {
    fn payout(&self) -> u32 {
        match *self {
            Self::Penny => 10,
            Self::Tuppence => 20,
            Self::Club => 30,
            Self::Diamond => 40,
            Self::Heart => 50,
            Self::Spade => 70,
            Self::Pound => 100,
            Self::TwoPound => 200,
        }
    }
}
impl Reel {
    const HEIGHT : i32 = 24 * 8;
    fn new(position: (i32,i32)) -> Reel {
        Reel { holding:false,reel_position: (thread_rng().gen_range(0,8) * 24) as f32, position, velocity: 0.0, highlight: TRANSPARENT }
    }
    fn is_spinning(&self) -> bool {
        !(self.velocity < 0.05)
    }
    fn tick(&mut self) {
        if self.velocity > 0.3 {
            self.reel_position = (self.reel_position + self.velocity) % Self::HEIGHT as f32;
            self.velocity -= 0.05;
        } else {
            if self.reel_position as i32 % 24 != 0 {
                self.reel_position = (self.reel_position + 0.3) % Self::HEIGHT as f32;
            } else {
                self.velocity = 0.0;
            }
        }
        
    }
    fn current_piece(&self) -> Piece {
        match (self.reel_position) as i32 / 24 {
            0 => Piece::Spade,
            1 => Piece::Penny,
            2 => Piece::TwoPound,
            3 => Piece::Diamond,
            4 => Piece::Tuppence,
            5 => Piece::Heart,
            6 => Piece::Pound,
            _ => Piece::Club
        }
    }

    fn draw<T:RenderTarget>(&self, canvas : &mut Canvas<T>, emblems : &Emblems) {
        canvas.set_draw_color(self.highlight);
        let mut p = self.reel_position as i32;
        let (x,y) = (self.position.0, self.position.1 - 24);
        if self.holding {
            canvas.set_draw_color(NEUTRAL_GRAY);
            canvas.fill_rect(Rect::new(x-4,y+4,40,40)).unwrap();
        } else if self.highlight != TRANSPARENT {
            canvas.fill_rect(Rect::new(x-4,y+4,40,40)).unwrap();
        }
        emblems.silver_coin.draw(canvas,(x,y+p));
        p = (p + 24) % Self::HEIGHT;
        emblems.spade.draw(canvas,(x,y+p));
        p = (p + 24) % Self::HEIGHT;
        emblems.club.draw(canvas,(x,y+p));
        p = (p + 24) % Self::HEIGHT;
        emblems.gold_coin.draw(canvas,(x,y+p));
        p = (p + 24) % Self::HEIGHT;
        emblems.heart.draw(canvas,(x,y+p));
        p = (p + 24) % Self::HEIGHT;
        emblems.silver_coin.draw(canvas,(x-2,y+p-2));
        emblems.silver_coin.draw(canvas,(x+2,y+p+2));
        p = (p + 24) % Self::HEIGHT;
        emblems.diamond.draw(canvas,(x,y+p));
        p = (p + 24) % Self::HEIGHT;
        emblems.gold_coin.draw(canvas,(x-2,y+p-2));
        emblems.gold_coin.draw(canvas,(x+2,y+p+2));
    }
}
impl <'r,T: 'r> Game<'r,T> {
    const COLORS : [Color;8] = [BRIGHT_GREEN, PALE_ORANGE, PALE_BROWN, rgba(239,41,41,255), rgba(173,127,168,255), WHITE, rgba(114,159,207,255), YELLOW];
    const MESSAGES : [&'static str;8] = ["This one could be it!", "Spin to win!", "Come on!", "Alea iacta est!", "Spin, spin, spin!","The lucky spin!", "If at first you don't succeed...", "Hit the jackpot!"];
    const MATCH_MESSAGES : [&'static str;8] = ["Triple match!", "You won!", "Got a Triple!", "Well done!", "Luck is on your side","You made it!", "Congratulations!", "Keep going!"];
    fn new(texture_creator : &'r TextureCreator<T>) -> Game<'r,T> {
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
        let mut chrome = Graphic::load_from(Cursor::new(&include_bytes!("../slots_chrome")[..])).unwrap().textured(&texture_creator);
        let mut overlay = Graphic::load_from(Cursor::new(&include_bytes!("../chrome_overlay")[..])).unwrap().textured(&texture_creator);
        chrome.update_texture(&tile_set);
        overlay.update_texture(&tile_set);
        let silver_coin = Emblem::new(276,NEUTRAL_GRAY,rgba(85,87,83,255), &tile_set,texture_creator);
        let gold_coin = Emblem::new(276,YELLOW,DARK_YELLOW,&tile_set,texture_creator);
        let spade = Emblem::new(292,rgba(173,127,168,255),rgba(92,52,102,255),&tile_set,texture_creator);
        let diamond = Emblem::new(288,rgba(114,159,207,255),rgba(8,32,74,255),&tile_set,texture_creator);
        let heart = Emblem::new(290,rgba(239,41,41,255),CRIMSON,&tile_set,texture_creator);
        let club = Emblem::new(294,rgba(85,87,83,255),DARK_CHARCOAL,&tile_set,texture_creator);
        let credits_gfx = Graphic::blank(6,1).textured(texture_creator);
        let bank_gfx = Graphic::blank(6,1).textured(texture_creator);
        let go_button = GoButton::new(texture_creator,&tile_set);
        let collect_button = CollectButton::new(texture_creator,&tile_set);
        let hold_buttons = [HoldButton::new(texture_creator,&tile_set), HoldButton::new(texture_creator,&tile_set),HoldButton::new(texture_creator,&tile_set)];
        let mut body_parts = vec![
            "YOUR LEFT LITTLE FINGER", 
            "YOUR LEFT RING FINGER",
            "YOUR LEFT MIDDLE FINGER",
            "YOUR LEFT INDEX FINGER",
            "YOUR LEFT THUMB",
            "YOUR RIGHT LITTLE FINGER", 
            "YOUR RIGHT RING FINGER",
            "YOUR RIGHT MIDDLE FINGER",
            "YOUR RIGHT INDEX FINGER",
            "YOUR RIGHT THUMB",
            "YOUR LEFT EAR",
            "YOUR RIGHT EAR",
            "YOUR NOSE",
            "YOUR LEFT EYE",
            "YOUR RIGHT EYE",
            "YOUR LEFT BIG TOE",
            "YOUR RIGHT BIG TOE",
            "YOUR LEFT SECOND TOE",
            "YOUR LEFT THIRD TOE",
            "YOUR LEFT FOURTH TOE",
            "YOUR LEFT FIFTH TOE",
            "YOUR RIGHT SECOND TOE",
            "YOUR RIGHT THIRD TOE",
            "YOUR RIGHT FOURTH TOE",
            "YOUR RIGHT FIFTH TOE",
            "YOUR TEETH",
            "YOUR TONGUE",
            "YOUR LEFT KIDNEY",
            "YOUR RIGHT KIDNEY",
            "YOUR LEFT LUNG",
            "YOUR RIGHT LUNG",            
        ];
        let mut posessions = vec![
            "YOUR HOUSE",
            "YOUR CAR",
            "YOUR COMPUTER",
            "YOUR MOBILE PHONE",
            "YOUR JEWELLERY",
            "YOUR FAMILY HEIRLOOM",
            "YOUR WASHING MACHINE",
            "YOUR VACUUM CLEANER",
            "YOUR OLD BOOKS",
            "YOUR GAME CONSOLE",
            "YOUR AIR CONDITIONER",
            "YOUR SOUL",
            "YOUR HOLIDAY SOUVENIRS",
            "YOUR STAMP COLLECTION",
            "YOUR STEREO SYSTEM",
            "YOUR FURNITURE",
            "YOUR WEDDING RING"
        ];
        posessions.shuffle(&mut thread_rng());
        body_parts.shuffle(&mut thread_rng());
        Game {
            chrome, overlay, inserting: Vec::new(), vending: Vec::new(),
            tile_set, emblems: Emblems{ silver_coin,gold_coin,club,diamond,heart,spade },
            reels: [Reel::new((20,88+17+4)), Reel::new((68,88+17+4)), Reel::new((116,88+17+4))],
            purse: 250, credits: 0, bank: 0,
            credits_gfx, bank_gfx,cursor_x: 0,cursor_y:0,mdx:0,mdy:0,
            jackpot: JackpotDisplay { ticks: 0, graphic: Graphic::blank(16,1).tile_cache_textured(texture_creator) },
            go_button, collect_button,hold_buttons, won_hold:false,
            body_parts, posessions, scrolling_x: WIDTH as i32*8, scrolling_message: Graphic::blank(80,1).tile_cache_textured(texture_creator)
        }
    }
    fn draw<C:RenderTarget>(&mut self, canvas : &mut Canvas<C>) {
        if self.is_spinning() {
            canvas.set_draw_color(rgba(250,250,230,255));
        } else {
            canvas.set_draw_color(rgba(250,250,200,255));
        }
        canvas.clear();
        self.reels[0].draw(canvas,&self.emblems);
        self.reels[1].draw(canvas,&self.emblems);
        self.reels[2].draw(canvas,&self.emblems);
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*8,WIDTH*12 + 16,50)).unwrap();
        self.chrome.draw(canvas,(0,17));
        self.credits_gfx.draw_rect(0,0,6,1,Tile {index:51, fg: WHITE, bg : CHARCOAL});
        self.credits_gfx[(0,0)] = Tile {index: 179, fg: WHITE, bg: TRANSPARENT};
        self.credits_gfx.draw_text(& (self.credits/100).to_string(),&self.tile_set,1,0,WHITE,TRANSPARENT);
        let mut x = 0;
        while self.credits_gfx[(x,0)].bg != CHARCOAL { x += 1};
        self.credits_gfx[(x,0)] = Tile {index: 28, fg: WHITE, bg:TRANSPARENT};
        self.credits_gfx.draw_text(& ((self.credits%100)/10).to_string(),&self.tile_set,x+1,0,WHITE,TRANSPARENT);
        self.credits_gfx.draw_text(& (self.credits%10).to_string(),&self.tile_set,x+2,0,WHITE,TRANSPARENT);
        self.credits_gfx.draw_rect(x+3,0,6,1,Tile {index:0, fg: WHITE, bg : CHARCOAL});
        self.credits_gfx.update_texture(&self.tile_set);
        self.credits_gfx.draw(canvas,(92,77));
        self.bank_gfx.draw_rect(0,0,6,1,Tile {index:51, fg: YELLOW, bg : CHARCOAL});
        self.bank_gfx[(0,0)] = Tile {index: 179, fg: YELLOW, bg: TRANSPARENT};
        self.bank_gfx.draw_text(& (self.bank/100).to_string(),&self.tile_set,1,0,YELLOW,TRANSPARENT);
        let mut x = 0;
        while self.bank_gfx[(x,0)].bg != CHARCOAL { x += 1};
        self.bank_gfx[(x,0)] = Tile {index: 28, fg: YELLOW, bg:TRANSPARENT};
        self.bank_gfx.draw_text(& ((self.bank%100) / 10).to_string(),&self.tile_set,x+1,0,YELLOW,TRANSPARENT);
        self.bank_gfx.draw_text(& (self.bank%10).to_string(),&self.tile_set,x+2,0,YELLOW,TRANSPARENT);
        self.bank_gfx.draw_rect(x+3,0,6,1,Tile {index:0, fg: YELLOW, bg : CHARCOAL});
        self.bank_gfx.update_texture(&self.tile_set);
        self.bank_gfx.draw(canvas,(20,77));
        self.jackpot.draw(canvas,(16,24+17));
        self.go_button.draw(canvas,(152,80+17));
        self.collect_button.draw(canvas,(152,120+17));
        self.hold_buttons[0].draw(canvas,(8,120+17));
        self.hold_buttons[1].draw(canvas,(56,120+17));
        self.hold_buttons[2].draw(canvas,(104,120+17));        
        for i in 0..(self.purse%100)/10 {
            self.emblems.silver_coin.draw(canvas,(242,156+17-i as i32*3));
        }
        if self.purse/100 > 0 {
            let dist = (144/(self.purse/100)).min(3);
            for i in 0..self.purse/100 {
                self.emblems.gold_coin.draw(canvas,(208,156+17-i as i32*dist as i32));
            }
        }
        if self.inserting.len() > 0 {
            for (c,r) in &self.inserting {
                if *r >= 0 {
                    let s = if *c { (208,156+17) } else { (242,156+17) };
                    let e = (160,48+17+4);
                    let p = (s.0 + ((e.0 - s.0) * (20 - *r as i32))/20, s.1 + ((e.1 - s.1) * (20 - *r as i32))/20);
                    if *c { self.emblems.gold_coin.draw(canvas,p) } else { self.emblems.silver_coin.draw(canvas,p)}
                } else {
                    let p = (160+r,48+17+4);
                    if *c { self.emblems.gold_coin.draw(canvas,p) } else { self.emblems.silver_coin.draw(canvas,p)}
                }
            }
            self.overlay.draw(canvas,(0,17));
        }
        if self.vending.len() > 0 {
            for (c,d,p,_,_) in &self.vending {
                if *d == 0 {
                    let pos = (p.0 as i32,p.1 as i32);
                    if *c { self.emblems.gold_coin.draw(canvas,pos) } else { self.emblems.silver_coin.draw(canvas,pos) };
                }
            }
        }

    }
    fn vend(&mut self, pound:bool) {
        let x = thread_rng().gen_range(16.0,176.0);
        let y = thread_rng().gen_range(152.0,168.0) + 10.0;
        let a : f32 = (thread_rng().gen_range(0.0,180.0) as f32).to_radians();
        let v = thread_rng().gen_range(1.5,2.5);
        let d = thread_rng().gen_range(0,30);
        self.vending.push((pound,d,(x,y),a,v));
    }
    fn can_collect(&self) -> bool {
        self.vending.len() == 0 && self.bank >= 10
    }
    fn collect(&mut self) {
        if self.can_collect() {
            while self.bank >= 100 {
                self.bank -= 100;
                self.vend(true);
            }
            while self.bank >= 10 {
                self.bank -= 10;
                self.vend(false);
            }
        }
    }
    fn add_to_bank(&mut self) {
        self.bank += 130;
    }
    fn is_spinning(&self) -> bool {
        self.reels[0].is_spinning() || self.reels[1].is_spinning() || self.reels[2].is_spinning()
    }
    fn can_spin(&self) -> bool {
        self.credits >= 10 || self.bank >= 10
    }
    
    fn spin(&mut self) {
        if self.can_spin() && !self.is_spinning() {
            if self.credits >= 10 { self.credits -= 10; }
            else { self.bank -= 10; }
            self.reels[0].highlight = TRANSPARENT;
            self.reels[1].highlight = TRANSPARENT;
            self.reels[2].highlight = TRANSPARENT;
            if !self.reels[0].holding { self.reels[0].velocity = thread_rng().gen_range(5.0,15.0) };
            if !self.reels[1].holding { self.reels[1].velocity = thread_rng().gen_range(5.0,15.0) };
            if !self.reels[2].holding { self.reels[2].velocity = thread_rng().gen_range(5.0,15.0) };
            if self.reels[0].holding && self.reels[1].holding && self.reels[2].holding {
                self.payout();
            } else {
                self.scrolling_message.draw_rect(0,0,80,1,Tile{fg:TRANSPARENT,bg:TRANSPARENT,index:0});
                self.scrolling_message.draw_text(Self::MESSAGES[thread_rng().gen_range(0,8)],&self.tile_set,0,0,Self::COLORS[thread_rng().gen_range(0,8)],TRANSPARENT);
                self.scrolling_message.update_texture(&self.tile_set);
                self.scrolling_x = WIDTH as i32 * 8;
            }
        }
    }
    fn insert_10p(&mut self) {
        if self.purse % 100 >= 10 {
            self.purse -= 10;
            self.inserting.push((false,20));
        }
    }
    fn insert_pound(&mut self) {
        if self.purse / 100 > 0 {
            self.purse -= 100;
            self.inserting.push((true,20));
        }
    }
    fn payout(&mut self) {
        if self.reels[0].current_piece() == self.reels[1].current_piece() && self.reels[1].current_piece() == self.reels[2].current_piece() {
            self.reels[0].highlight = FULL_SWEEP_COLOR;
            self.reels[1].highlight = FULL_SWEEP_COLOR;
            self.reels[2].highlight = FULL_SWEEP_COLOR;
            self.bank += self.reels[0].current_piece().payout();
            self.scrolling_message.draw_rect(0,0,80,1,Tile{fg:TRANSPARENT,bg:TRANSPARENT,index:0});
            self.scrolling_message.draw_text(Self::MATCH_MESSAGES[thread_rng().gen_range(0,8)],&self.tile_set,0,0,Self::COLORS[thread_rng().gen_range(0,8)],TRANSPARENT);
            self.scrolling_message.update_texture(&self.tile_set);
            self.scrolling_x = WIDTH as i32 * 8;
        } else if self.reels[0].current_piece() == self.reels[1].current_piece() {
            self.reels[0].highlight = MATCH_COLOR;
            self.reels[1].highlight = MATCH_COLOR;
            self.bank += self.reels[0].current_piece().payout()/2;

        } else if self.reels[1].current_piece() == self.reels[2].current_piece() {
            self.reels[1].highlight = MATCH_COLOR;
            self.reels[2].highlight = MATCH_COLOR;
            self.bank += self.reels[1].current_piece().payout()/2;

        } else if self.reels[0].current_piece() == self.reels[2].current_piece() {
            self.reels[0].highlight = MATCH_COLOR;
            self.reels[2].highlight = MATCH_COLOR;
            self.bank += self.reels[0].current_piece().payout()/2;
        }
        self.check_jackpot();
        self.reels[0].holding = false;
        self.reels[1].holding = false;
        self.reels[2].holding = false;
        self.won_hold = thread_rng().gen_range(0,4) == 0;
    }
    fn can_hold(&self) -> bool {
        self.won_hold && !self.is_spinning()
    }
    fn hold(&mut self, index:usize) {
        if self.can_hold() {
            self.reels[index].holding  = !self.reels[index].holding;
        }
    }
    fn pawn_shop(&mut self) {
        self.scrolling_x = WIDTH as i32*8;
        if let Some(x) = self.posessions.pop()  {
            self.scrolling_message.draw_rect(0,0,80,1,Tile{fg:TRANSPARENT,bg:TRANSPARENT,index:0});
            self.scrolling_message.draw_text("You manage to sell ",&self.tile_set,0,0,NEUTRAL_GRAY,TRANSPARENT);
            self.scrolling_message.draw_text(x,&self.tile_set,19,0,WHITE,TRANSPARENT);
            self.scrolling_message.draw_text(" for ",&self.tile_set,20 -1 + x.len() as u32,0,NEUTRAL_GRAY,TRANSPARENT);
            self.scrolling_message[(25 -1 + x.len() as u32,0)] = Tile {index:179, fg: YELLOW, bg: TRANSPARENT};
            let amount = thread_rng().gen_range(1,16) * 10;
            let pounds = (amount / 100).to_string();
            self.scrolling_message.draw_text(&pounds,&self.tile_set,26 -1 + x.len() as u32,0,YELLOW,TRANSPARENT);
            self.scrolling_message[(26 -1 + x.len() as u32 +pounds.len() as u32,0)] = Tile {index:28, fg: YELLOW, bg: TRANSPARENT};
            self.scrolling_message.draw_text(&((amount%100)/10).to_string(),&self.tile_set,26 + x.len() as u32 +pounds.len() as u32,0,YELLOW,TRANSPARENT);
            self.scrolling_message.draw_text(&(amount%10).to_string(),&self.tile_set,27 + x.len() as u32 +pounds.len() as u32,0,YELLOW,TRANSPARENT);
            self.purse += amount;
            self.scrolling_message.update_texture(&self.tile_set);
        } else {
            self.scrolling_message.draw_rect(0,0,80,1,Tile{fg:TRANSPARENT,bg:TRANSPARENT,index:0});
            self.scrolling_message.draw_text("No more posessions to sell :(",&self.tile_set,0,0,NEUTRAL_GRAY,TRANSPARENT);
            self.scrolling_message.update_texture(&self.tile_set);
        }
    }
    fn loan_shark(&mut self) {
        self.scrolling_x = WIDTH as i32*8;
        if let Some(x) = self.body_parts.pop()  {
            self.scrolling_message.draw_rect(0,0,80,1,Tile{fg:TRANSPARENT,bg:TRANSPARENT,index:0});
            self.scrolling_message.draw_text("You have obtained a loan of ",&self.tile_set,0,0,NEUTRAL_GRAY,TRANSPARENT);
            self.scrolling_message[(28,0)] = Tile {index:179, fg: YELLOW, bg: TRANSPARENT};
            let amount = thread_rng().gen_range(1,26) * 10;
            let pounds = (amount / 100).to_string();
            self.scrolling_message.draw_text(&pounds,&self.tile_set,29,0,YELLOW,TRANSPARENT);
            self.scrolling_message[(29+pounds.len() as u32,0)] = Tile {index:28, fg: YELLOW, bg: TRANSPARENT};
            self.scrolling_message.draw_text(&((amount%100)/10).to_string(),&self.tile_set,30+pounds.len() as u32,0,YELLOW,TRANSPARENT);
            self.scrolling_message.draw_text(&(amount%10).to_string(),&self.tile_set,31+pounds.len() as u32,0,YELLOW,TRANSPARENT);
            self.scrolling_message.draw_text(" using ",&self.tile_set,32 + pounds.len() as u32,0,NEUTRAL_GRAY,TRANSPARENT);
            self.scrolling_message.draw_text(x,&self.tile_set,39 + pounds.len() as u32,0,WHITE,TRANSPARENT);
            self.scrolling_message.draw_text(" as collateral",&self.tile_set,39 + pounds.len() as u32 + x.len() as u32,0,NEUTRAL_GRAY,TRANSPARENT);
            self.purse += amount;
            self.scrolling_message.update_texture(&self.tile_set);
        } else {
            self.scrolling_message.draw_rect(0,0,80,1,Tile{fg:TRANSPARENT,bg:TRANSPARENT,index:0});
            self.scrolling_message.draw_text("No more body parts to put as collateral :(",&self.tile_set,0,0,NEUTRAL_GRAY,TRANSPARENT);
            self.scrolling_message.update_texture(&self.tile_set);
        }
    }
    fn check_jackpot(&mut self) {
        if self.bank >= 1600 {
            self.bank = 0;
            self.scrolling_message.draw_rect(0,0,80,1,Tile{fg:TRANSPARENT,bg:TRANSPARENT,index:0});
            self.scrolling_message.draw_text("YOU HIT THE JACKPOT!!!!!!!!",&self.tile_set,0,0,YELLOW,TRANSPARENT);
            self.scrolling_message.update_texture(&self.tile_set);
            self.scrolling_x = WIDTH as i32 * 8;
            for _ in 0..50 { 
                self.vend(true)
            }
        }
    }
    fn collides_pennies(x : i32,y : i32) -> bool {
        x > 241 && y > 10
    }
    fn collides_pounds(x:i32,y:i32) -> bool {
        x > 200 && y > 10 && !Self::collides_pennies(x, y)
    }
    fn collides_go(x:i32,y:i32)-> bool {
        if x >= 160 && x <= 186 {
            if y >= 88 && y <= 116 {
                return true
            }
        }
        false
    }
    fn collides_collect(x:i32,y:i32)-> bool {
        if x >= 160 && x <= 186 {
            if y >= 128 && y <= 140 {
                return true
            }
        }
        false
    }
    fn collides_hold(x:i32,y:i32)-> Option<usize> {
        if !(y >= 128 && y <= 140) { return None };
        if x >= 14 && x <= 50 {
                return Some(0);
        }
        if x >= 62 && x <= 98 {
                return Some(1);
        }
        if x >= 110 && x <= 146 {
                return Some(2);
        }
        return None;
    }
    fn mouse_up(&mut self) {
        if Self::collides_pennies(self.cursor_x,self.cursor_y) && Self::collides_pennies(self.mdx, self.mdy) {
            self.insert_10p();
        }
        if Self::collides_pounds(self.cursor_x,self.cursor_y) && Self::collides_pounds(self.mdx, self.mdy) {
            self.insert_pound();
        }
        if Self::collides_go(self.cursor_x, self.cursor_y) && Self::collides_go(self.mdx, self.mdy) {
            self.spin();
        }
        if Self::collides_collect(self.cursor_x, self.cursor_y) && Self::collides_collect(self.mdx, self.mdy) {
            self.collect();
        }
        if let Some(i) = Self::collides_hold(self.cursor_x, self.cursor_y) {
            if Some(i) == Self::collides_hold(self.mdx, self.mdy) {
                self.hold(i);
            }
        }
    }
    fn mouse_down(&mut self) {
        self.mdx = self.cursor_x;
        self.mdy = self.cursor_y;
    }
    fn mouse_moved(&mut self, x : i32, y : i32) {
        self.cursor_x = x;
        self.cursor_y = y;
    }
    
    fn tick(&mut self) {
        let x = self.is_spinning();
        self.reels[0].tick();
        self.reels[1].tick();
        self.reels[2].tick();
        for (c,r) in self.inserting.iter_mut() {
            *r = *r - 1;            
            if *r == -10 {
                self.credits += if *c { 100 } else { 10 }
            }
        }
        self.inserting = self.inserting.drain(..).filter(|(_,r)| *r > -10).collect();
        for (c,d,x,a,v) in self.vending.iter_mut() {
            if *d == 0 {
                x.0 += f32::cos(*a) * *v;
                x.1 += f32::sin(*a) * *v;
                if x.0 > (WIDTH * 8) as f32 || x.1 > (HEIGHT * 8 + 17) as f32 || x.0 < -24.0 || x.1 < -24.0 {
                    *v = 0.0;
                    self.purse += if *c { 100 } else { 10 }
                }
            } else {
                *d -= 1;
            }
        }
        if self.scrolling_x > -80 * 8 {
            self.scrolling_x -= 2;
        }
        self.vending = self.vending.drain(..).filter(|(_,_,_,_,v)| *v >= f32::EPSILON).collect();
        self.jackpot.tick(self.bank,self.is_spinning(),&self.tile_set);
        self.go_button.tick(self.is_spinning(),self.can_spin());
        self.collect_button.tick(self.vending.len() >0,self.can_collect());
        self.hold_buttons[0].tick(self.reels[0].holding,self.can_hold());
        self.hold_buttons[1].tick(self.reels[1].holding,self.can_hold());
        self.hold_buttons[2].tick(self.reels[2].holding,self.can_hold());
        if x && !self.is_spinning() {
            // we stopped spinning
            self.payout();
        }
    }
}
const WIDTH : u32 =34;
const HEIGHT : u32 =22;
fn main() {
    let sdl_context = sdl2::init().unwrap();
    sdl2::hint::set_with_priority("SDL_HINT_RENDER_SCALE_QUALITY", "0",&sdl2::hint::Hint::Override);
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.text_input().start();
    let window = video_subsystem.window("jackpot", WIDTH*8, HEIGHT*8 + 16 + 17)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(WIDTH*8, HEIGHT*8 + 16+ 17).unwrap();
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut game = Game::new(&texture_creator);
    let mut rate_limiter = FPSManager::new();
    rate_limiter.set_framerate(40).unwrap();
    let mut micro_mode = false;
    let mut menu = MenuBar::new(WIDTH*8)
                    .add(Menu::new("GAME",96,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Restart", 15,Keycode::N,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(80, &texture_creator,&game.tile_set))
                            .add(MenuItem::new("Quit", 363, Keycode::F12,&texture_creator,&game.tile_set)))
                    .add(Menu::new("FINANCES",112,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Loan Shark", 352, Keycode::F1,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Pawn Shop", 353, Keycode::F2,&texture_creator,&game.tile_set)))
                    .add(Menu::new("VIEW",112,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Micro-mode",360, Keycode::F9, &texture_creator, &game.tile_set)));
    let mut cx = 0;
    let mut cy = 0;

    loop {
        game.tick();
        game.draw(&mut canvas);
        canvas.set_draw_color(BLACK);
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*8 + 17,WIDTH*12 + 16,17)).unwrap();
        canvas.set_draw_color(CHARCOAL);
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*8 + 18,WIDTH*12 + 16,16)).unwrap();
        game.scrolling_message.draw(&mut canvas,(game.scrolling_x, HEIGHT as i32*8 + 21));
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
                Event::MouseButtonUp {y,..} if y > 17 => {
                    game.mouse_up()
                }
                Event::MouseButtonDown {y,..} if y > 17 => {
                    game.mouse_down()
                }
                Event::KeyDown{ keycode: Some(Keycode::Z),..} => {
                    game.insert_10p()
                }
                Event::KeyDown{ keycode: Some(Keycode::X),..} => {
                    game.insert_pound()
                }
                Event::KeyDown{ keycode: Some(Keycode::C),..} => {
                    game.collect()
                }
                Event::KeyDown{ keycode: Some(Keycode::F10),..} => {
                    game.add_to_bank()
                }
                Event::KeyDown{ keycode: Some(Keycode::N),..} => {
                    game = Game::new(&texture_creator);
                }
                Event::KeyDown{ keycode: Some(Keycode::Num1),..} => {
                    game.hold(0)
                }
                Event::KeyDown{ keycode: Some(Keycode::Num2),..} => {
                    game.hold(1)
                }
                Event::KeyDown{ keycode: Some(Keycode::Num3),..} => {
                    game.hold(2)
                }

                Event::KeyDown{ keycode: Some(Keycode::Space),..} => {
                    game.spin()
                }
                Event::KeyDown{ keycode: Some(Keycode::F1),..} => {
                    game.loan_shark()
                }
                Event::KeyDown{ keycode: Some(Keycode::F2),..} => {
                    game.pawn_shop()
                }
                Event::KeyDown { keycode: Some(Keycode::F9), ..} => {
                    if micro_mode {
                        micro_mode = false;
                        canvas.window_mut().set_size(WIDTH*8, HEIGHT*8+16+17).unwrap_or_default();
                    } else {
                        micro_mode = true;
                        canvas.window_mut().set_size(WIDTH*4, (HEIGHT*8+16+17)/2).unwrap_or_default();
                    }
                },
                Event::MouseMotion { x,y,..} => {
                    let ax = x;
                    let ay = y - 17;
                    if (ax,ay) != (cx,cy) {
                        cx = ax;
                        cy = ay;
                        game.mouse_moved(cx,cy);
                    }
                }
                _ => {},
            }
        }
        rate_limiter.delay();
    }
}


