use ::imprint::{Cell,Imprint};
use game::{Tetris, Status};
use game::piece::{PieceShape};

use sdl2::render::RenderTarget;
use sdl2::render::{Canvas,Texture};
use tesserae::{*};
use std::io::Cursor;
use sdl2::render::TextureCreator;
use utils::color::{*};
pub struct BaseDrawingContext<'r> {
    pub tile_set : TileSet,
    chrome: Graphic<Texture<'r>>, 
    graphic_i : Graphic<Texture<'r>>,
    graphic_j : Graphic<Texture<'r>>,
    graphic_l : Graphic<Texture<'r>>,
    graphic_o : Graphic<Texture<'r>>,
    graphic_s : Graphic<Texture<'r>>,
    graphic_z : Graphic<Texture<'r>>,
    graphic_t : Graphic<Texture<'r>>,
    graphic_hl : Graphic<Texture<'r>>,
    graphic_garbage : Graphic<Texture<'r>>,
    score: Graphic<Texture<'r>>,
    top_score: Graphic<Texture<'r>>,
    level: Graphic<Texture<'r>>,
}
impl <'r> BaseDrawingContext<'r> {
    pub fn new<T>(texture_creator: &'r TextureCreator<T>) -> BaseDrawingContext<'r> {
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
        let mut graphic_garbage = Graphic::load_from(Cursor::new(&include_bytes!("../block_garbage")[..])).unwrap().textured(texture_creator);
        graphic_garbage.update_texture(&tile_set);
        let mut chrome = Graphic::load_from(Cursor::new(&include_bytes!("../chrome")[..])).unwrap().textured(texture_creator);
        chrome.update_texture(&tile_set);
        let mut graphic_i = Graphic::load_from(Cursor::new(&include_bytes!("../block_i")[..])).unwrap().textured(texture_creator);
        graphic_i.update_texture(&tile_set);
        let mut graphic_j = Graphic::load_from(Cursor::new(&include_bytes!("../block_j")[..])).unwrap().textured(texture_creator);
        graphic_j.update_texture(&tile_set);
        let mut graphic_l = Graphic::load_from(Cursor::new(&include_bytes!("../block_l")[..])).unwrap().textured(texture_creator);
        graphic_l.update_texture(&tile_set);
        let mut graphic_o = Graphic::load_from(Cursor::new(&include_bytes!("../block_o")[..])).unwrap().textured(texture_creator);
        graphic_o.update_texture(&tile_set);
        let mut graphic_s = Graphic::load_from(Cursor::new(&include_bytes!("../block_s")[..])).unwrap().textured(texture_creator);
        graphic_s.update_texture(&tile_set);
        let mut graphic_z = Graphic::load_from(Cursor::new(&include_bytes!("../block_z")[..])).unwrap().textured(texture_creator);
        graphic_z.update_texture(&tile_set);
        let mut graphic_t = Graphic::load_from(Cursor::new(&include_bytes!("../block_t")[..])).unwrap().textured(texture_creator);
        graphic_t.update_texture(&tile_set);
        let mut graphic_hl = Graphic::load_from(Cursor::new(&include_bytes!("../block_hl")[..])).unwrap().textured(texture_creator);
        graphic_hl.update_texture(&tile_set);
        BaseDrawingContext {
            tile_set: tile_set,
            chrome:chrome,
            graphic_garbage: graphic_garbage,
            graphic_i: graphic_i,
            graphic_j: graphic_j,
            graphic_l: graphic_l,
            graphic_o: graphic_o,
            graphic_s: graphic_s,
            graphic_z: graphic_z,
            graphic_t: graphic_t,
            graphic_hl:graphic_hl,
            score: Graphic::blank(6,1).textured(texture_creator),
            top_score: Graphic::blank(6,1).textured(texture_creator),
            level: Graphic::blank(6,1).textured(texture_creator),
        }
    }
    pub fn draw_cell<T:RenderTarget>(&self, canvas: &mut Canvas<T>, p : PieceShape, highlight:bool, position:(i32,i32)) {
        let pos = (position.0-6, position.1-6);
        match p {
            _ if highlight => self.graphic_hl.draw(canvas,pos),
            PieceShape::Garbage => self.graphic_garbage.draw(canvas,pos),
            PieceShape::I => self.graphic_i.draw(canvas,pos),
            PieceShape::J => self.graphic_j.draw(canvas,pos),
            PieceShape::L => self.graphic_l.draw(canvas,pos),
            PieceShape::O => self.graphic_o.draw(canvas,pos),
            PieceShape::S => self.graphic_s.draw(canvas,pos),
            PieceShape::Z => self.graphic_z.draw(canvas,pos),
            PieceShape::T => self.graphic_t.draw(canvas,pos),
        }
    }
    pub fn draw_box<T:RenderTarget>(&self, canvas: &mut Canvas<T>, w: u32, h:u32, piece: PieceShape, offset: (i32,i32), highlight: bool, x : i32, y: i32, b: u32) {
        for cy in 0..h as i32 {
            for cx in 0..w as i32 {
                if y + cy >= b as i32 {
                    self.draw_cell(canvas, piece, highlight, (offset.0 + (x + cx) * 12,offset.1 + (y + cy - b as i32) * 12))
                }
            }
        }

    }
    pub fn draw_imprint<T:RenderTarget>(&self, canvas: &mut Canvas<T>, p : &Imprint<PieceShape>, offset: (i32,i32), highlight: bool, x : i32, y: i32, b: u32) {
        let (w, h) = p.size();
        for cy in 0..h as i32 {
            for cx in 0..w as i32 {
                if let Cell::Filled(ps) = p[(cx as usize, cy as usize)] {
                    if y + cy >= b as i32 {
                        self.draw_cell(canvas, ps, highlight, (offset.0 + (x + cx) * 12,offset.1 + (y + cy - b as i32) * 12))
                    }
                }
            }
        }
    }

    pub fn draw_game<T: RenderTarget>(&mut self, c: &mut Canvas<T>, g: &Tetris) -> Result<(), String> {
        //self.ctx.draw(c, g)?;
        c.clear();
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
        self.draw_imprint(c,g.board(), (board_x,board_y), g.status.is_inactive(), 0, 0,2);
        if let Some(p) = g.next() {
            if let  Status::Lowering(_) = g.status { } else {
                self.draw_imprint(c,p,(board_x + 140,board_y+4),false,0,0,1);
            }
        }
        self.score.draw(c,(board_x + 140, board_y + 68));
        self.top_score.draw(c,(board_x + 140, board_y + 116));
        self.level.draw(c,(board_x + 140, board_y + 164));
        match g.status {
            Status::Active | Status::Paused => {
                self.draw_imprint(
                    c,
                    &g.current.imprint(),
                    (board_x,board_y),
                    g.status.is_inactive(),
                    g.position.0,
                    g.position.1,
                    2
                );
            }
            Status::Raising(f) => {
                self.draw_imprint(
                    c,
                    &g.current.imprint(),
                    (board_x,board_y),
                    g.status.is_inactive(),
                    g.position.0,
                    g.position.1,
                    2
                );
                self.draw_box(c,g.board().size().0 as u32,(g.board().size().1 - f) as u32,PieceShape::Garbage,(board_x,board_y),false,0,f as i32,2);
            }
            Status::Lowering(f) => {
                self.draw_box(c,g.board().size().0 as u32,(g.board().size().1 - f) as u32,PieceShape::Garbage,(board_x,board_y),false,0,f as i32,2);
            }
            Status::Menu(f) => {
                let o = (f / 2) as i32 - g.board().size().0 as i32;
                let points = [
                    (0, 2),
                    (0, 3),
                    (0, 4),
                    (0, 6),
                    (2, 2),
                    (2, 4),
                    (2, 5),
                    (2, 6),
                    (1, 2),
                    (1, 4),
                    (1, 6),
                    (4, 2),
                    (4, 3),
                    (4, 4),
                    (4, 5),
                    (4, 6),
                    (5, 2),
                    (5, 6),
                    (6, 2),
                    (6, 3),
                    (6, 4),
                    (6, 5),
                    (6, 6),
                    (8, 2),
                    (8, 3),
                    (8, 4),
                    (9, 4),
                    (9, 5),
                    (9, 6),
                    (10, 2),
                    (10, 3),
                    (10, 4),
                    (12, 2),
                    (13, 6),
                    (12, 3),
                    (12, 4),
                    (12, 5),
                    (12, 6),
                    (14, 2),
                    (14, 3),
                    (14, 4),
                    (14, 5),
                    (14, 6),
                    (16, 2),
                    (16, 5),
                    (16, 6),
                    (17, 2),
                    (17, 6),
                    (18, 2),
                    (17, 4),
                   // (16, 4),
                   // (18, 4),
                    (18, 3),
                    (18, 6),
                    (20, 2),
                    (20, 3),
                    (20, 4),
                    (20, 6),
                ];
                for &(x, y) in points.iter() {
                    if x >= o && x - o < g.board().size().0 as i32 {
                        let ps = match x {
                            0..=3 => PieceShape::Z,
                            3..=7 => PieceShape::J,
                            8..=11 => PieceShape::T,
                            12..=15 => PieceShape::S,
                            16..=19 => PieceShape::L,
                            20..=22 => PieceShape::O,
                            _ => PieceShape::Garbage
                        };
                        self.draw_box(c, 1,1, ps, (board_x,board_y), false, x - o, y,2);
                    }
                }
            }

            Status::Clearing(f) => {
                for y in &g.lines {
                    self.draw_box(c, g.board().size().0 as u32, 1, PieceShape::Garbage, (board_x,board_y), f % 2 == 0, 0, *y as i32, 2);
                }
            }

            Status::Placing(p, x, y) => {
               // c.set_draw_color(HI_COLOR);
                self.draw_imprint(c, &p.imprint(), (board_x,board_y),true, x, y,2);
            }
        }
        Ok(())
    }
}