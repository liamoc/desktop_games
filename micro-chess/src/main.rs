
extern crate tesserae;
extern crate sdl2;
extern crate rand;
extern crate utils;

use sdl2::rect::Rect;
use littlewing::attack::Attack;
use littlewing::clock::Clock;
use littlewing::search::Search;
use littlewing::piece_move::PieceMove;
use littlewing::piece_move_notation::PieceMoveNotation;
use littlewing::piece_move_generator::PieceMoveGenerator;
use littlewing::fen::FEN;
use utils::menu::{*};

//use std::env;
use tesserae::{Graphic,Tile,TileSet};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::render::{Texture, BlendMode, TextureCreator};
use sdl2::video::Window;
use sdl2::Sdl; 
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use utils::framerate::FPSManager;
use std::io::Cursor;
use utils::color::{*};
use littlewing::piece::{Piece,PieceAttr};
use littlewing::game::Game;
use littlewing::color;

struct GraphicsSet<T> {
    tile_set: TileSet,
    pieces_black: [Graphic<T>;6],
    pieces_shaded: [Graphic<T>;6],
    pieces_white: [Graphic<T>;6],
    table: Graphic<T>,
    coords: Graphic<T>,
    coords_rev: Graphic<T>,
}
impl <'r> GraphicsSet<Texture<'r>> {
    fn piece<T>(texture_creator: &'r TextureCreator<T>, tile_set: &TileSet, col : Color, a : usize, b: usize, c : usize, d: usize) -> Graphic<Texture<'r>> {
        let mut x = Graphic::blank(2, 2).textured(texture_creator);
        x[(0,0)] = Tile { index: a, fg: col, bg: TRANSPARENT };
        x[(1,0)] = Tile { index: b, fg: col, bg: TRANSPARENT };
        x[(0,1)] = Tile { index: c, fg: col, bg: TRANSPARENT };
        x[(1,1)] = Tile { index: d, fg: col, bg: TRANSPARENT };
        x.update_texture(tile_set);
        x
    }
    fn new<T>(texture_creator: &'r TextureCreator<T>) -> GraphicsSet<Texture<'r>> {        
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
        let mut table = Graphic::blank(8*2, 8*2).textured(&texture_creator);
        let mut coords = Graphic::blank(10*2, 10*2).textured(&texture_creator);
        let mut coords_rev = Graphic::blank(10*2, 10*2).textured(&texture_creator);
        let letters = ['a','b','c','d','e','f','g','h'];
        for y in 1..9 {
            coords.draw_text(&y.to_string(), &tile_set, 0, y*2, DARKER_GRAY, TRANSPARENT);
            coords.draw_text(&y.to_string(), &tile_set, 9*2, y*2, DARKER_GRAY, TRANSPARENT);
            coords_rev.draw_text(&y.to_string(), &tile_set, 0, (9-y)*2, DARKER_GRAY, TRANSPARENT);
            coords_rev.draw_text(&y.to_string(), &tile_set, 9*2, (9-y)*2, DARKER_GRAY, TRANSPARENT);
            coords.draw_text(&letters[(y-1) as usize].to_string(), &tile_set, (9-y)*2, 0, DARKER_GRAY, TRANSPARENT);
            coords.draw_text(&letters[(y-1) as usize].to_string(), &tile_set, (9-y)*2, 9*2, DARKER_GRAY, TRANSPARENT);
            coords_rev.draw_text(&letters[(y-1) as usize].to_string(), &tile_set, y*2, 0, DARKER_GRAY, TRANSPARENT);
            coords_rev.draw_text(&letters[(y-1) as usize].to_string(), &tile_set, y*2, 9*2, DARKER_GRAY, TRANSPARENT);
            
        }
        coords.update_texture(&tile_set);
        coords_rev.update_texture(&tile_set);
        let mut white = true;
        for y in 0..8 {
            for x in 0..8 {
                let col = if white {
                    rgba(233,185,110,255)
                } else {
                    rgba(193,125,17,255)
                };
                table[(x*2,y*2)] = Tile { index: 1, fg: col, bg: BLACK };
                table[(x*2+1,y*2)] = Tile { index: 1, fg: col, bg: BLACK };
                table[(x*2,y*2+1)] = Tile { index: 1, fg: col, bg: BLACK };
                table[(x*2+1,y*2+1)] = Tile { index: 1, fg: col, bg: BLACK };
                white = !white;
            }
            white = !white;
        }
        table.update_texture(&tile_set);
        let pieces_black = [
            Self::piece(texture_creator,&tile_set,DARK_CHARCOAL,198,0,214,0),
            Self::piece(texture_creator,&tile_set,DARK_CHARCOAL,180,181,212,213),
            Self::piece(texture_creator,&tile_set,DARK_CHARCOAL,182,183,212,213),
            Self::piece(texture_creator,&tile_set,DARK_CHARCOAL,208,209,216,213),
            Self::piece(texture_creator,&tile_set,DARK_CHARCOAL,194,195,210,211),
            Self::piece(texture_creator,&tile_set,DARK_CHARCOAL,196,197,210,211),
        ];
        let shade = rgba(135,137,133,255);
        let pieces_shaded = [
            Self::piece(texture_creator,&tile_set,shade,198,0,214,0),
            Self::piece(texture_creator,&tile_set,shade,180,181,212,213),
            Self::piece(texture_creator,&tile_set,shade,182,183,212,213),
            Self::piece(texture_creator,&tile_set,shade,208,209,216,213),
            Self::piece(texture_creator,&tile_set,shade,194,195,210,211),
            Self::piece(texture_creator,&tile_set,shade,196,197,210,211),
        ];
        let pieces_white = [
            Self::piece(texture_creator,&tile_set,WHITE,198,0,214,0),
            Self::piece(texture_creator,&tile_set,WHITE,180,181,212,213),
            Self::piece(texture_creator,&tile_set,WHITE,182,183,212,213),
            Self::piece(texture_creator,&tile_set,WHITE,208,209,216,213),
            Self::piece(texture_creator,&tile_set,WHITE,194,195,210,211),
            Self::piece(texture_creator,&tile_set,WHITE,196,197,210,211),
        ];
        GraphicsSet {
            tile_set: tile_set,pieces_black,pieces_white,table,pieces_shaded,
            coords,coords_rev
        }
    }

}

fn draw_piece<'r>(sel : Piece, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>, position : (i32,i32)) {
    let outline_set = &graphics.pieces_black; 
    let main_set = if sel.color() == color::WHITE {
        &graphics.pieces_white
    } else {
        &graphics.pieces_shaded
    };
    let shift = if sel.is_pawn() { 4 } else { 0 };
    let ix = if sel.is_pawn() { 0 }
    else if sel.is_rook() { 1 }
    else if sel.is_bishop() { 2 }
    else if sel.is_knight() { 3 }
    else if sel.is_queen() { 4 }
    else if sel.is_king() { 5 }
    else { 255 };
    outline_set[ix].draw(canvas,(position.0-1+shift,position.1));
    outline_set[ix].draw(canvas,(position.0+1+shift,position.1));
    outline_set[ix].draw(canvas,(position.0-1+shift,position.1+1));
    outline_set[ix].draw(canvas,(position.0+shift,position.1+1));
    outline_set[ix].draw(canvas,(position.0+1+shift,position.1+1));
    outline_set[ix].draw(canvas,(position.0-1+shift,position.1-1));
    outline_set[ix].draw(canvas,(position.0+shift,position.1-1));
    outline_set[ix].draw(canvas,(position.0+1+shift,position.1-1));
    main_set[ix].draw(canvas,(position.0+shift,position.1));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum EndGame {
    Victory (color::Color),
    Stalemate
}

struct Animation {
    time: u32,
    piece: Piece,
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
        draw_piece(self.piece,canvas, graphics, (x,y));
    }
    
}
impl Clone for Table {
    
