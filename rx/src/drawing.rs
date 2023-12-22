use ::imprint::{Cell,Imprint};
use game::{Game, Status};
use game::piece::{FieldTile, PieceColor};

use sdl2::pixels::Color;
use sdl2::render::RenderTarget;
use sdl2::render::{Canvas,Texture};
use tesserae::{*};
use utils::OutlinedTile;
use std::io::Cursor;
use sdl2::render::TextureCreator;
use utils::color::{*};

use crate::game::GameSpeed;
pub struct BaseDrawingContext<'r> {
    pub tile_set : TileSet,
    chrome: Graphic<Texture<'r>>, 
    graphic_hl: OutlinedTile<'r>,
    graphic_hl2: OutlinedTile<'r>,
    graphic_virus: [OutlinedTile<'r>;3],
    graphic_virus2: [OutlinedTile<'r>;3],
    graphic_orphan: [OutlinedTile<'r>;3],
    graphic_tab_l: [OutlinedTile<'r>;3],
    graphic_tab_r: [OutlinedTile<'r>;3],
    graphic_tab_t: [OutlinedTile<'r>;3],
    graphic_tab_b: [OutlinedTile<'r>;3],
    graphic_speed: [OutlinedTile<'r>;3],
    score: Graphic<Texture<'r>>,
    top_score: Graphic<Texture<'r>>,
    level: Graphic<Texture<'r>>,
    press_space: Graphic<Texture<'r>>,
    tick: u32,
}
impl <'r> BaseDrawingContext<'r> {
    pub fn new<T>(texture_creator: &'r TextureCreator<T>) -> BaseDrawingContext<'r> {
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));        
        let mut chrome = Graphic::load_from(Cursor::new(&include_bytes!("../chrome")[..])).unwrap().textured(texture_creator);
        chrome.update_texture(&tile_set);
        let mut press_space = Graphic::blank(5,2).textured(texture_creator);
        press_space.draw_text("PRESS", &tile_set, 0, 0, utils::color::PALE_PURPLE, utils::color::TRANSPARENT);
        press_space.draw_text("SPACE", &tile_set, 0, 1, utils::color::PALE_PURPLE, utils::color::TRANSPARENT);
        press_space.update_texture(&tile_set);
        
        const C1  : Color = rgba(114,159,207,255);
        let C2 : Color = rgba(244,40,40,255);
        
        let graphic_speed = [
            OutlinedTile::new(193, utils::color::ORANGE, &tile_set, &texture_creator),
            OutlinedTile::new(215, utils::color::BRIGHT_GREEN, &tile_set, &texture_creator),
            OutlinedTile::new(405, utils::color::YELLOW, &tile_set, &texture_creator),
        ];
        let graphic_virus = [
            OutlinedTile::new(150, C2, &tile_set, &texture_creator),
            OutlinedTile::new(150, utils::color::YELLOW, &tile_set, &texture_creator),
            OutlinedTile::new(148, utils::color::BRIGHT_GREEN, &tile_set, &texture_creator),
        ];
        let graphic_virus2 = [
            OutlinedTile::new(256, C2, &tile_set, &texture_creator),
            OutlinedTile::new(256, utils::color::YELLOW, &tile_set, &texture_creator),
            OutlinedTile::new(257, utils::color::BRIGHT_GREEN, &tile_set, &texture_creator),
        ];
        let graphic_orphan = [
            OutlinedTile::new(457, C2, &tile_set, &texture_creator),
            OutlinedTile::new(457, utils::color::YELLOW, &tile_set, &texture_creator),
            OutlinedTile::new(457, utils::color::BRIGHT_GREEN, &tile_set, &texture_creator),
        ];
        let graphic_tab_l = [
            OutlinedTile::new(334, C2, &tile_set, &texture_creator),
            OutlinedTile::new(334, utils::color::YELLOW, &tile_set, &texture_creator),
            OutlinedTile::new(334, utils::color::BRIGHT_GREEN, &tile_set, &texture_creator),
        ];
        let graphic_tab_r = [
            OutlinedTile::new(335, C2, &tile_set, &texture_creator),
            OutlinedTile::new(335, utils::color::YELLOW, &tile_set, &texture_creator),
            OutlinedTile::new(335, utils::color::BRIGHT_GREEN, &tile_set, &texture_creator),
        ];
        let graphic_tab_t = [
            OutlinedTile::new(318, C2, &tile_set, &texture_creator),
            OutlinedTile::new(318, utils::color::YELLOW, &tile_set, &texture_creator),
            OutlinedTile::new(318, utils::color::BRIGHT_GREEN, &tile_set, &texture_creator),
        ];

        let graphic_tab_b = [
            OutlinedTile::new(319, C2, &tile_set, &texture_creator),
            OutlinedTile::new(319, utils::color::YELLOW, &tile_set, &texture_creator),
            OutlinedTile::new(319, utils::color::BRIGHT_GREEN, &tile_set, &texture_creator),
        ];
        let graphic_hl = OutlinedTile::new(149,utils::color::WHITE,&tile_set,&texture_creator);
        let graphic_hl2 = OutlinedTile::new(149,utils::color::PALE_PURPLE,&tile_set,&texture_creator);
        BaseDrawingContext {
            tile_set: tile_set,tick:0,
            chrome:chrome,press_space,graphic_virus2,
            graphic_hl,graphic_hl2,graphic_speed,
            graphic_virus,graphic_orphan,graphic_tab_b,graphic_tab_l,graphic_tab_r,graphic_tab_t,
            score: Graphic::blank(6,1).textured(texture_creator),
            top_score: Graphic::blank(6,1).textured(texture_creator),
            level: Graphic::blank(6,1).textured(texture_creator),
        }
    }
    pub fn draw_cell<T:RenderTarget>(&self, canvas: &mut Canvas<T>, p : FieldTile, position:(i32,i32)) {
        let pos = (position.0, position.1);

        match p {
            FieldTile::Virus(c) => (if (self.tick / 4) % 2 == 0 { &self.graphic_virus[c.to_index()] } 
                                               else { &self.graphic_virus2[c.to_index()] }).draw(canvas,pos),
            FieldTile::Orphan(c) => self.graphic_orphan[c.to_index()].draw(canvas,pos),
            FieldTile::CapLeft(c) => self.graphic_tab_l[c.to_index()].draw(canvas,pos),
            FieldTile::CapRight(c) => self.graphic_tab_r[c.to_index()].draw(canvas,pos),
            FieldTile::CapTop(c) => self.graphic_tab_t[c.to_index()].draw(canvas,pos),
            FieldTile::CapBottom(c) => self.graphic_tab_b[c.to_index()].draw(canvas,pos),
        }
    }
    pub fn draw_box<T:RenderTarget>(&self, canvas: &mut Canvas<T>, w: u32, h:u32, pieces: &[FieldTile], offset: (i32,i32), x : i32, y: i32, b: u32) {
        for cy in 0..h as i32 {
            for cx in 0..w as i32 {
                if y + cy >= b as i32 {
                    self.draw_cell(canvas, pieces[(cx as usize + cy as usize) % pieces.len()], (offset.0 + (x + cx) * 10,offset.1 + (y + cy - b as i32) * 10))
                }
            }
        }

    }
    pub fn draw_imprint<T:RenderTarget>(&self, canvas: &mut Canvas<T>, p : &Imprint<FieldTile>, offset: (i32,i32), x : i32, y: i32, b: u32) {
        let (w, h) = p.size();
        for cy in 0..h as i32 {
            for cx in 0..w as i32 {
                if let Cell::Filled(ps) = p[(cx as usize, cy as usize)] {
                    if y + cy >= b as i32 {
                        self.draw_cell(canvas, ps, (offset.0 + (x + cx) * 10,offset.1 + (y + cy - b as i32) * 10))
                    }
                }
            }
        }
    }

    pub fn draw_game<T: RenderTarget>(&mut self, c: &mut Canvas<T>, g: &Game) -> Result<(), String> {
        //self.ctx.draw(c, g)?;
        c.clear();
        self.tick += 1;
        if self.tick > 256 {
            self.tick = 0;
        }
        self.chrome.draw(c,(0,17));
        //let main = &self.ctx.main;
        self.score.draw_rect(0,0,6,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        self.score.draw_text(&g.score().to_string(),&self.tile_set,0,0, WHITE,TRANSPARENT);
        self.top_score.draw_rect(0,0,6,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        self.top_score.draw_text(&g.top_score().to_string(),&self.tile_set,0,0, WHITE,TRANSPARENT);
        self.level.draw_rect(0,0,6,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        self.level.draw_text(&(g.current_level()+1).to_string(),&self.tile_set,0,0, WHITE,TRANSPARENT);
        self.score.update_texture(&self.tile_set);
        self.level.update_texture(&self.tile_set);
        self.top_score.update_texture(&self.tile_set);
        let board_x = 16;
        let board_y = 33;
        self.draw_imprint(c,g.board(), (board_x,board_y), 0, 0,1);
        if let Status::Clearing(n) = g.status {
            for loc in &g.clearing {
                if n % 2 == 0 {
                    self.graphic_hl.draw(c,(board_x + loc.0 as i32 *10, board_y + (loc.1 - 1) as i32 * 10))
                } else {
                    self.graphic_hl2.draw(c,(board_x + loc.0 as i32 *10, board_y + (loc.1 - 1) as i32 * 10))
                }
            }
            
        }
        
        if let Status::Paused = g.status { 
            self.press_space.draw(c, (board_x + 104,board_y+8))
        } else { 
            if let Some(p) = g.next() {
                if let  Status::Lowering(_) = g.status { } else {
                    self.draw_imprint(c,&p,(board_x + 113,board_y+10),0,0,1);
                }
            }
        }
        self.score.draw(c,(board_x + 100, board_y + 68));
        self.top_score.draw(c,(board_x + 100, board_y + 116));
        self.level.draw(c,(board_x + 100, board_y + 164));
        self.graphic_speed[g.config.speed.to_index()].draw(c,(board_x + 140, board_y + 163));
        match g.status {
            Status::Active | Status::Paused | Status::Reacting | Status::Clearing(_) | Status::Falling(_,_) | Status::Infecting(_,_) => {
                self.draw_imprint(
                    c,
                    &g.current.imprint(),
                    (board_x,board_y),
                    g.position.0,
                    g.position.1,
                    1
                );
            }
            Status::Raising(f) => {
                self.draw_imprint(
                    c,
                    &g.current.imprint(),
                    (board_x,board_y),
                    g.position.0,
                    g.position.1,
                    1
                );
                let viruses = &[FieldTile::Virus(PieceColor::Green),FieldTile::Virus(PieceColor::Red),FieldTile::Virus(PieceColor::Yellow)];
                self.draw_box(c,g.board().size().0 as u32,(g.board().size().1 - f) as u32,viruses,(board_x,board_y),0,f as i32,1);
            }
            Status::Lowering(f) => {
                let viruses = &[FieldTile::Virus(PieceColor::Green),FieldTile::Virus(PieceColor::Red),FieldTile::Virus(PieceColor::Yellow)];
                self.draw_box(c,g.board().size().0 as u32,(g.board().size().1 - f) as u32,viruses,(board_x,board_y),0,f as i32,1);
            }
            Status::Menu(_f) => {
                self.press_space.draw(c, (board_x + 104,board_y+8))
            }
        }
        Ok(())
    }
}