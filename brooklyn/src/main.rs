
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
use utils::framerate::FPSManager;
use sdl2::rect::Rect;
use std::io::Cursor;
use rand::{thread_rng,Rng};
use utils::color::{*};



struct GraphicsSet<T> {
    tile_set: TileSet,
    pipe_pieces: [[Graphic<T>;3];32],
    fill_cells:[[Graphic<T>;5];4],
    covering_cells:[[Graphic<T>;3];16],
    progress_cell:[Graphic<T>;3],
    bonus_cursors: [[Graphic<T>;2];4],
    cursor: Graphic<T>,    
    cursor_outline: Graphic<T>,
    direction_marker: [Graphic<T>;2],
    error_marker: Graphic<T>,
    error_marker_shadow: Graphic<T>,
    error_marker_highlight: Graphic<T>,
}
const UI_DARK : Color = rgba(85,87,83,255);
const UI_LIGHT : Color = rgba(176,179,172,255);


impl <'r> GraphicsSet<Texture<'r>> {
    fn cursor<T>(texture_creator: &'r TextureCreator<T>, color: Color, tile_set: &TileSet) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(5,5).textured(texture_creator);
        ret[(0,0)] = Tile { index: 137, fg: TRANSPARENT, bg: color };
        ret[(4,0)] = Tile { index: 136, fg: TRANSPARENT, bg: color };
        ret[(4,4)] = Tile { index: 118, bg: TRANSPARENT, fg: color };
        ret[(0,4)] = Tile { index: 119, bg: TRANSPARENT, fg: color };
        ret.update_texture(tile_set);
        ret
    }
    fn covering_cell<T>(tile_set: &TileSet, amount : u8, cset: [Color;3], texture_creator: &'r TextureCreator<T>) -> [Graphic<Texture<'r>>;3] {
        let mut ret1 = Graphic::blank(4,4).textured(texture_creator);
        let mut ret2 = Graphic::blank(4,4).textured(texture_creator);
        let mut ret3 = Graphic::blank(4,4).textured(texture_creator);
        let mut v = amount;
        if v == 0 { return [ret1,ret2,ret3] };
        for y in (0..4).rev() {
            for x in 0..4 {
                ret1[(x,y)] = Tile{index:1,bg:TRANSPARENT,fg: cset[0]};
                ret2[(x,y)] = Tile{index:1,bg:TRANSPARENT,fg: cset[1]};
                ret3[(x,y)] = Tile{index:1,bg:TRANSPARENT,fg: cset[2]};
                v -= 1;
                if v == 0 {
                    ret1.update_texture(tile_set);
                    ret2.update_texture(tile_set);
                    ret3.update_texture(tile_set);
                    return [ret1,ret2,ret3]
                }
            }
        }
        [ret1,ret2,ret3]
    }
    fn fill_cells<T>(tile_set:&TileSet, texture_creator: &'r TextureCreator<T>) -> [[Graphic<Texture<'r>>;5];4] {
        let mut fore = PALE_BLUE;
        fore.a = 200;
        let back = TRANSPARENT;
        let mut x = [
            [ Graphic::solid(1, 1, Tile { index: 0, bg: back, fg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 169, bg: back, fg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 121, bg: back, fg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 201, fg: back, bg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 0, fg: back, bg: fore}).textured(texture_creator),
            ],
            [ Graphic::solid(1, 1, Tile { index: 0, bg: back, fg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 201, bg: back, fg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 121, fg: back, bg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 169, fg: back, bg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 0, fg: back, bg: fore}).textured(texture_creator),
            ],
            [ Graphic::solid(1, 1, Tile { index: 0, bg: back, fg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 186, bg: back, fg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 120, fg: back, bg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 184, fg: back, bg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 0, fg: back, bg: fore}).textured(texture_creator),
            ],
            [ Graphic::solid(1, 1, Tile { index: 0, bg: back, fg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 184, bg: back, fg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 120, bg: back, fg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 186, fg: back, bg: fore}).textured(texture_creator),
              Graphic::solid(1, 1, Tile { index: 0, fg: back, bg: fore}).textured(texture_creator),
            ]
        ];
        for a in x.iter_mut() { for b in a { b.update_texture(&tile_set); }};
        x
    }
    fn pipe_pieces<T,Y>(pipe_type: PipeType, colour_set: [Color;3], source_gfx: &Graphic<Y>, tile_set: &TileSet, texture_creator: &'r TextureCreator<T>) -> [Graphic<Texture<'r>>;3] {        
        [ Self::pipe_piece(pipe_type, false, source_gfx, colour_set[0], tile_set, texture_creator),
          Self::pipe_piece(pipe_type, true, source_gfx, colour_set[1], tile_set, texture_creator),
          Self::pipe_piece(pipe_type, false, source_gfx, colour_set[2], tile_set, texture_creator),
        ]
    }
    fn direction_marker<T,Y>(source_gfx: &Graphic<Y>, x: u32, tile_set: &TileSet, texture_creator: &'r TextureCreator<T>) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(4,4).textured(texture_creator);
        ret.copy_tiles_from(source_gfx, x, 8, 4, 4, 0, 0);
        ret.update_texture(tile_set);
        ret
    }
    fn bonus_cursor<T,Y>(source_gfx: &Graphic<Y>, x: u32, tile_set: &TileSet, texture_creator: &'r TextureCreator<T>) -> [Graphic<Texture<'r>>;2] {
        let mut ret = Graphic::blank(4,4).textured(texture_creator);
        let mut ret2 = Graphic::blank(4,4).textured(texture_creator);
        ret.copy_tiles_from(source_gfx, x, 16, 4, 4, 0, 0);
        ret2.copy_tiles_from(source_gfx, x, 16, 4, 4, 0, 0);
        for x in 0..4 {
            for y in 0..4 {
                if ret[(x,y)].fg.a != 0 {
                    ret[(x,y)].fg = WHITE;
                    ret2[(x,y)].fg = BLACK;
                } 
                if ret[(x,y)].bg.a != 0 {
                    ret[(x,y)].bg = WHITE;
                    ret2[(x,y)].bg = BLACK;
                }
            }
        }
        ret.update_texture(tile_set);
        ret2.update_texture(tile_set);
        [ret,ret2]
    }
    fn error_marker<T,Y>(source_gfx: &Graphic<Y>, fg: Color, tile_set: &TileSet, texture_creator: &'r TextureCreator<T>) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(4,4).textured(texture_creator);
        ret.copy_tiles_from(source_gfx, 0, 16, 4, 4, 0, 0);
        for x in 0..4 {
            for y in 0..4 {
                if ret[(x,y)].fg.a != 0 {
                    ret[(x,y)].fg = fg;
                } 
                if ret[(x,y)].bg.a != 0 {
                    ret[(x,y)].bg = fg;                
                }
            }
        }
        ret.update_texture(tile_set);
        ret
    }
    fn pipe_piece<T,Y>(pipe_type: PipeType, show_symbols:bool, source_gfx: &Graphic<Y>, fg: Color, tile_set: &TileSet, texture_creator: &'r TextureCreator<T>) -> Graphic<Texture<'r>> {
        let mut ret = Graphic::blank(4,4).textured(texture_creator);
        let point = match pipe_type {
            PipeType::UR => (0,0),
            PipeType::UL => (12,0),
            PipeType::DL => (8,4),
            PipeType::DR => (4,4),
            PipeType::X => (4,0),
            PipeType::H => (8,0),
            PipeType::V => (0,4),
            PipeType::BL => (4,8),
            PipeType::RH => (0,8),
            PipeType::RV => (12,4),
            PipeType::S2  => (8,12),
            PipeType::UR1 => (24,4),
            PipeType::UR2 => (24,0),
            PipeType::UL1 => (36,4),
            PipeType::UL2 => (36,0),
            PipeType::DR1 => (28,4),
            PipeType::DR2 => (28,0),
            PipeType::DL1 => (32,0),
            PipeType::DL2 => (32,4),
            PipeType::V1 => (20,0),
            PipeType::V2 => (20,4),
            PipeType::H1 => (16,4),
            PipeType::H2 => (16,0),
            PipeType::S1  => (12,12),
            PipeType::S3  => (16,12),
            PipeType::S4  => (20,12),
            PipeType::BH  => (28,8),
            PipeType::BV  => (24,8),
            PipeType::E1  => (8,8),
            PipeType::E2  => (12,8),
            PipeType::E3  => (0,12),
            PipeType::E4  => (4,12),
        };
        ret.copy_tiles_from(source_gfx, point.0, point.1, 4, 4, 0, 0);
        for x in 0..4 {
            for y in 0..4 {
                if ret[(x,y)].fg.a != 0 {
                    ret[(x,y)].fg = fg;
                } 
                if ret[(x,y)].bg.a != 0 {
                    ret[(x,y)].bg = fg;
                    ret[(x,y)].fg = if show_symbols {BLACK} else {fg};
                }
            }
        }
        ret.update_texture(tile_set);
        ret
    }
    fn new<T>(texture_creator: &'r TextureCreator<T>) -> GraphicsSet<Texture<'r>> {
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
        let pipes = Graphic::load_from(Cursor::new(&include_bytes!("../pipes")[..])).unwrap();

        let cursor = Self::cursor(texture_creator,WHITE, &tile_set);
        let cursor_outline = Self::cursor(texture_creator,CHARCOAL, &tile_set);
        let csets = [ 
            [ 
                rgba(193,125,17,255),
                rgba(169,109,0,255),
                rgba(143,89,2,255),
            ],[
                rgba(239,41,41,255),
                rgba(204,0,0,255),
                rgba(164,0,0,255),
            ],[
                rgba(114,159,207,255),
                rgba(52,101,164,255),
                rgba(32,74,175,255),
            ],[
                rgba(173,127,168,255),
                rgba(117,80,123,255),
                rgba(92,52,102,255),
            ],[
                rgba(252,233,79,255),
                rgba(237,212,0,255),
                rgba(196,160,0,255),
            ],[
                rgba(128,226,52,255),
                rgba(115,210,22,255),
                rgba(78,154,6,255)
            ],[
                rgba(108,206,203,255),
                rgba(78,165,177,255),
                rgba(33,140,141,255),
            ],[
                rgba(252,175,62,255),
                rgba(245,121,0,255),
                rgba(206,92,0,255),
            ],[
                rgba(211,215,207,255),
                rgba(186,189,182,255),
                rgba(156,159,152,255)
            ],[
                rgba(238,238,238,255),
                rgba(211,215,207,255),
                rgba(186,189,182,255),
            ],[
                rgba(115,120,128,255),
                rgba(85,87,83,255),
                rgba(46,52,54,255),
            ]
        ];
        let covering_cells = [
            Self::covering_cell(&tile_set,1,csets[10],texture_creator),
            Self::covering_cell(&tile_set,2,csets[10],texture_creator),
            Self::covering_cell(&tile_set,3,csets[10],texture_creator),
            Self::covering_cell(&tile_set,4,csets[10],texture_creator),
            Self::covering_cell(&tile_set,5,csets[10],texture_creator),
            Self::covering_cell(&tile_set,6,csets[10],texture_creator),
            Self::covering_cell(&tile_set,7,csets[10],texture_creator),
            Self::covering_cell(&tile_set,8,csets[10],texture_creator),
            Self::covering_cell(&tile_set,9,csets[10],texture_creator),
            Self::covering_cell(&tile_set,10,csets[10],texture_creator),
            Self::covering_cell(&tile_set,11,csets[10],texture_creator),
            Self::covering_cell(&tile_set,12,csets[10],texture_creator),
            Self::covering_cell(&tile_set,13,csets[10],texture_creator),
            Self::covering_cell(&tile_set,14,csets[10],texture_creator),
            Self::covering_cell(&tile_set,15,csets[10],texture_creator),
            Self::covering_cell(&tile_set,16,csets[10],texture_creator),
        ];
        let pipe_pieces = [
            Self::pipe_pieces(PipeType::BL,csets[8],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::UR,csets[2],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::DR,csets[7],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::UL,csets[1],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::DL,csets[4],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::V,csets[3],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::H,csets[6],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::X,csets[9],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::RV,csets[0],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::RH,csets[5],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::S2,csets[8],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::UR1,csets[2],&pipes,&tile_set,texture_creator),// => (24,4),
            Self::pipe_pieces(PipeType::UR2,csets[2],&pipes,&tile_set,texture_creator),// => (24,0),
            Self::pipe_pieces(PipeType::UL1,csets[1],&pipes,&tile_set,texture_creator),// => (36,4),
            Self::pipe_pieces(PipeType::UL2,csets[1],&pipes,&tile_set,texture_creator),// => (36,0),
            Self::pipe_pieces(PipeType::DR1,csets[7],&pipes,&tile_set,texture_creator),// => (28,4),
            Self::pipe_pieces(PipeType::DR2,csets[7],&pipes,&tile_set,texture_creator),// => (28,0),
            Self::pipe_pieces(PipeType::DL1,csets[4],&pipes,&tile_set,texture_creator),// => (32,0),
            Self::pipe_pieces(PipeType::DL2,csets[4],&pipes,&tile_set,texture_creator),// => (32,4),
            Self::pipe_pieces(PipeType::V1 ,csets[3],&pipes,&tile_set,texture_creator),//=> (20,0),
            Self::pipe_pieces(PipeType::V2 ,csets[3],&pipes,&tile_set,texture_creator),//=> (20,4),
            Self::pipe_pieces(PipeType::H1 ,csets[6],&pipes,&tile_set,texture_creator),//=> (16,0),
            Self::pipe_pieces(PipeType::H2 ,csets[6],&pipes,&tile_set,texture_creator),//=> (16,4),
            Self::pipe_pieces(PipeType::S1,csets[8],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::S3,csets[8],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::S4,csets[8],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::BH,csets[8],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::BV,csets[8],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::E1,csets[8],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::E2,csets[8],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::E3,csets[8],&pipes,&tile_set,texture_creator),
            Self::pipe_pieces(PipeType::E4,csets[8],&pipes,&tile_set,texture_creator),
        ];
        let fills = Self::fill_cells(&tile_set,texture_creator);
        let error_marker = Self::error_marker(&pipes, DARK_RED, &tile_set, texture_creator);
        let error_marker_highlight = Self::error_marker(&pipes, WHITE, &tile_set, texture_creator);
        let error_marker_shadow = Self::error_marker(&pipes, BLACK, &tile_set, texture_creator);
        let direction_marker = [Self::direction_marker(&pipes,16,&tile_set,texture_creator),Self::direction_marker(&pipes,20,&tile_set,texture_creator)];
        let progress_cell = Self::covering_cell(&tile_set,1,csets[9],texture_creator);
        let bonus_cursors = [
            Self::bonus_cursor(&pipes,8,&tile_set,texture_creator),
            Self::bonus_cursor(&pipes,16,&tile_set,texture_creator),
            Self::bonus_cursor(&pipes,12,&tile_set,texture_creator),
            Self::bonus_cursor(&pipes,20,&tile_set,texture_creator),
        ];
        GraphicsSet {
            tile_set: tile_set,cursor, cursor_outline, error_marker, error_marker_highlight, error_marker_shadow,
            pipe_pieces, fill_cells:fills, direction_marker,covering_cells,progress_cell,bonus_cursors
        }
    }

}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PipeType {
    UR,UL, DR, DL, X, V, H, RV, RH, BL, S2,
    UR1,UR2,UL1,UL2,DR1,DR2,DL1,DL2,V1,V2,H1,H2,
    S1,S3,S4,BH,BV,E1,E2,E3,E4
}
impl PipeType {
    fn capacity(&self) -> u32 { 
        let r = if *self == PipeType::RV || *self == PipeType::RH {
            8
        } else if *self == PipeType::S1 || *self == PipeType::S2 || *self == PipeType::S3 || *self == PipeType::S4 {
            16
        } else {
            4
        };
        r * 4 + 2 
    }
    fn output_for(&self, d: Direction ) -> Direction {
        match self {
            PipeType::UR if d == Direction::S => Direction::E,
            PipeType::UR1 if d == Direction::S => Direction::E,
            PipeType::UR if d == Direction::E => Direction::S,
            PipeType::UR2 if d == Direction::E => Direction::S,
            PipeType::UL if d == Direction::S => Direction::W,
            PipeType::UL1 if d == Direction::S => Direction::W,
            PipeType::UL if d == Direction::W => Direction::S,
            PipeType::UL2 if d == Direction::W => Direction::S,
            PipeType::DR if d == Direction::N => Direction::E,
            PipeType::DR1 if d == Direction::N => Direction::E,
            PipeType::DR if d == Direction::E => Direction::N,
            PipeType::DR2 if d == Direction::E => Direction::N,
            PipeType::DL if d == Direction::N => Direction::W,
            PipeType::DL1 if d == Direction::N => Direction::W,
            PipeType::DL if d == Direction::W => Direction::N,
            PipeType::DL2 if d == Direction::W => Direction::N,
            PipeType::X  if d == Direction::W => Direction::E,
            PipeType::X  if d == Direction::E => Direction::W,
            PipeType::X  if d == Direction::N => Direction::S,
            PipeType::X  if d == Direction::S => Direction::N,
            PipeType::H  if d == Direction::W => Direction::E,
            PipeType::BH  if d == Direction::W => Direction::E,
            PipeType::H1  if d == Direction::W => Direction::E,
            PipeType::H  if d == Direction::E => Direction::W,
            PipeType::BH  if d == Direction::E => Direction::W,
            PipeType::H2  if d == Direction::E => Direction::W,
            PipeType::V  if d == Direction::N => Direction::S,
            PipeType::BV  if d == Direction::N => Direction::S,
            PipeType::V1  if d == Direction::N => Direction::S,
            PipeType::V  if d == Direction::S => Direction::N,
            PipeType::BV  if d == Direction::S => Direction::N,
            PipeType::V2  if d == Direction::S => Direction::N,
            PipeType::RH  if d == Direction::W => Direction::E,
            PipeType::RH  if d == Direction::E => Direction::W,
            PipeType::RV  if d == Direction::N => Direction::S,
            PipeType::RV  if d == Direction::S => Direction::N,
            PipeType::BH  if d == Direction::W => Direction::E,
            PipeType::BH  if d == Direction::E => Direction::W,
            PipeType::BV  if d == Direction::N => Direction::S,
            PipeType::BV  if d == Direction::S => Direction::N,
            PipeType::S1 => Direction::N,
            PipeType::S2 => Direction::S,
            PipeType::S3 => Direction::E,
            PipeType::S4 => Direction::W,
            _ => Direction::N,
        }
    }
    fn accepts(&self, d : Direction) -> bool {
        match self {
            PipeType::UR => d == Direction::S || d == Direction::E,
            PipeType::UR2 => d == Direction::E,
            PipeType::UR1 => d == Direction::S,
            PipeType::UL => d == Direction::S || d == Direction::W,
            PipeType::UL2 => d == Direction::W,
            PipeType::UL1 => d == Direction::S,
            PipeType::DL => d == Direction::N || d == Direction::W,
            PipeType::DL2 => d == Direction::W,
            PipeType::DL1 => d == Direction::N,
            PipeType::DR => d == Direction::N || d == Direction::E,
            PipeType::DR2 => d == Direction::E,
            PipeType::DR1 => d == Direction::N,
            PipeType::X => true,
            PipeType::H => d == Direction::E || d == Direction::W,
            PipeType::BH => d == Direction::E || d == Direction::W,
            PipeType::H1 => d == Direction::W,
            PipeType::H2 => d == Direction::E,
            PipeType::V => d == Direction::N || d == Direction::S,
            PipeType::BV => d == Direction::N || d == Direction::S,
            PipeType::V1 => d == Direction::N,
            PipeType::V2 => d == Direction::S,
            PipeType::RV => d == Direction::N || d == Direction::S,
            PipeType::RH => d == Direction::E || d == Direction::W,
            PipeType::BL => false,
            PipeType::E1 => d == Direction::N,
            PipeType::E2 => d == Direction::S,
            PipeType::E3 => d == Direction::E,
            PipeType::E4 => d == Direction::W,
            _ => false,
        }
    }
    fn flows(&self, d : Direction) -> [[Option<(usize,Direction)>;4];4] {
        use Direction::*;
        match d.opposite() {
            _ if *self == PipeType::E4
              => [ [None, None, None, None],
                   [Some((0*4,E)), Some((1*4,E)),None,None],
                   [Some((0*4,E)), Some((1*4,E)),None,None],
                   [None, None, None,None],
                 ],
            _ if *self == PipeType::E3
              => [ [None, None, None, None],
                   [None, None,Some((1*4,W)), Some((0*4,W))],
                   [None, None,Some((1*4,W)), Some((0*4,W))],
                   [None, None, None,None],
                 ],
            _ if *self == PipeType::E1
              => [ [None, Some((0*4,S)), Some((0*4,S)),None],
                   [None, Some((1*4,S)), Some((1*4,S)),None],
                   [None, None, None, None],
                   [None, None, None, None],
                 ],
            _ if *self == PipeType::E2
              => [ [None, None, None, None],
                   [None, None, None, None],
                   [None, Some((1*4,N)), Some((1*4,N)),None],
                   [None, Some((0*4,N)), Some((0*4,N)),None],
                 ],
            _ if *self == PipeType::S4
              => [ [None, None, None, None],
                   [Some((3*4,W)), Some((2*4,W)),None,None],
                   [Some((3*4,W)), Some((2*4,W)),None,None],
                   [None, None, None,None],
                 ],
            _ if *self == PipeType::S3
              => [ [None, None, None, None],
                   [None, None,Some((2*4,E)), Some((3*4,E))],
                   [None, None,Some((2*4,E)), Some((3*4,E))],
                   [None, None, None,None],
                 ],
            _ if *self == PipeType::S1
              => [ [None, Some((3*4,N)), Some((3*4,N)),None],
                   [None, Some((2*4,N)), Some((2*4,N)),None],
                   [None, None, None, None],
                   [None, None, None, None],
                 ],
            _ if *self == PipeType::S2
              => [ [None, None, None, None],
                   [None, None, None, None],
                   [None, Some((2*4,S)), Some((2*4,S)),None],
                   [None, Some((3*4,S)), Some((3*4,S)),None],
                 ],
            S if *self == PipeType::X || *self == PipeType::V || *self == PipeType::BV || *self == PipeType::V1
              => [ [None, Some((0*4,S)), Some((0*4,S)), None],
                   [None, Some((1*4,S)), Some((1*4,S)), None],
                   [None, Some((2*4,S)), Some((2*4,S)), None],
                   [None, Some((3*4,S)), Some((3*4,S)), None],
                 ],
            S if *self == PipeType::DL || *self == PipeType::DL1
              => [ [None, Some((0*4,S)), Some((0*4,S)), None],
                   [Some ((3*4,W)), Some((1*4,S)), Some((1*4,S)), None],
                   [Some ((3*4,W)), Some((2*4,W)), Some((2*4,S)), None],
                   [None, None, None, None],
                 ],
            S if *self == PipeType::DR || *self == PipeType::DR1
              => [ [None, Some((0*4,S)), Some((0*4,S)), None],
                   [None, Some((1*4,S)), Some((1*4,S)), Some ((3*4,E))],
                   [None, Some((2*4,S)), Some((2*4,E)), Some ((3*4,E))],
                   [None, None, None, None],
                 ],
            S if *self == PipeType::RV
              => [ [Some((0*4,S)), Some((0*4,S)), Some((0*4,S)), Some((0*4,S))],
                   [Some((1*4,S)), Some((1*4,S)), Some((1*4,S)), Some((1*4,S))],
                   [Some((2*4,S)), Some((2*4,S)), Some((2*4,S)), Some((2*4,S))],
                   [Some((3*4,S)), Some((3*4,S)), Some((3*4,S)), Some((3*4,S))],
                 ],
            N if *self == PipeType::X || *self == PipeType::V || *self == PipeType::BV || *self == PipeType::V2
              => [ [None, Some((3*4,N)), Some((3*4,N)), None],
                   [None, Some((2*4,N)), Some((2*4,N)), None],
                   [None, Some((1*4,N)), Some((1*4,N)), None],
                   [None, Some((0*4,N)), Some((0*4,N)), None],
                 ],
            N if *self == PipeType::UL || *self == PipeType::UL1
              => [ [None, None, None, None],
                   [Some ((3*4,W)), Some((2*4,W)), Some((2*4,N)), None],
                   [Some ((3*4,W)), Some((1*4,N)), Some((1*4,N)), None],
                   [None, Some((0*4,N)), Some((0*4,N)), None],
                 ],
            N if *self == PipeType::UR || *self == PipeType::UR1
              => [ [None, None, None, None],
                   [None, Some((2*4,N)), Some((2*4,E)), Some ((3*4,E))],
                   [None, Some((1*4,N)), Some((1*4,N)), Some ((3*4,E))],
                   [None, Some((0*4,N)), Some((0*4,N)), None],
                 ],
            N if *self == PipeType::RV
              => [ [Some((3*4,N)), Some((3*4,N)), Some((3*4,N)), Some((3*4,N))],
                   [Some((2*4,N)), Some((2*4,N)), Some((2*4,N)), Some((2*4,N))],
                   [Some((1*4,N)), Some((1*4,N)), Some((1*4,N)), Some((1*4,N))],
                   [Some((0*4,N)), Some((0*4,N)), Some((0*4,N)), Some((0*4,N))],
                 ],
            E if *self == PipeType::H || *self == PipeType::X || *self == PipeType::BH || *self == PipeType::H1
              => [ [None, None, None, None],
                   [Some((0*4,E)), Some((1*4,E)), Some((2*4,E)), Some((3*4,E))],
                   [Some((0*4,E)), Some((1*4,E)), Some((2*4,E)), Some((3*4,E))],
                   [None, None, None, None],
                 ],
            E if *self == PipeType::RH
              => [ [Some((0*4,E)), Some((1*4,E)), Some((2*4,E)), Some((3*4,E))],
                   [Some((0*4,E)), Some((1*4,E)), Some((2*4,E)), Some((3*4,E))],
                   [Some((0*4,E)), Some((1*4,E)), Some((2*4,E)), Some((3*4,E))],
                   [Some((0*4,E)), Some((1*4,E)), Some((2*4,E)), Some((3*4,E))],
                 ],
            E if *self == PipeType::UL || *self == PipeType::UL2
              => [ [None, None, None, None],
                   [Some ((0*4,E)), Some((1*4,E)), Some((2*4,E)), None],
                   [Some ((0*4,E)), Some((1*4,S)), Some((2*4,S)), None],
                   [None, Some((3*4,S)), Some((3*4,S)), None],
                 ],
            E if *self == PipeType::DL || *self == PipeType::DL2
              => [ [None, Some((3*4,N)), Some((3*4,N)), None],
                   [Some ((0*4,E)), Some((1*4,E)), Some((2*4,N)), None],
                   [Some ((0*4,E)), Some((1*4,E)), Some((2*4,E)), None],
                   [None, None,None, None],
                 ],
            W if *self == PipeType::H || *self == PipeType::X || *self == PipeType::BH || *self == PipeType::H2
              => [ [None, None, None, None],
                   [Some((3*4,W)), Some((2*4,W)), Some((1*4,W)), Some((0*4,W))],
                   [Some((3*4,W)), Some((2*4,W)), Some((1*4,W)), Some((0*4,W))],
                   [None, None, None, None],
                 ],
            W if *self == PipeType::RH
              => [ [Some((3*4,W)), Some((2*4,W)), Some((1*4,W)), Some((0*4,W))],
                   [Some((3*4,W)), Some((2*4,W)), Some((1*4,W)), Some((0*4,W))],
                   [Some((3*4,W)), Some((2*4,W)), Some((1*4,W)), Some((0*4,W))],
                   [Some((3*4,W)), Some((2*4,W)), Some((1*4,W)), Some((0*4,W))],
                 ],
            W if *self == PipeType::UR  || *self == PipeType::UR2
              => [ [None        , None, None, None],
                   [None, Some((2*4,W)), Some((1*4,W)), Some((0*4,W))],
                   [None, Some((2*4,S)), Some((1*4,S)), Some((0*4,W))],
                   [None, Some((3*4,S)), Some((3*4,S)), None],
                 ],
            W if *self == PipeType::DR  || *self == PipeType::DR2
              => [ [None, Some((3*4,N)), Some((3*4,N)), None],
                   [None, Some((2*4,N)), Some((1*4,W)), Some ((0*4,W))],
                   [None, Some((2*4,W)), Some((1*4,W)), Some ((0*4,W))],
                   [None, None,None, None],
                 ],
            _ => [ [None;4] ;4]
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Direction { N, S, E, W }
impl Direction {
    fn filled_ix(&self) -> usize {
        match self {
            Direction::N => 0,
            Direction::S => 1,
            Direction::E => 2,
            _ => 3
        }
    }
    fn opposite(&self) -> Direction {
        match self {
            Direction::N => Direction::S,
            Direction::S => Direction::N,
            Direction::E => Direction::W,
            Direction::W => Direction::E,
        }
    }
    fn move_to(&self, p : (usize,usize)) -> (usize,usize) {
        match self {
            Direction::N => (p.0,p.1-1),
            Direction::S => (p.0,p.1+1),
            Direction::E => (p.0+1,p.1),
            Direction::W => (p.0-1,p.1),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Splash {
    Paused, 
    Level(u32),
    GameOver
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PumpOutcome {
    Nothing(bool, bool, bool,bool),
    PumpTo(Direction),
    Start, EndPieceFilled
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Pipe {
    ty : PipeType,    
    filled: [u32;4],
}
impl Pipe {
    fn can_destroy(&self) -> bool {
        !(self.ty == PipeType::S1 || self.ty == PipeType::S2 || self.ty == PipeType::S3 || self.ty == PipeType::S4 || self.ty == PipeType::BL)
        && !(self.ty == PipeType::E1 || self.ty == PipeType::E2 || self.ty == PipeType::E3 || self.ty == PipeType::E4 )
        && self.filled == [0,0,0,0]
    }
    fn random_extended_pipe() -> Pipe {
        let pt = match thread_rng().gen_range(11,23) - 11 {
            0 => PipeType::DL1,
            1 => PipeType::DL2,
            2 => PipeType::UL1,
            3 => PipeType::UL2,
            4 => PipeType::DR1,
            5 => PipeType::DR2,
            6 => PipeType::UR1,
            7 => PipeType::UR2,
            8 => PipeType::H1,
            9 => PipeType::H2,
            10 => PipeType::V1,
            _ => PipeType::V2,
        };
        Pipe { ty: pt, filled: [0,0,0,0] }
    }
    fn random_pipe() -> Pipe {
        let pt = match thread_rng().gen_range(1,8) {
            1 => PipeType::UR,
            2 => PipeType::UL,
            3 => PipeType::DR,
            4 => PipeType::DL,
            5 => PipeType::X,
            6 => PipeType::V,
            7 => PipeType::H,
            8 => PipeType::RH,
            9 => PipeType::RV,
            _ => PipeType::BL,
        };
        Pipe { ty: pt, filled: [0,0,0,0] }
    }
    fn pump(&mut self, origin: Direction ) -> PumpOutcome {
        if self.filled[origin.filled_ix()] == self.ty.capacity() && self.ty != PipeType::E1 && self.ty != PipeType::E2 && self.ty != PipeType::E3 && self.ty != PipeType::E4  {
            PumpOutcome::PumpTo(self.ty.output_for(origin))
        } else {
            let b = self.filled[origin.filled_ix()] == 0;
            self.filled[origin.filled_ix()] += 1;
            if self.ty == PipeType::E1 || self.ty == PipeType::E2 || self.ty == PipeType::E3 || self.ty == PipeType::E4 {
                if self.filled[origin.filled_ix()] > self.ty.capacity() - 5 {
                    self.filled[origin.filled_ix()] = self.ty.capacity() -5;
                    return PumpOutcome::EndPieceFilled
                }
            };
            let mut b2 = false;
            for i in 0..4 {
                if i != origin.filled_ix() && self.filled[i] > 0 {
                    b2 = true;
                }
            }
            PumpOutcome::Nothing(b,self.ty == PipeType::RH || self.ty == PipeType::RV || self.ty == PipeType::BH || self.ty == PipeType::BV,b2, self.ty==PipeType::E1 || self.ty==PipeType::E2 || self.ty ==PipeType::E3 || self.ty == PipeType::E4)
        }
    }
    fn draw<'t>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'t>>, position: (i32,i32)) {
        let i = match self.ty {
            PipeType::BL => 0,
            PipeType::UR => 1,
            PipeType::DR => 2,
            PipeType::UL => 3,
            PipeType::DL => 4,
            PipeType::V => 5,
            PipeType::H => 6,
            PipeType::X => 7,
            PipeType::RV => 8,
            PipeType::RH => 9,
            PipeType::S2 => 10,
            PipeType::UR1 => 11,
            PipeType::UR2 => 12,
            PipeType::UL1 => 13,
            PipeType::UL2 => 14,
            PipeType::DR1 => 15,
            PipeType::DR2 => 16,
            PipeType::DL1 => 17,
            PipeType::DL2 => 18,
            PipeType::V1 => 19,
            PipeType::V2 => 20,
            PipeType::H1 => 21,
            PipeType::H2 => 22,
            PipeType::S1 => 23,
            PipeType::S3 => 24,
            PipeType::S4 => 25,
            PipeType::BH => 26,
            PipeType::BV => 27,
            PipeType::E1 => 28,
            PipeType::E2 => 29,
            PipeType::E3 => 30,
            PipeType::E4 => 31,
        };
        if self.filled != [0,0,0,0] {
            for d in &[Direction::N,Direction::S,Direction::E,Direction::W] {
                let flow = self.ty.flows(*d);
                for x in 0..4 {
                    for y in 0..4 {
                        match flow[y][x] {
                            None => {},
                            Some((g,d2)) =>{
                                let factor = if self.ty == PipeType::RH || self.ty == PipeType::RV {2} 
                                            else if self.ty == PipeType::S1 || self.ty == PipeType::S2 || self.ty == PipeType::S4 || self.ty == PipeType::S3 {4}
                                            else {1};
                                let remaining = 
                                    self.filled[d.filled_ix()] as i32 / factor
                                    - (g as i32);
                                let clamped = remaining.min(4).max(0) as usize;
                                if clamped > 0 {
                                    graphics.fill_cells[d2.filled_ix()][clamped].draw(canvas,(position.0+8*x as i32,position.1+8*y as i32))
                                }                        
                            }
                        }
                    }
                }
                if self.filled[d.filled_ix()] > self.ty.capacity() - 2 {
                    let d2 = self.ty.output_for(*d);
                    let position2 = match d2 {
                        Direction::N => (position.0+12,position.1-8),
                        Direction::S => (position.0+12,position.1+32),
                        Direction::E => (position.0+32,position.1+12),
                        Direction::W => (position.0-8,position.1+12),
                    };
                    graphics.fill_cells[d2.filled_ix()][(self.filled[d.filled_ix()] + 2 - self.ty.capacity()) as usize].draw(canvas,(position2.0,position2.1))
                }
            }
        }
        graphics.pipe_pieces[i][0].draw(canvas,(position.0+2,position.1-2));
        graphics.pipe_pieces[i][2].draw(canvas,(position.0+2,position.1+2));
        graphics.pipe_pieces[i][2].draw(canvas,(position.0,position.1+2));
        graphics.pipe_pieces[i][2].draw(canvas,(position.0-2,position.1+2));
        graphics.pipe_pieces[i][2].draw(canvas,(position.0+2,position.1));
        graphics.pipe_pieces[i][0].draw(canvas,(position.0-2,position.1-2));
        graphics.pipe_pieces[i][0].draw(canvas,(position.0-2,position.1));
        graphics.pipe_pieces[i][0].draw(canvas,(position.0,position.1-2));
        graphics.pipe_pieces[i][1].draw(canvas,(position.0,position.1));
        graphics.pipe_pieces[i][0].draw(canvas,(position.0+1,position.1-1));
        graphics.pipe_pieces[i][2].draw(canvas,(position.0+1,position.1+1));
        graphics.pipe_pieces[i][2].draw(canvas,(position.0,position.1+1));
        graphics.pipe_pieces[i][2].draw(canvas,(position.0-1,position.1+1));
        graphics.pipe_pieces[i][2].draw(canvas,(position.0+1,position.1));
        graphics.pipe_pieces[i][0].draw(canvas,(position.0-1,position.1-1));
        graphics.pipe_pieces[i][0].draw(canvas,(position.0-1,position.1));
        graphics.pipe_pieces[i][0].draw(canvas,(position.0,position.1-1));
        graphics.pipe_pieces[i][1].draw(canvas,(position.0,position.1));
    }
}
pub struct Table {    
    pieces: [[Option<Pipe>;8];10],
    queue: VecDeque<Pipe>, 
    oneway_chance: u8,
    score: u32,
    required: u32,
    bonus:bool,
    has_endpiece: bool,
    level: u32,
    tick_cap: u32,
    cleaning:bool,
    fastforward: bool,
    splash:Option<Splash>,
    crossovers: u8,
    looping_rows: Vec<usize>,
    looping_cols: Vec<usize>,
    level_over: Option<(Direction, (usize,usize))>,
    destroying: Option<(Option<Pipe>, (usize,usize))>,
    destroying_progress: u8,
    source_pos: (usize,usize),
    cursor_location: Option<(usize,usize)>,
}

impl Table {
    fn fill_queues(&mut self) {
        while self.queue.len() < 6 {
            let pipe = if thread_rng().gen_range(0,100) < self.oneway_chance {
                Pipe::random_extended_pipe()
            } else {
                Pipe::random_pipe()
            };
            self.queue.push_back(pipe);
        }
    }
    fn populate_board(&mut self) {
        self.setup_level(self.level);
    }
    fn next_level(&mut self) {
        if self.bonus {
            self.bonus = false;
            self.setup_level(self.level);
        } else {
            self.level += 1;
            if self.level % 4 == 0 {
                self.bonus = true;
                self.setup_bonus_level()
            } else {
                self.setup_level(self.level);
            }
        }
        
    }
    fn previous_level(&mut self) {
        if self.level > 0 && self.level % 4 == 0 && !self.bonus {
            self.bonus = true;
            self.setup_bonus_level();
            return;
        } else if self.level % 4 == 0 && self.bonus {
            self.bonus = false;            
        }
        self.level = self.level.max(1) - 1;
        self.setup_level(self.level);
    }
    fn setup_bonus_level(&mut self) {
        self.setup_level(self.level);
        self.required = 0;
        self.has_endpiece = false;
        for x in 0..10 {
            for y in 0..8 {
                if let Some(p) = &self.pieces[x][y] {
                    if p.ty == PipeType::BL || p.ty == PipeType::E1 || p.ty == PipeType::E2 || p.ty == PipeType::E3 || p.ty == PipeType::E4 {
                        self.pieces[x][y] = self.queue.pop_front();
                    }
                } else {
                    self.pieces[x][y] = self.queue.pop_front();
                }
                self.fill_queues()
            }
        }
        self.queue = VecDeque::new();
        if let Some(p) = &self.pieces[self.source_pos.0][self.source_pos.1] {
            let (x,y) = p.ty.output_for(Direction::S).move_to(self.source_pos);
            self.pieces[x][y] = None;
        }
        
    }
    fn setup_level(&mut self, level: u32) {
        self.has_endpiece = false;
        self.level_over = None;
        self.queue = VecDeque::new();
        self.fastforward = false;
        self.pieces = Default::default();
        self.level = level;
        self.crossovers = 0;
        self.required = 15 + level / 10;
        self.tick_cap = 13 - (level / 8);
        self.oneway_chance = level.max(15) as u8 - 15;
        self.source_pos = (thread_rng().gen_range(1,9), thread_rng().gen_range(1,7));
        self.pieces[self.source_pos.0][self.source_pos.1] = Some(Pipe {
            filled: [0,0,0,0],
            ty: match thread_rng().gen_range(0,4) {
                0 => PipeType::S1,
                1 => PipeType::S2,
                2 => PipeType::S3,
                _ => PipeType::S4,
        }});
        self.looping_rows = vec![];
        self.looping_cols = vec![];
        if level % 3 == 0 && level > 0 {
            loop {
                let x = thread_rng().gen_range(1,9);
                let y = thread_rng().gen_range(1,7);
                if x == self.source_pos.0 || y == self.source_pos.1  { continue }
                self.pieces[x][y] = Some(Pipe {
                    filled: [0,0,0,0],
                    ty: match thread_rng().gen_range(0,4) {
                        0 => PipeType::E1,
                        1 => PipeType::E2,
                        2 => PipeType::E3,
                        _ => PipeType::E4,
                }});
                self.required -= 3;
                self.has_endpiece = true;
                break;
            }
        }
        let mut loops = ((level.max(5) - 5) / 4).min(5); 
        while loops > 0 {
            match thread_rng().gen_range(0,2) {
                0 => { 
                    let x = thread_rng().gen_range(0,10);
                    if !self.looping_cols.contains(&x) {
                        self.looping_cols.push(x);
                        loops -= 1;
                    }
                },
                _ => {
                    let x = thread_rng().gen_range(0,8);
                    if !self.looping_rows.contains(&x) {
                        self.looping_rows.push(x);
                        loops -= 1;
                    }
                } 
            }
        }
        let mut blocks = ((level.max(2) - 2) / 4).min(7); 
        while blocks > 0 {
            let x = thread_rng().gen_range(0,10);
            let y = thread_rng().gen_range(0,8);
            if self.pieces[x][y].is_none() {
                self.pieces[x][y] = Some(Pipe { filled: [0,0,0,0], ty: PipeType::BL});
                if self.is_unsafe() {
                    self.pieces[x][y] = None;
                } else {
                    blocks -= 1;
                }
            }
        }
        let mut bonuses = (level / 4).min(2); 
        while bonuses > 0 {
            let x = thread_rng().gen_range(0,10);
            let y = thread_rng().gen_range(0,8);
            if self.pieces[x][y].is_none() {
                self.pieces[x][y] = Some(Pipe { filled: [0,0,0,0], ty: match thread_rng().gen_range(0,2) {
                    0 => PipeType::BH, 
                    _ => PipeType::BV
                }});
                if self.is_unsafe() {
                    self.pieces[x][y] = None;
                } else {
                    bonuses -= 1;
                }
            }
        }
        let mut reservoirs = (level / 6).min(2); 
        while reservoirs > 0 {
            let x = thread_rng().gen_range(0,10);
            let y = thread_rng().gen_range(0,8);
            if self.pieces[x][y].is_none() {
                self.pieces[x][y] = Some(Pipe { filled: [0,0,0,0], ty: match thread_rng().gen_range(0,2) {
                    0 => PipeType::RH, 
                    _ => PipeType::RV
                }});
                if self.is_unsafe() {
                    self.pieces[x][y] = None;
                } else {
                    reservoirs -= 1;
                }
            }
        }
        self.fill_queues();
    }
    fn is_unsafe(&self) -> bool {
        for y in 0..8 {
            for x in 0..10 {
                if let Some(p) = &self.pieces[x][y] {
                    for d in &[Direction::N, Direction::S, Direction::E, Direction::W] {
                        if p.ty.accepts(*d) || p.ty == PipeType::S1 || p.ty == PipeType::S2 || p.ty == PipeType::S3 || p.ty == PipeType::S4 {
                            let d2 = p.ty.output_for(*d);
                            if d2 == Direction::S && y == 7
                            || d2 == Direction::N && y == 0
                            || d2 == Direction::E && x == 9
                            || d2 == Direction::W && x == 0 {
                                return true;
                            } else {
                                let p = d2.move_to((x,y));
                                if let Some(p2) = &self.pieces[p.0][p.1] {
                                    if !p2.ty.accepts(d2) {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }
    fn new() -> Table {
        let mut table = Table {
            pieces: Default::default(),
            queue: VecDeque::new(),
            bonus:false,
            fastforward: false,
            cursor_location: None, 
            destroying: None, 
            has_endpiece:false,
            splash:Some(Splash::Level(0)),
            crossovers:0,
            destroying_progress: 0,
            level_over: None,           
            score: 0,
            level: 0,
            cleaning: false,
            required:0,
            tick_cap: 12,
            oneway_chance: 20,
            source_pos:(4,4),
            looping_rows: vec![],
            looping_cols: vec![]
        };
        table.populate_board();
        table
    }
    
    fn clicked(&mut self) {
        if self.destroying.is_some() { return; }
        if let Some((x,y)) = self.cursor_location {        
            if self.queue.front().is_some() || self.bonus {
                if let Some(pp) = &self.pieces[x][y] {                    
                    if pp.ty == PipeType::S1 || pp.ty == PipeType::S2 || pp.ty == PipeType::S3 || pp.ty == PipeType::S4 {
                        self.fastforward = true;
                    } else if !self.bonus && pp.can_destroy () {
                        self.score = self.score.max(50) - 50;
                        self.destroying = Some((self.queue.pop_front(),(x,y)));
                    } else if self.bonus && y > 0 && self.pieces[x][y-1].is_none() {
                        self.pieces[x][y-1] = self.pieces[x][y].clone();
                        self.pieces[x][y] = None;
                    } else if self.bonus && y < 7 && self.pieces[x][y+1].is_none() {
                        self.pieces[x][y+1] = self.pieces[x][y].clone();
                        self.pieces[x][y] = None;
                    } else if self.bonus && x > 0 && self.pieces[x-1][y].is_none() {
                        self.pieces[x-1][y] = self.pieces[x][y].clone();
                        self.pieces[x][y] = None;
                    } else if self.bonus && x < 9 && self.pieces[x+1][y].is_none() {
                        self.pieces[x+1][y] = self.pieces[x][y].clone();
                        self.pieces[x][y] = None;
                    }
                } else if !self.bonus && self.pieces[x][y].is_none() {
                    self.pieces[x][y] = Some(self.queue.pop_front().unwrap());
                }
                //
                if !self.bonus { self.fill_queues() }
            }
        }
    }
    fn draw<'t>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'t>>) {
        //canvas.set_draw_color(rgba(238,238,236,255));
        canvas.set_draw_color(DARK_CHARCOAL);
        canvas.clear();

        //canvas.fill_rect(rect: R)
        for y in 0..8 {
            for x in 0..10 {
                let _ = if x % 2 == 0 && y % 2 == 0 || x % 2 == 1 && y % 2 == 1 { 
                    //canvas.set_draw_color(rgba(221,215,217,255));
                    canvas.set_draw_color(CHARCOAL);//rgba(85,87,83,255));
                    canvas.fill_rect(Rect::new(x *36+4,y*36+16+5,32,32)).ok();
                } else { 
                    //canvas.set_draw_color(rgba(211,205,207,255));
                    canvas.set_draw_color(rgba(115,120,110,255));
                    //canvas.fill_rect(Rect::new(x *36+4,y*36+16+5,32,32)).ok() 
                };
                if x == 0 && self.looping_rows.contains(&(y as usize)) {
                    graphics.direction_marker[1].draw(canvas,(x*36+4,y*36+16+5));
                }
                if y == 0 && self.looping_cols.contains(&(x as usize)) {
                    graphics.direction_marker[0].draw(canvas,(x*36+4,y*36+16+5));
                }
                if x == 9 && self.looping_rows.contains(&(y as usize)) {
                    graphics.direction_marker[1].draw(canvas,(x*36-4+32,y*36+16+5));
                }
                if y == 7 && self.looping_cols.contains(&(x as usize)) {
                    graphics.direction_marker[0].draw(canvas,(x*36+4,y*36+16-5+32));
                }
            }
        }
        for x in (0..10).rev() {
            for y in (0..8).rev() {
                if let Some(t) = &self.pieces[x][y] {
                    t.draw(canvas, graphics, (x as i32*36+4,y as i32*36+16+5));
                }
            }
        }
        if let Some((_,(x,y))) = &self.destroying {
            let xx = *x as i32 * 36 + 4;
            let yy = *y as i32 * 36 + 17;
            let position = (xx,yy+4);
            let i = self.destroying_progress as usize / 2;
            graphics.covering_cells[i][0].draw(canvas,(position.0+2,position.1-2));
            graphics.covering_cells[i][2].draw(canvas,(position.0+2,position.1+2));
            graphics.covering_cells[i][2].draw(canvas,(position.0,position.1+2));
            graphics.covering_cells[i][2].draw(canvas,(position.0-2,position.1+2));
            graphics.covering_cells[i][2].draw(canvas,(position.0+2,position.1));
            graphics.covering_cells[i][0].draw(canvas,(position.0-2,position.1-2));
            graphics.covering_cells[i][0].draw(canvas,(position.0-2,position.1));
            graphics.covering_cells[i][0].draw(canvas,(position.0,position.1-2));
            graphics.covering_cells[i][1].draw(canvas,(position.0,position.1));
            graphics.covering_cells[i][0].draw(canvas,(position.0+1,position.1-1));
            graphics.covering_cells[i][2].draw(canvas,(position.0+1,position.1+1));
            graphics.covering_cells[i][2].draw(canvas,(position.0,position.1+1));
            graphics.covering_cells[i][2].draw(canvas,(position.0-1,position.1+1));
            graphics.covering_cells[i][2].draw(canvas,(position.0+1,position.1));
            graphics.covering_cells[i][0].draw(canvas,(position.0-1,position.1-1));
            graphics.covering_cells[i][0].draw(canvas,(position.0-1,position.1));
            graphics.covering_cells[i][0].draw(canvas,(position.0,position.1-1));
            graphics.covering_cells[i][1].draw(canvas,(position.0,position.1));
        }
        if let Some((d,(x,y))) = self.level_over {
            let mut xx = x as i32 * 36 + 4;
            let mut yy = y as i32 * 36 + 17;
            match d {
                Direction::N => yy -= 16,
                Direction::S => yy += 16,
                Direction::E => xx += 16,
                Direction::W => xx -= 16,
            }
            graphics.error_marker_shadow.draw(canvas,(xx-2,yy));
            graphics.error_marker_shadow.draw(canvas,(xx-2,yy+2));
            graphics.error_marker_shadow.draw(canvas,(xx-2,yy-2));
            graphics.error_marker_shadow.draw(canvas,(xx+2,yy));
            graphics.error_marker_shadow.draw(canvas,(xx+2,yy+2));
            graphics.error_marker_shadow.draw(canvas,(xx+2,yy-2));
            graphics.error_marker_shadow.draw(canvas,(xx  ,yy+2));
            graphics.error_marker_shadow.draw(canvas,(xx  ,yy-2));
            graphics.error_marker_shadow.draw(canvas,(xx-2,yy));
            graphics.error_marker_highlight.draw(canvas,(xx-1,yy+1));
            graphics.error_marker_highlight.draw(canvas,(xx-1,yy-1));
            graphics.error_marker_highlight.draw(canvas,(xx-1,yy));
            graphics.error_marker_highlight.draw(canvas,(xx+1,yy));
            graphics.error_marker_highlight.draw(canvas,(xx+1,yy+1));
            graphics.error_marker_highlight.draw(canvas,(xx+1,yy-1));
            graphics.error_marker_highlight.draw(canvas,(xx  ,yy+1));
            graphics.error_marker_highlight.draw(canvas,(xx  ,yy-1));
            graphics.error_marker.draw(canvas,(xx,yy));

        }
        if let Some((x,y)) = self.cursor_location {
            if self.destroying.is_none() { 
                graphics.cursor_outline.draw(canvas,(x as i32 * 36-1,y as i32 *36+16));
                graphics.cursor_outline.draw(canvas,(x as i32 * 36+1,y as i32 *36+16));
                graphics.cursor_outline.draw(canvas,(x as i32 * 36-1,y as i32 *36+16-1));
                graphics.cursor_outline.draw(canvas,(x as i32 * 36+1,y as i32 *36+16-1));
                graphics.cursor_outline.draw(canvas,(x as i32 * 36,  y as i32 *36+16-1));
                graphics.cursor_outline.draw(canvas,(x as i32 * 36-1,y as i32 *36+16+1));
                graphics.cursor_outline.draw(canvas,(x as i32 * 36+1,y as i32 *36+16+1));
                graphics.cursor_outline.draw(canvas,(x as i32 * 36,y as i32 *36+16+1));
                graphics.cursor.draw(canvas,(x as i32 * 36,y as i32 *36+16));
                if self.bonus {
                    let mut d = 255;
                    if x > 0 && self.pieces[x-1][y].is_none() {
                        d = 3
                    } else if x < 9 && self.pieces[x+1][y].is_none() {
                        d = 2
                    } else if y > 0 && self.pieces[x][y-1].is_none() {
                        d = 0
                    } else if y < 7 && self.pieces[x][y+1].is_none() {
                        d = 1
                    }
                    if let Some(p) = &self.pieces[x][y] {
                        if p.ty == PipeType::S1 || p.ty == PipeType::S2 || p.ty == PipeType::S3 || p.ty == PipeType::S4 {
                            d = 255;
                        }
                    }
                    if d < 4 {                        
                        let pos = (x as i32*36+4,y as i32*36+16+4);
                        graphics.bonus_cursors[d][1].draw(canvas,(pos.0-1,pos.1));
                        graphics.bonus_cursors[d][1].draw(canvas,(pos.0+1,pos.1));
                        graphics.bonus_cursors[d][1].draw(canvas,(pos.0-1,pos.1-1));
                        graphics.bonus_cursors[d][1].draw(canvas,(pos.0+1,pos.1-1));
                        graphics.bonus_cursors[d][1].draw(canvas,(pos.0,pos.1-1));
                        graphics.bonus_cursors[d][1].draw(canvas,(pos.0-1,pos.1+1));
                        graphics.bonus_cursors[d][1].draw(canvas,(pos.0+1,pos.1+1));
                        graphics.bonus_cursors[d][1].draw(canvas,(pos.0,pos.1+1));
                        graphics.bonus_cursors[d][0].draw(canvas,(pos.0,pos.1));
                    }
                }
            }
        }
        canvas.set_draw_color(rgba(115,120,118,255));
        let _ = canvas.fill_rect(Rect::new(364,0,50,395));
        canvas.set_draw_color(UI_LIGHT);
        let _ = canvas.fill_rect(Rect::new(0,308,420,20));
        canvas.set_draw_color(UI_DARK);
        let _ = canvas.fill_rect(Rect::new(364,0,1,308));
        let _ = canvas.fill_rect(Rect::new(0,308,420,1));
        for i in 0..self.queue.len() {
            let t = &self.queue[i];
            t.draw(canvas,graphics,(371,i as i32 *40+15+8 + if i > 0 {16} else {0}));
        }
    }
    fn subtick(&mut self) {
        if self.destroying.is_some() {
            self.destroying_progress += 1;
            if self.destroying_progress == 32 {
                self.destroying_progress = 0;
                let (p,(x,y)) = self.destroying.clone().unwrap();
                self.pieces[x][y] = p;
                self.destroying = None;

            }
        }
        if self.destroying.is_none() {
            if self.cleaning {
                if !self.bonus {
                    for x in 0..10 {
                        for y in 0..8 {
                            if let Some(p) = &self.pieces[x][y] {
                                if p.can_destroy() {
                                    self.score = self.score.max(100) - 100;
                                    self.destroying = Some((None,(x,y)));
                                }
                            }
                        }
                    }
                }
                if self.destroying.is_none() {
                    self.cleaning = false;
                    self.level_over();
                }
            }
        }
    }
    fn level_over(&mut self) {
        if self.required > 0 || self.has_endpiece {
            self.splash = Some(Splash::GameOver)
        } else {
            self.next_level();
            self.splash = Some(Splash::Level(self.level));
        }
    }
    fn tick(&mut self) {
        let mut origin = self.source_pos;
        let mut dir = Direction::S;
        let mut outcome = PumpOutcome::Start;
        loop {
            match outcome {
                PumpOutcome::Start => {
                    if let Some(ref mut p) = self.pieces[origin.0][origin.1] {
                        outcome = p.pump(dir);
                    } else { break }
                },
                PumpOutcome::EndPieceFilled => {
                    self.level_over = None;
                    self.cleaning = true;
                    break;
                }
                PumpOutcome::PumpTo(d) => {
                    let old = origin;
                    if d == Direction::S && origin.1 == 7 {
                        if self.looping_cols.contains(&origin.0) {
                            origin.1 = 0;
                        } else {
                            self.level_over = Some((d,old));
                            self.cleaning = true;
                            break
                        }
                    } else if d == Direction::N && origin.1 == 0 {
                        if self.looping_cols.contains(&origin.0) {
                            origin.1 = 7;
                        } else {
                            self.level_over = Some((d,old));
                            self.cleaning = true;
                            break
                        }
                    } else if d == Direction::E && origin.0 == 9 {
                        if self.looping_rows.contains(&origin.1) {
                            origin.0 = 0;
                        } else {
                            self.level_over = Some((d,old));
                            self.cleaning = true;
                            break
                        }
                    } else if d == Direction::W && origin.0 == 0 {
                        if self.looping_rows.contains(&origin.1) {
                            origin.0 = 9;
                        } else {
                            self.level_over = Some((d,old));
                            self.cleaning = true;
                            break
                        }
                    } else {
                        origin = d.move_to(origin);
                    }
                    dir = d.opposite();
                    if let Some(ref mut p) = self.pieces[origin.0][origin.1] {
                        if p.ty.accepts(dir) {
                            outcome = p.pump(dir);
                        } else {
                            self.level_over = Some((d,old));
                            self.cleaning = true;
                            break
                        }
                    } else {
                        self.level_over = Some((d,old));
                        self.cleaning = true;
                        break; 
                    }
                },
                PumpOutcome::Nothing(new, bonus,crossover,endpiece) => {
                    let mut score = 0;
                    if new {
                        if self.required > 0 {
                            self.required -= 1;
                            score += 50;
                        } else {
                            score += 100;
                        }
                        if crossover {
                            score += 500;
                            self.crossovers += 1;
                            if self.crossovers % 5 == 0 && self.crossovers >= 5 {
                                self.score += 10000
                            }
                        }
                        if endpiece {
                            score += 1000;
                            self.has_endpiece = false;
                        }
                        if bonus {
                            score *= 10;
                        }
                        if self.fastforward {
                            score *= 2;
                        }
                        self.score += score;
                    }
                    break;
                }
            }
        }
    }
    fn collides_location(&self, position: (i32,i32)) -> Option<(usize, usize)> {
        let bx = (position.0 - 6) / 36;
        let by = (position.1 - 6 - 16) / 36;
        if bx >= 10 || by >= 8 { return None };
        if bx < 0 || by < 0 { return None };
        return Some((bx as usize,by as usize))
    }
}
const WIDTH:u32=409;
const HEIGHT:u32=308+18;
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

    let mut level_gfx = Graphic::blank(8,1).textured(&texture_creator);
    let mut score_gfx = Graphic::blank(6,1).textured(&texture_creator);
    let mut menu = MenuBar::new(WIDTH)
                    .add(Menu::new("GAME",132+16,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Next Level",25, Keycode::X,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Previous Level",27, Keycode::Z,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(132+2, &texture_creator, &graphics_set.tile_set))                            
                            .add(MenuItem::new("Quit",363, Keycode::F12,&texture_creator,&graphics_set.tile_set)))
                    .add(Menu::new("ACTION",92+16+16+8,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Fast-Forward", 7, Keycode::F,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Pause", 17, Keycode::P,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(92+8+16, &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Restart",15, Keycode::N,&texture_creator,&graphics_set.tile_set)))
                    .add(Menu::new("VIEW",72+(5*8),&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Micro-mode",360, Keycode::F9,&texture_creator, &graphics_set.tile_set)));
    let mut rate_limiter = FPSManager::new();
    rate_limiter.set_framerate(60).unwrap();
    let mut micro_mode = false;
    let mut tick_counter = 0;
    let mut starting_level = 0;
    let mut lose = Graphic::load_from(Cursor::new(&include_bytes!("../lose")[..])).unwrap().textured(&texture_creator);
    lose.update_texture(&graphics_set.tile_set);
    let mut level_splash = Graphic::load_from(Cursor::new(&include_bytes!("../level_splash")[..])).unwrap().textured(&texture_creator);
    level_splash.update_texture(&graphics_set.tile_set);
    let mut paused_splash = Graphic::load_from(Cursor::new(&include_bytes!("../paused_splash")[..])).unwrap().textured(&texture_creator);
    paused_splash.update_texture(&graphics_set.tile_set);

        let mut md = false;
    loop {
        canvas.set_draw_color(CHARCOAL);
        canvas.clear();
        table.draw(&mut canvas, &graphics_set);
        score_gfx.draw_rect(0, 0, 6, 1, Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:0});
        score_gfx.draw_text(&table.score.to_string(), &graphics_set.tile_set , 0, 0, DARK_CHARCOAL, TRANSPARENT);
        score_gfx.update_texture(&graphics_set.tile_set);

        level_gfx.draw_rect(0,0,8,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        if table.bonus {
            level_gfx.draw_text(&("Bonus ".to_owned() + &(table.level/4).to_string()),&graphics_set.tile_set,0,0,BLACK,TRANSPARENT);
        } else {
            level_gfx.draw_text(&("Level ".to_owned() + &(table.level+1).to_string()),&graphics_set.tile_set,0,0,BLACK,TRANSPARENT);
        }
        
        level_gfx.update_texture(&graphics_set.tile_set);
        for i in 0..table.required {
            let position = (396 - i as i32 * 12,HEIGHT as i32-36);
            graphics_set.progress_cell[1].draw(& mut canvas,(position.0,position.1));
            graphics_set.progress_cell[0].draw(& mut canvas,(position.0+1,position.1-1));
            graphics_set.progress_cell[2].draw(& mut canvas,(position.0+1,position.1+1));
            graphics_set.progress_cell[2].draw(& mut canvas,(position.0,position.1+1));
            graphics_set.progress_cell[2].draw(& mut canvas,(position.0-1,position.1+1));
            graphics_set.progress_cell[2].draw(& mut canvas,(position.0+1,position.1));
            graphics_set.progress_cell[0].draw(& mut canvas,(position.0-1,position.1-1));
            graphics_set.progress_cell[0].draw(& mut canvas,(position.0-1,position.1));
            graphics_set.progress_cell[0].draw(& mut canvas,(position.0,position.1-1));
            graphics_set.progress_cell[1].draw(& mut canvas,(position.0,position.1));
        }
        
        if let Some(s) = &table.splash {
            match *s {
                Splash::Level(_) => { 
                    level_splash.draw(&mut canvas, (WIDTH as i32 /2 - (11*4), HEIGHT as i32 /2 - 16 ));
                    level_gfx.draw(&mut canvas, (WIDTH as i32 /2 - (8 * 4) + if table.level < 9 { 4} else {0}, HEIGHT as i32 /2 - 4));
                },
                Splash::GameOver => lose.draw(&mut canvas, (WIDTH as i32 /2 - (21*4), HEIGHT as i32 /2 - (21*4) )),
                Splash::Paused => paused_splash.draw(&mut canvas, (WIDTH as i32 /2 - (11*4), HEIGHT as i32 /2 - 20)),
            }
        }
        menu.draw(&mut canvas);
        score_gfx.draw(&mut canvas, (4 as i32,HEIGHT as i32-12));
        canvas.present();
        if table.splash.is_none() {
            tick_counter += 1;
            tick_counter %= if table.fastforward {1} else { table.tick_cap };
            if tick_counter == 0 { table.tick(); }
        }
        table.subtick();
        for event in event_pump.poll_iter() {
            let h = menu.handle_event(event.clone(), &mut event_subsystem);
            match event {
                _ if h => {},
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::F12), .. } => {
                    return
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
                Event::MouseButtonDown { ..} if table.splash.is_some() => {
                    md = true;
                },
                Event::MouseButtonUp { ..} if table.splash.is_some() && md => {
                    md= false;
                    if table.splash == Some(Splash::GameOver) {
                        table = Table::new();
                        table.setup_level(starting_level);
                        table.splash = Some(Splash::Level(starting_level));
                    } else {
                        table.splash = None;
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Z), ..} => {
                    if table.level == starting_level {
                        starting_level = starting_level.max(1) - 1;
                    }
                    table.previous_level();
                    table.splash = Some(Splash::Level(table.level));
                    table.score = 0;
                },

                Event::KeyDown { keycode: Some(Keycode::N), ..} => {
                    table = Table::new();                   
                    table.setup_level(starting_level);
                    table.splash = Some(Splash::Level(table.level));
                    table.score = 0;
                },
                Event::KeyDown { keycode: Some(Keycode::X), ..} => {
                    if table.level == starting_level {
                        starting_level += 1;
                    }
                    table.next_level();
                    table.splash = Some(Splash::Level(table.level));
                    table.score = 0;
                },
                Event::KeyDown {..} if table.splash.is_some() => {
                    if table.splash == Some(Splash::GameOver) {
                        table = Table::new();
                        table.setup_level(starting_level);
                        table.splash = Some(Splash::Level(starting_level));
                    } else {
                        table.splash = None;
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::F), ..} if table.splash.is_none() => {
                    table.fastforward = true;
                },
                Event::MouseButtonUp { ..} if table.splash.is_none() => {
                    md = false ;
                }
                Event::MouseButtonDown { mouse_btn: _, x, y, ..} if table.splash.is_none() => {
                    let loc = table.collides_location((x,y));
                    table.cursor_location = loc;
                    table.clicked();
                    table.tick();
                }
                Event::MouseMotion { x, y, ..} if table.splash.is_none() => {
                    let loc = table.collides_location((x,y));
                    table.cursor_location = loc;
                }
                Event::KeyDown { keycode: Some(Keycode::P), ..} => {
                    table.splash = Some(Splash::Paused);
                }

                _ => {},
            }
        }
        rate_limiter.delay();
        
    }
}


fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("brooklyn", WIDTH, HEIGHT)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    main_loop(window, &sdl_context);
}