    fn clone(&self) -> Self { 
        let mut other = Table::new();
        other.game = self.game.clone();
        other.ai_player = self.ai_player.clone();
        other.time_limit = self.time_limit;
        other.search_depth = self.search_depth;
        other.ai_thinking = true;
        other.game_over = self.game_over.clone();
        other
    }
}
pub struct Table {    
    game : Game,
    ai_thinking : bool,
    ai_player : Option<color::Color>,
    time_limit: u64,
    search_depth: i8,
    animations: Vec<(Animation,u32)>,  
    game_over : Option<EndGame>,
    highlights: Option<(usize,usize,usize,usize)>,
    san_history: Vec<String>,
}

const OFFSET : (i32,i32) = (4+12,20+12);

impl Table {
    pub fn can_undo(&self) -> bool {
        if self.animations.len() > 0 { return false; }
        if self.game.history.len() > 1 {
            true
        } else { false }
    }
    pub fn undo(&mut self) {
        if let Some(history) = self.game.history.pop() {
            self.highlights = None;
            self.game.undo_move(history);
            self.san_history.pop();
            if self.ai_player == Some(self.game.side()) {
                if let Some(history) = self.game.history.pop() {
                    self.game.undo_move(history);
                    self.san_history.pop();
                }
            }
        }
    }
    fn animate_move(&mut self, start: (i32,i32), end: (i32,i32), piece: Piece, time: u32, and_then: Box<dyn FnOnce(&mut Table)>) {
        self.animations.push((Animation { time: time, piece: piece, start: start, end:end, and_then: and_then},0))
    }
    pub fn add_piece_to(&mut self, origin:(usize,usize), piece:Piece) {
        self.game.board[origin.1*8 + origin.0] = piece;
    }
    pub fn animate_piece(&mut self, dest:(usize,usize), piece:Piece, start_pos: (i32,i32), and_then: Box<dyn FnOnce(&mut Table)>) {
        let dest_pos = (if !self.is_reversing() { 7-dest.0 } else {dest.0} as i32 * 16 + OFFSET.0, (if self.is_reversing()  {7 - dest.1} else {dest.1}) as i32 * 16 + OFFSET.1);        
        self.animate_move(start_pos, dest_pos, piece, 14, and_then);
    } 
    fn new() -> Table {
        let table = Table {
            animations: Vec::new(),
            ai_thinking: false, search_depth: 8, time_limit:500,
            ai_player: Some(color::BLACK),
            game: Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap(),
            game_over: None, highlights: None,
            san_history: vec![],
        };        
        table
    }

    fn is_reversing(&self) -> bool {
        self.ai_player != Some(color::WHITE) //self.game_mode == 0 // && self.game.side() == color::WHITE
    }
    fn draw<'t>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'t>>) {
        let reversing = self.is_reversing();
        graphics.table.draw(canvas,OFFSET);
        let mut i = 0;
        let mut y = if reversing { 7 } else { 0 }; 
        let step = if reversing { -1 } else {1 };
        let stop = if reversing { 0 } else { 7}; 
        if reversing {
            graphics.coords_rev.draw(canvas,(4,20));
        } else {
            graphics.coords.draw(canvas,(4,20));
        }
        for y in 0..8 {
            for x in 0..8 {
                if let Some((ox,oy,dx,dy)) = self.highlights {
                    if (ox,oy) == (x as usize,y as usize) || (dx,dy) == (x as usize,y as usize) {
                        let yy = if reversing { 7 - y } else { y};
                        let xx = if !reversing { 7 - x } else { x};
                        canvas.set_draw_color(rgba(150,150,255,128));
                        canvas.fill_rect(Rect::new(xx*16+OFFSET.0,yy*16+OFFSET.1,16,16)).unwrap();
                    }
                }
            }
        }
        loop {
            for x in 0.. 8 {
                if self.game.board[i] != 0 {
                    let xx = if !self.is_reversing() {
                        7 - x
                    } else {
                        x
                    };
                    draw_piece(self.game.board[i], canvas, graphics, (xx * 16+OFFSET.0, y * 16 + OFFSET.1));
                }
                i += 1;
            }
            if y == stop {
                break;
            } else {
                y += step;
            }
        }
        for (x,i) in &self.animations {
            x.draw(canvas,graphics, *i)
        }
    }

    fn tick(&mut self) -> Vec<(Animation,u32)> {
        for (_,i) in &mut self.animations {
            *i+=1;            
        }
        let (done, v): (Vec<_>, Vec<_>) = self.animations.drain(..).partition(|(x,i)| *i >= x.time );
        self.animations = v;
        done
    }
    fn can_pick_up(&self, x: usize, y: usize) -> bool {
        if self.animations.len() > 0 { 
            return false
        }
        if self.ai_player == Some(self.game.side()) { return false }
        if self.game.board[y*8+x] != 0 {
            self.game.board[y*8+x].color() == self.game.side()
        } else {
            false
        }
    }
    fn pick_up(&mut self,x: usize, y: usize) -> Piece {
        let ret = self.game.board[y * 8 + x];
        self.game.board[y*8+x] = 0;
        ret
    }
    fn generate_lan(&self, origin: (usize,usize), dest: (usize,usize), piece: Piece, knight_promote : bool ) -> String {
        let letters = ['a','b','c','d','e','f','g','h'];
        let extra = if piece.is_pawn() {
            if dest.1 == 0 || dest.1 == 7 {
                if knight_promote { "n"} else { "q" }
            } else { "" }
        } else { "" };
        letters[origin.0].to_string() + &(origin.1 + 1).to_string() + &letters[dest.0].to_string() + &(dest.1 + 1).to_string() + &extra.to_string() // todo knight promotions
    }
    fn can_make_move(&mut self, origin: (usize,usize), dest: (usize,usize), piece: Piece, knight_promote: bool) -> Option<String> {
        self.game.board[(origin.1 * 8 + origin.0)] = piece;
        let lan = self.generate_lan(origin,dest,piece,knight_promote);
        let moves = self.game.get_moves();
        let my_move = self.game.move_from_lan(&lan);
        for m in moves {
            if m == my_move {
                self.game.board[(origin.1 * 8 + origin.0)] = 0;
                return Some(lan)
            }
        }
        self.game.board[(origin.1 * 8 + origin.0)] = 0;
        
        None
    }
    fn find_ai_move(mut game : Game, time_limit : u64, search_depth : i8 ) -> Option<PieceMove> {
        game.clock = Clock::new(1, time_limit);        
        let mut r = None;
        while r.is_none() {
            r = game.search(1..search_depth);
        }
        r
    }
    fn make_ai_move(&mut self, m : PieceMove) {
        let a = m.from(); 
        let b = m.to();
        let origin = ((a % 8) as usize, (a / 8) as usize);
        let dest = ((b % 8) as usize, (b / 8) as usize);
        let start_pos = ((if !self.is_reversing() { 7 - origin.0 } else {origin.0}) as i32*16 + OFFSET.0, (if self.is_reversing() { 7 - origin.1 as i32 } else { origin.1 as i32 }) * 16 + OFFSET.1);
        let piece = self.game.board[a as usize]; 
        self.game.board[a as usize] = 0;
        self.animate_piece(dest, piece, start_pos, Box::new(move |tbl| {
            tbl.game.board[a as usize] = piece; 
            tbl.san_history.push(tbl.game.move_to_san(m));
            tbl.game.make_move(m);
            tbl.game.history.push(m);
            tbl.highlights = Some((origin.0, origin.1,dest.0,dest.1));
            tbl.check_mate(); 
        }));                
    }
    fn make_move(&mut self, origin: (usize,usize), dest: (usize,usize), piece: Piece, lan: String) {
        self.game.board[(origin.1 * 8 + origin.0)] = piece;
        let my_move = self.game.move_from_lan(&lan);
        self.san_history.push(self.game.move_to_san(my_move));
        self.game.make_move(my_move);
        self.game.history.push(my_move);
        self.highlights = Some((origin.0, origin.1,dest.0,dest.1));
        self.check_mate();
    }
    fn check_mate(&mut self) {
        if self.game.is_mate() {
            if self.game.is_check(self.game.side()) {
                self.game_over = Some(EndGame::Victory(if self.game.side() == color::WHITE { color::BLACK} else {color::WHITE}));
            } else {
                self.game_over = Some(EndGame::Stalemate)
            }
        }
    }
    fn collides(&self, position: (i32,i32)) -> Option<(Piece,usize, usize,(i32,i32))> {
        let x = (position.0 - OFFSET.0) / 16;
        let y = (position.1 - OFFSET.1) / 16;
        if x >= 0 && x < 8 && y >= 0 && y < 8 {
            let offset_x = (x * 16 + OFFSET.0) - position.0;
            let offset_y = (y * 16 + OFFSET.1) - position.1;
            let yy = if self.is_reversing() { 7 - y } else { y };
            let xx = if !self.is_reversing() { 7 - x } else { x };
            Some((self.game.board[(yy*8+x) as usize],xx as usize, yy as usize, (offset_x,offset_y)))
        } else {
            None
        }
    }
}

use std::sync::mpsc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameObject {
    Well(usize)
}
const WIDTH : u32 = 228;
const HEIGHT : u32 = 196;
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
    let mut mx = 0;
    let mut my = 0;
    let mut md = false;
    let mut moves_gfx = Graphic::blank(6,20).textured(&texture_creator);
    let mut status_gfx = Graphic::blank(25,1).textured(&texture_creator);
    let mut attached : Option<(Piece, (usize,usize))> = None;
    let chan = mpsc::channel();
    #[allow(unused_assignments)]
    let mut tx = chan.0;
    let mut rx = chan.1;
    let mut grab_offset : (i32,i32) = (0,0);    
    let mut menu = MenuBar::new(WIDTH)
                    .add(Menu::new("GAME",152,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Easy AI Black", 352, Keycode::F1,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Normal AI Black", 353, Keycode::F2,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Hard AI Black", 354, Keycode::F3,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(136, &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Easy AI White", 355, Keycode::F4,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Normal AI White", 356, Keycode::F5,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Hard AI White", 357, Keycode::F6,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(136, &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Two Player", 358, Keycode::F7,&texture_creator,&graphics_set.tile_set))
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
    let mut shift_down = false;
    loop {
        canvas.set_draw_color(CHARCOAL);
        canvas.clear();
        table.draw(&mut canvas, &graphics_set);
        if let Some((piece,_)) = &attached {
            draw_piece(*piece, &mut canvas,&graphics_set, (mx + grab_offset.0,my + grab_offset.1));
            //cards.draw(&mut canvas, &graphics_set, (mx + grab_offset.0,my + grab_offset.1));
        }



        let won = table.game_over.is_some();
        if table.ai_player == Some(table.game.side()) && table.animations.len() == 0 && !table.ai_thinking && !won {
            let x = table.game.clone();
            let chan = mpsc::channel();
            tx = chan.0; 
            rx = chan.1;
            let tl = table.time_limit;
            let sd = table.search_depth;
            let builder = std::thread::Builder::new().
                name("ai_thinking".to_string()).
                stack_size(4 << 20);

            builder.spawn(move || {
                tx.send(Table::find_ai_move(x,tl,sd)).unwrap();
            }).unwrap();
            table.ai_thinking = true;
        }
        moves_gfx.draw_rect(0, 0, 6, 20, Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:0});
        let mut col = table.game.side();
        for i in 0..20.min(table.game.history.len()) {
            if col == color::WHITE { col = color::BLACK } else { col = color::WHITE};
            moves_gfx.draw_text(&table.san_history[table.game.history.len() - 1 - i],
            &graphics_set.tile_set, 0, i as u32, if col == color::WHITE { WHITE } else { DARKER_GRAY }, TRANSPARENT);
        }
        moves_gfx.update_texture(&graphics_set.tile_set);
        status_gfx.draw_rect(0,0,25,1, Tile {fg: TRANSPARENT, bg: TRANSPARENT, index: 0});
        let mut sx = 0;
        if let Some(e) = table.game_over {
            match e {
            EndGame::Victory(s) => {
                status_gfx.draw_text(if s == color::WHITE { "WHITE" } else { "BLACK" }, &graphics_set.tile_set, 0, 0, if s == color::WHITE { WHITE } else { DARKER_GRAY }, TRANSPARENT );
                status_gfx.draw_text("wins!",&graphics_set.tile_set, 6,0, PALE_ORANGE, TRANSPARENT);
            }, 
            EndGame::Stalemate => {
                status_gfx.draw_text("Stalemate!",&graphics_set.tile_set, 6,0, TEAL, TRANSPARENT);
            }
            }
        } else {
            if table.game.is_check(table.game.side()) {
                status_gfx.draw_text("Check! ",&graphics_set.tile_set,0,0,YELLOW,TRANSPARENT);
                sx += 7;
            }
            let s = table.game.side();
            status_gfx.draw_text(if s == color::WHITE { "WHITE" } else { "BLACK" }, &graphics_set.tile_set, sx, 0, if s == color::WHITE { WHITE } else { DARKER_GRAY }, TRANSPARENT );
            if table.ai_thinking {
                status_gfx.draw_text(" is thinking...", &graphics_set.tile_set, sx+5, 0, PALE_BLUE, TRANSPARENT);
            } else if table.animations.len() > 0 {
                status_gfx.draw_text(" is moving...", &graphics_set.tile_set, sx+5, 0, PALE_BLUE, TRANSPARENT);
            } else {
                status_gfx.draw_text(" to move.", &graphics_set.tile_set, sx+5, 0, PALE_ORANGE, TRANSPARENT);
            }
        
        }
        canvas.set_draw_color(DARK_CHARCOAL);
        canvas.fill_rect(Rect::new(10*16, 0,160,HEIGHT)).unwrap();
        canvas.fill_rect(Rect::new(0, HEIGHT as i32-20,WIDTH,20)).unwrap();
        status_gfx.update_texture(&graphics_set.tile_set);
        moves_gfx.draw(&mut canvas, (10*16+OFFSET.0,OFFSET.1-14));
        status_gfx.draw(&mut canvas, (OFFSET.0-9,HEIGHT as i32-15));
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
            let old_ai_thinking = table.ai_thinking;
            while !event_happened {
                if table.ai_thinking {
                    if let Ok(Some(r)) = rx.try_recv() {
                        table.make_ai_move(r);
                        table.ai_thinking = false;
                    }
                } 
                if table.ai_thinking != old_ai_thinking {
                    event_happened = true;
                }
                for event in event_pump.poll_iter() {
                    event_happened = true;
                    let h = menu.handle_event(event.clone(), &mut event_subsystem);
                    match event {
                        _ if h => {},
                        Event::Quit {..} |
                        Event::KeyDown { keycode: Some(Keycode::F12), .. } => {
                            return;
                        },
                        Event::KeyDown { keycode: Some(Keycode::F1), ..} => {
                            table = Table::new();
                            attached = None; 
                            md = false;
                            table.time_limit = 10;
                            table.search_depth = 2;
                            table.ai_player = Some(color::BLACK);
                        },
                        Event::KeyDown { keycode: Some(Keycode::F2), ..} => {
                            table = Table::new();
                            attached = None; 
                            md = false;
                            table.time_limit = 500;
                            table.search_depth = 8;
                            table.ai_player = Some(color::BLACK);
                        },
                        Event::KeyDown { keycode: Some(Keycode::F3), ..} => {
                            table = Table::new();
                            attached = None; 
                            md = false;
                            table.time_limit = 2000;
                            table.search_depth = 15;
                            table.ai_player = Some(color::BLACK);
                        },
                        Event::KeyDown { keycode: Some(Keycode::LShift), ..} => {
                            shift_down = true;
                        }
                        Event::KeyUp { keycode: Some(Keycode::LShift), ..} => {
                            shift_down = false;
                        }
                        Event::KeyDown { keycode: Some(Keycode::RShift), ..} => {
                            shift_down = true;
                        }
                        Event::KeyUp { keycode: Some(Keycode::RShift), ..} => {
                            shift_down = false;
                        }
                        Event::KeyDown { keycode: Some(Keycode::F4), ..} => {
                            table = Table::new();
                            attached = None; 
                            md = false;
                            table.time_limit = 10;
                            table.search_depth = 2;
                            table.ai_player = Some(color::WHITE);
                        },
                        Event::KeyDown { keycode: Some(Keycode::F5), ..} => {
                            table = Table::new();
                            attached = None; 
                            md = false;
                            table.time_limit = 500;
                            table.search_depth = 8;
                            table.ai_player = Some(color::WHITE);
                        },
                        Event::KeyDown { keycode: Some(Keycode::F6), ..} => {
                            table = Table::new();
                            attached = None; 
                            md = false;
                            table.time_limit = 2000;
                            table.search_depth = 15;
                            table.ai_player = Some(color::WHITE);
                        },
                        Event::KeyDown { keycode: Some(Keycode::F7), ..} => {
                            table = Table::new();
                            attached = None; 
                            md = false;
                            table.ai_player = None;
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
                            let ai = table.ai_player;
                            let sd = table.search_depth;
                            let tl = table.time_limit;
                            table = Table::new();
                            table.ai_player = ai;
                            table.search_depth = sd;
                            table.time_limit = tl;
                            attached = None;
                            md = false;
                        },
                        Event::MouseButtonUp { ..} if won => {
                            let ai = table.ai_player;
                            let sd = table.search_depth;
                            let tl = table.time_limit;
                            table = Table::new();
                            table.ai_player = ai;
                            table.search_depth = sd;
                            table.time_limit = tl;
                            attached = None;
                            md = false;
                        },
                        Event::KeyDown { keycode: Some(Keycode::Z), ..} => {
                            table.undo()
                        },
                        Event::KeyDown { keycode: Some(Keycode::N), ..} => {
                            let ai = table.ai_player;
                            let sd = table.search_depth;
                            let tl = table.time_limit;
                            table = Table::new();
                            table.ai_player = ai;
                            table.search_depth = sd;
                            table.time_limit = tl;
                            attached = None;
                            md = false;
                        },
                        Event::MouseButtonDown { mouse_btn: _, x, y, ..} if !won => {
                            let (sx,sy) = (x,y);//(x * dw as i32 / ww as i32, y * dh as i32 / wh as i32);
                            md = true;
                            mx = sx; my = sy;
                            if let Some((_,bx,by, offset)) = table.collides((mx,my)) {
                                if table.can_pick_up(bx,by) {
                                    attached = Some((table.pick_up(bx,by), (bx,by)));
                                    grab_offset = offset;
                                }
                                //}
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
                            if let Some((piece,origin)) = attached {
                                if !md {
                                    if let Some((_,bx,by,_)) = table.collides((mx,my)) {
                                        if origin != (bx,by) {
                                            if let Some(lan) = table.can_make_move(origin,(bx,by),piece,shift_down) {
                                                placed = true;
                                                table.make_move(origin,(bx,by),piece,lan)
                                            }
                                        }                                        
                                    }
                                } 
                                if !placed {
                                    if md {
                                        md = false;
                                        table.add_piece_to(origin,piece);
                                    } else {
                                        table.animate_piece(origin, piece, (mx + grab_offset.0,my + grab_offset.1),Box::new(move |tbl| { tbl.add_piece_to(origin,piece)}));
                                    }
                                }
                                attached = None;
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



fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("micro chess", 320, 240)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    main_loop(window,&sdl_context);
    
    //cards::main_loop::<Spider<OneSuit>>();
    
}