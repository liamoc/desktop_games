extern crate tesserae;
extern crate sdl2;
extern crate utils;
extern crate rand;
use std::ops::{Index,IndexMut};
use rand::Rng;
use tesserae::{Graphic,Tile,TileSet};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::keyboard::Keycode;
use sdl2::gfx::framerate::FPSManager;
use std::io::Cursor;
use std::env;
use std::time::Instant;
use utils::menu::{*};
const fn rgba(r:u8,g:u8,b:u8,a:u8) -> Color {
    Color { r:r, g:g, b:b, a:a}
}
const REVEAL_BG : Color = rgba(183,146,103,255);
const WHITE : Color = rgba(255,255,255,255);
const BLUE : Color = rgba(32,74,135,255);
const GREEN : Color = rgba(78,108,6,255);
const PURPLE : Color = rgba(92,52,102,255);
const TEAL : Color = rgba(27,128,120,255);
const CRIMSON : Color = rgba(141,0,0,255);
const AMBER : Color = rgba(159,76,0,255);
const BROWN : Color = rgba(119,90,5,255);
const CHARCOAL : Color = rgba(13,46,52,255);
const YELLOW : Color = rgba(252,233,39,255);
const UNREVEAL_BG : Color = rgba(193,125,17,255);
const EDGE_HI : Color = rgba(203,153,72,255);
const EDGE_LO : Color = rgba(169,109,0,255);
const UI_DARK : Color = rgba(85,87,83,255);
const UI_LIGHT : Color = rgba(176,179,172,255);
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum CellStatus {
    Hidden, Revealed, Flagged
}

#[derive(Copy, Clone, Debug)]
struct Cell {
    count: u32,
    mine: bool, 
    status: CellStatus
}

#[derive(Clone, Debug)]
struct Game {
    width: u32,
    height: u32,
    num_mines: u32,
    board: Vec<Cell>,
    populated: bool,
    num_flags: i32,
}
impl Index<(u32,u32)> for Game {
    type Output=Cell;
    fn index(&self, index: (u32,u32)) -> &Cell {
        &self.board[(index.0 + index.1 * self.width) as usize]
    }
}
impl IndexMut<(u32,u32)> for Game {
    fn index_mut(&mut self, index: (u32,u32)) -> &mut Cell {
        &mut self.board[(index.0 + index.1 * self.width) as usize]
    }
}
impl Game {
    fn new(width : u32, height: u32, num_mines: u32) -> Game {
        let mut board = Vec::new();
        let cell = Cell { count: 0, mine: false, status: CellStatus::Hidden};
        for _ in 0..(width*height) {
            board.push(cell)
        }        
        Game {
            width: width,
            height: height,
            num_mines: num_mines,
            board: board,
            populated: false,
            num_flags: num_mines as i32,
        }
    }
    fn surrounds(&self, position: (u32,u32)) -> Vec<(u32,u32)> {
        let (x,y) = position; 
        let mut ret = Vec::new();
        if x > 0 {
            if y > 0 { ret.push((x-1, y-1))}
            ret.push((x-1, y));
            if y < self.height-1 { ret.push((x-1, y+1)) }
        }
        if x < self.width-1 {
            if y > 0 { ret.push((x+1, y-1))}
            ret.push((x+1, y));
            if y < self.height-1 { ret.push((x+1, y+1)) }
        }
        if y > 0 { ret.push((x, y-1))}
        if y < self.height-1 { ret.push((x, y+1)) }
        ret
    }
    fn populate_board(&mut self, safe_spot: (u32,u32)) {
        let mut rng = rand::thread_rng();
        for _ in 0..self.num_mines {
            loop {
                let x = rng.gen_range(0, self.width);
                let y = rng.gen_range(0, self.height);
                if (x,y) != safe_spot && !self[(x,y)].mine {
                    self[(x,y)].mine = true;
                    for p in self.surrounds((x,y)) { 
                        self[p].count += 1;
                    }
                    break;
                }                
            }            
        }
        self.populated = true;
    }
    fn step_on(&mut self, position: (u32,u32)) {
        if !self.populated {
            self.populate_board(position)            
        }         
        if self[position].status == CellStatus::Hidden {
            self[position].status = CellStatus::Revealed;
            if self[position].count == 0 {
                for i in self.surrounds(position) {
                    self.step_on(i)
                }
            }
        } else if self[position].status == CellStatus::Revealed 
               && self.surrounds(position).iter()
                    .filter(|x| self[**x].status == CellStatus::Flagged 
                            || (self[**x].mine && self[**x].status == CellStatus::Revealed) )
                    .count() == self[position].count as usize {
                for i in self.surrounds(position) {
                    if self[i].status == CellStatus::Hidden { self.step_on(i) }
                }
        }
    }
    fn toggle_flag(&mut self, position: (u32,u32)) { 
        match self[position].status {
            CellStatus::Flagged => 
                { self[position].status = CellStatus::Hidden; self.num_flags += 1}, 
            CellStatus::Hidden => 
                { self[position].status = CellStatus::Flagged; self.num_flags -= 1},
            CellStatus::Revealed => {
                let flags_and_revealed_mines = self.surrounds(position).iter()
                    .filter(|x| self[**x].status == CellStatus::Flagged 
                        || (self[**x].mine && self[**x].status == CellStatus::Revealed) )
                    .count();
                let hiddens = self.surrounds(position).iter()
                    .filter(|x| self[**x].status == CellStatus::Hidden).count();                    
                if flags_and_revealed_mines + hiddens == self[position].count as usize {
                    for i in self.surrounds(position) {
                        if self[i].status == CellStatus::Hidden { self[i].status = CellStatus::Flagged; self.num_flags -= 1};
                    }
                }
            },             
        }
    }
    fn revealed_mines(&self) -> u32 {
        let mut c = 0;
        for y in 0..self.height { 
            for x in 0..self.width {
                if self[(x,y)].status == CellStatus::Revealed && self[(x,y)].mine {
                    c += 1
                }
            }
        }
        c
    }
    fn unrevealed_blanks(&self) -> u32 {
        let mut c = 0;
        for y in 0..self.height { 
            for x in 0..self.width {
                if self[(x,y)].status != CellStatus::Revealed && !self[(x,y)].mine {
                    c += 1
                }
            }
        }
        c
    }
    fn tile_for(cell : Cell) -> Tile {
        match cell.status {
            CellStatus::Revealed => {
                if cell.mine {
                    Tile { index: 77, fg: YELLOW, bg: CRIMSON}
                } else { match cell.count {
                        0 => Tile { index: 0, fg: WHITE, bg: REVEAL_BG},
                        1 => Tile { index: 52, fg: BLUE, bg: REVEAL_BG},
                        2 => Tile { index: 53, fg: GREEN, bg: REVEAL_BG},
                        3 => Tile { index: 54, fg: PURPLE, bg: REVEAL_BG},
                        4 => Tile { index: 55, fg: TEAL, bg: REVEAL_BG},
                        5 => Tile { index: 56, fg: CRIMSON, bg: REVEAL_BG},
                        6 => Tile { index: 57, fg: AMBER, bg: REVEAL_BG},
                        7 => Tile { index: 58, fg: BROWN, bg: REVEAL_BG},
                        8 => Tile { index: 59, fg: CHARCOAL, bg: REVEAL_BG},
                        _ => Tile { index: 77, fg: YELLOW, bg: REVEAL_BG}
                    } 
                }
            }
            CellStatus::Flagged => Self::FLAG,
            CellStatus::Hidden => Self::UNREVEAL_BG_FILL
        }
    }
    fn tile_arrangement(&self, position:(u32,u32)) -> u8 {
        let mut ret = 0;
        let (x,y) = position;
        if y > 0 {
            if x > 0 && self[(x-1,y-1)].status != CellStatus::Revealed {
                ret |= 0b1000;    
            }             
            if x < self.width && self[(x,y-1)].status != CellStatus::Revealed {
                ret |= 0b0100;    
            }
        }
        if y < self.height {
            if x > 0 && self[(x-1,y)].status != CellStatus::Revealed {
                ret |= 0b0010;    
            }
            if x < self.width && self[(x,y)].status != CellStatus::Revealed {
                ret |= 0b0001;
            }
        }
        ret
    }
    const TL_OUTER_CORNER : Tile = Tile { index: 302, fg: REVEAL_BG, bg:EDGE_HI};
    const TR_OUTER_CORNER : Tile = Tile { index: 303, fg: REVEAL_BG, bg:EDGE_LO};
    const BL_OUTER_CORNER : Tile = Tile { index: 287, fg: REVEAL_BG, bg:EDGE_HI};
    const BR_OUTER_CORNER : Tile = Tile { index: 286, fg: REVEAL_BG, bg:EDGE_LO};
    const TL_INNER_CORNER : Tile = Tile { index: 286, fg: UNREVEAL_BG , bg:EDGE_HI};
    const TR_INNER_CORNER : Tile = Tile { index: 287, fg: UNREVEAL_BG , bg:EDGE_LO};
    const BL_INNER_CORNER : Tile = Tile { index: 303, fg: UNREVEAL_BG , bg:EDGE_HI};
    const BR_INNER_CORNER : Tile = Tile { index: 302, fg: UNREVEAL_BG , bg:EDGE_LO};
    const BT_DIAG : Tile = Tile { index: 206, fg: EDGE_LO, bg:EDGE_HI};
    const TB_DIAG : Tile = Tile { index: 206, fg: EDGE_HI, bg:EDGE_LO};
    const EDGE_HI_FILL : Tile = Tile { index: 0, fg: REVEAL_BG, bg:EDGE_HI};
    const EDGE_LO_FILL : Tile = Tile { index: 0, fg: REVEAL_BG, bg:EDGE_LO};
    const UNREVEAL_BG_FILL : Tile = Tile { index: 0, fg: REVEAL_BG, bg:UNREVEAL_BG};
    const REVEAL_BG_FILL : Tile = Tile { index: 0, fg: REVEAL_BG, bg:REVEAL_BG};
    const FLAG : Tile = Tile { index: 112, fg: WHITE, bg: UNREVEAL_BG};

    fn draw_t_shape<T>(board: &mut Graphic<T>, position : (u32, u32), tiles: [Tile;5]) {
        let (offset_x,offset_y) = position;
        if offset_y > 0 { board.set_tile(offset_x , offset_y-1, tiles[0]) };
        if offset_x > 0 { board.set_tile(offset_x-1 , offset_y, tiles[1]) };
        board.set_tile(offset_x , offset_y, tiles[2]);
        board.set_tile(offset_x+1 , offset_y, tiles[3]);
        board.set_tile(offset_x , offset_y+1, tiles[4]);

    }
    fn draw<T>(&self, board: &mut Graphic<T>, chrome: &mut Graphic<T>, tile_set: &TileSet) {
        chrome.draw_rect(0,0,self.width*2+2, 1, Tile{index:0,fg:UI_DARK, bg:UI_LIGHT} );
        chrome.set_tile(1, 0, Tile{index:112,fg:CRIMSON, bg:UI_LIGHT});
        chrome.draw_text(&(self.num_flags - self.revealed_mines() as i32).to_string(), tile_set , 3, 0, CHARCOAL, UI_LIGHT);
        board.draw_rect(0,0,self.width*2+2, self.height*2+2, Tile{index:0, fg:REVEAL_BG, bg: REVEAL_BG});
        for y in 0..self.height { 
            for x in 0..self.width {
                board.set_tile(x*2 + 1, y*2 + 1, Self::tile_for(self[(x,y)]))
            }
        }        
        for y in 0..=self.height { 
            for x in 0..=self.width {
                let offset = (x*2,y*2);                
                match self.tile_arrangement((x,y)) {
                    0b0000 => 
                        Self::draw_t_shape(board, offset, [Self::REVEAL_BG_FILL;5]),
                    0b0001 => 
                        Self::draw_t_shape(board, offset, [Self::REVEAL_BG_FILL, Self::REVEAL_BG_FILL, Self::TL_OUTER_CORNER, Self::EDGE_HI_FILL, Self::EDGE_HI_FILL]),
                    0b0010 => 
                        Self::draw_t_shape(board, offset, [Self::REVEAL_BG_FILL, Self::EDGE_HI_FILL, Self::TR_OUTER_CORNER, Self::REVEAL_BG_FILL, Self::EDGE_LO_FILL]),
                    0b0011 => 
                        Self::draw_t_shape(board, offset, [Self::REVEAL_BG_FILL, Self::EDGE_HI_FILL, Self::EDGE_HI_FILL, Self::EDGE_HI_FILL, Self::UNREVEAL_BG_FILL]),
                    0b0100 => 
                        Self::draw_t_shape(board, offset, [Self::EDGE_HI_FILL, Self::REVEAL_BG_FILL, Self::BL_OUTER_CORNER, Self::EDGE_LO_FILL, Self::REVEAL_BG_FILL]),
                    0b0101 =>
                        Self::draw_t_shape(board, offset, [Self::EDGE_HI_FILL, Self::REVEAL_BG_FILL, Self::EDGE_HI_FILL, Self::UNREVEAL_BG_FILL, Self::EDGE_HI_FILL]),
                    0b0110 =>
                        Self::draw_t_shape(board, offset, [Self::EDGE_HI_FILL, Self::EDGE_HI_FILL, Self::BT_DIAG, Self::EDGE_LO_FILL, Self::EDGE_LO_FILL]),
                    0b0111 => 
                        Self::draw_t_shape(board, offset, [Self::EDGE_HI_FILL, Self::EDGE_HI_FILL, Self::TL_INNER_CORNER, Self::UNREVEAL_BG_FILL, Self::UNREVEAL_BG_FILL]),
                    0b1000 => 
                        Self::draw_t_shape(board, offset, [Self::EDGE_LO_FILL, Self::EDGE_LO_FILL, Self::BR_OUTER_CORNER, Self::REVEAL_BG_FILL, Self::REVEAL_BG_FILL]),
                    0b1001 =>
                        Self::draw_t_shape(board, offset, [Self::EDGE_LO_FILL, Self::EDGE_LO_FILL, Self::TB_DIAG, Self::EDGE_HI_FILL, Self::EDGE_HI_FILL]),
                    0b1010 => 
                        Self::draw_t_shape(board, offset, [Self::EDGE_LO_FILL, Self::UNREVEAL_BG_FILL, Self::EDGE_LO_FILL, Self::REVEAL_BG_FILL, Self::EDGE_LO_FILL]),
                    0b1011 => 
                        Self::draw_t_shape(board, offset, [Self::EDGE_LO_FILL, Self::UNREVEAL_BG_FILL, Self::TR_INNER_CORNER, Self::EDGE_HI_FILL, Self::UNREVEAL_BG_FILL]),
                    0b1100 => 
                        Self::draw_t_shape(board, offset, [Self::UNREVEAL_BG_FILL, Self::EDGE_LO_FILL, Self::EDGE_LO_FILL, Self::EDGE_LO_FILL, Self::REVEAL_BG_FILL]),
                    0b1101 => 
                        Self::draw_t_shape(board, offset, [Self::UNREVEAL_BG_FILL, Self::EDGE_LO_FILL, Self::BL_INNER_CORNER, Self::UNREVEAL_BG_FILL, Self::EDGE_HI_FILL]),
                    0b1110 => 
                        Self::draw_t_shape(board, offset, [Self::UNREVEAL_BG_FILL, Self::UNREVEAL_BG_FILL, Self::BR_INNER_CORNER, Self::EDGE_LO_FILL, Self::EDGE_LO_FILL]),
                    0b1111 => 
                        Self::draw_t_shape(board, offset, [Self::UNREVEAL_BG_FILL;5]),
                    _ => (),
                }
            }
        }
    }
}

fn main_loop(width:u32,height:u32,mines: u32) -> Option<(u32,u32,u32)> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.text_input().start();
    let window = video_subsystem.window("archaeologist", (width*2+1)*8, (height*2+1)*8+17+17)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size((width*2+1)*8, (height*2+1)*8+17+17).unwrap();
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut graphic = Graphic::blank(width*2+1, height*2+1).textured(&texture_creator);
    let mut game = Game::new(width,height,mines);
    let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../tiles")[..]));
    let mut cursor = Graphic::load_from(Cursor::new(&include_bytes!("../cursor")[..])).unwrap().textured(&texture_creator);
    let mut cursor_down = Graphic::load_from(Cursor::new(&include_bytes!("../cursor_down")[..])).unwrap().textured(&texture_creator);
    let mut win = Graphic::load_from(Cursor::new(&include_bytes!("../win")[..])).unwrap().textured(&texture_creator);
    let mut lose = Graphic::load_from(Cursor::new(&include_bytes!("../lose")[..])).unwrap().textured(&texture_creator);
    cursor.update_texture(&tile_set);
    cursor_down.update_texture(&tile_set);
    win.update_texture(&tile_set);
    lose.update_texture(&tile_set);
    let mut chrome = Graphic::solid(width*2+1,1, Tile{index:169, fg: UI_DARK, bg:UI_LIGHT}).textured(&texture_creator);    
    chrome.update_texture(&tile_set);
    let mut timer = Graphic::blank(5,1).textured(&texture_creator);
    game.draw(&mut graphic, &mut chrome, &tile_set);
    graphic.update_texture(&tile_set);
    chrome.update_texture(&tile_set);
    let mut mx = 0;
    let mut my = 0;
    let mut md = false;
    let mut gameover = false;
    let mut won = false;  
    let mut start = Instant::now();
    let mut old_time = 50000;
    let mut menu = MenuBar::new((width*2+1)*8)
                    .add(Menu::new("GAME",72,&texture_creator,&tile_set)
                            .add(MenuItem::new("Quick", Keycode::F1,&texture_creator,&tile_set))
                            .add(MenuItem::new("Easy", Keycode::F2,&texture_creator,&tile_set))
                            .add(MenuItem::new("Normal", Keycode::F3,&texture_creator,&tile_set))
                            .add(MenuItem::new("Hard", Keycode::F4,&texture_creator,&tile_set))
                            .add(MenuItem::separator(56, &texture_creator,&tile_set))
                            .add(MenuItem::new("Restart", Keycode::N,&texture_creator,&tile_set))
                            .add(MenuItem::new("Quit", Keycode::F12,&texture_creator,&tile_set)))
                    .add(Menu::new("VIEW",104,&texture_creator,&tile_set)
                            .add(MenuItem::new("Micro-mode", Keycode::F9, &texture_creator, &tile_set)));
    let mut rate_limiter = FPSManager::new();
    rate_limiter.set_framerate(40).unwrap();
    let mut micro_mode = false;
    loop {
        let time = start.elapsed().as_secs();
        if !gameover {
            if time != old_time {
                let s = time.to_string();            
                timer.draw_rect(0,0,5,1,Tile{index:0,fg:CHARCOAL,bg:rgba(0,0,0,0)});
                timer.draw_text(&s,&tile_set,5-s.len() as u32,0,rgba(0,0,0,255),rgba(0,0,0,0));
                timer.update_texture(&tile_set);
                old_time = time;
            }
            if game.revealed_mines() > 0 {
                gameover = true;                
            } else if game.unrevealed_blanks() == 0 {
                gameover = true; 
                won = true;
            } 
            if gameover {
                for j in 0..height {
                    for i in 0..width {
                        if game[(i,j)].mine {
                            game[(i,j)].status = CellStatus::Revealed;
                        }
                    }
                } 
                game.draw(&mut graphic, &mut chrome, &tile_set);
                graphic.update_texture(&tile_set);
            }
        }
        canvas.set_draw_color(UI_LIGHT);
        canvas.clear();
        graphic.draw(&mut canvas, (0,17));
        if !gameover { 
            if md { cursor_down.draw(&mut canvas, (mx*16, my * 16 + 17 )); }
            else { cursor.draw(&mut canvas, (mx*16, my * 16 + 17  )); }
        }
        canvas.set_draw_color(rgba(0,0,0,255));
        let h = height as i32 * 16 + 16 + 9;
        canvas.draw_line((0,h), (width as i32 * 16 + 8,h)).unwrap();
        chrome.draw(&mut canvas, (0,height as i32 * 16 + 16 + 14));
        timer.draw(&mut canvas, (width as i32 *2*8 - 40, height as i32 * 16 + 16 + 14));
        if gameover {
            let x = (width as i32 *2+1) * 4 - (21 * 4);
            let y = (height as i32 *2+1) * 4 - (21 * 4) + 17;
            if won {
                win.draw(&mut canvas, (x,y));
            } else {
                lose.draw(&mut canvas, (x,y));
            } 
        }
        menu.draw(&mut canvas);
        canvas.present();        
        for event in event_pump.poll_iter() {
            let h = menu.handle_event(event.clone(), &mut event_subsystem);
            match event {
                _ if h => {},
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::F12), .. } => {
                    return None
                },
                Event::KeyDown { keycode: Some(Keycode::F1),..} => {
                    return Some((10,10,10));
                },
                Event::KeyDown { keycode: Some(Keycode::F2),..} => {
                    return Some((20,20,40));
                },
                Event::KeyDown { keycode: Some(Keycode::F3),..} => {
                    return Some((20,20,80));
                },
                Event::KeyDown { keycode: Some(Keycode::F4),..} => {
                    return Some((30,30,220));
                },
                Event::KeyDown { keycode: Some(Keycode::F9), ..} => {
                    if micro_mode {
                        micro_mode = false;
                        canvas.window_mut().set_size((width*2+1)*8, (height*2+1)*8+17+17).unwrap_or_default();
                    } else {
                        micro_mode = true;
                        canvas.window_mut().set_size(((width*2+1)*8)/2, ((height*2+1)*8+17+17)/2).unwrap_or_default();
                    }
                }
                Event::KeyDown { keycode: Some(Keycode::N),..} => {
                    game = Game::new(width,height,mines);
                    start = Instant::now();
                    game.draw(&mut graphic, &mut chrome, &tile_set);
                    graphic.update_texture(&tile_set);
                    chrome.update_texture(&tile_set);
                    gameover = false; 
                    won = false;
                    md = false;
                }
                Event::MouseButtonDown { mouse_btn: _, x, y, ..} => {
                    let (sx,sy) = (x,y);// (x * dw as i32 / ww as i32, y * dh as i32 / wh as i32);
                    md = true; 
                    mx = ((sx-4)/16).min(width as i32-1).max(0); 
                    my = ((sy-21)/16).min(height  as i32-1).max(0);
                }
                Event::MouseMotion { x, y, ..} if !md => {
                    let (sx,sy) = (x,y) ;//(x * dw as i32 / ww as i32, y * dh as i32 / wh as i32);
                    mx = ((sx-4)/16).min(width  as i32-1).max(0); 
                    my = ((sy-21)/16).min(height  as i32-1).max(0);
                }

                Event::MouseButtonUp { ..} if gameover => {
                    game = Game::new(width,height,mines);
                    start = Instant::now();
                    game.draw(&mut graphic, &mut chrome, &tile_set);
                    graphic.update_texture(&tile_set);
                    chrome.update_texture(&tile_set);
                    gameover = false; 
                    won = false;
                    md = false;
                }
                Event::MouseButtonUp { mouse_btn: button, x, y, ..} => {
                    md = false;
                    let (sx,sy) = (x,y);//(x * dw as i32 / ww as i32, y * dh as i32 / wh as i32);
                    let tx = ((sx-4)/16).min(width as i32-1).max(0); 
                    let ty = ((sy-21)/16).min(height as i32-1).max(0);
                    if (tx,ty) == (mx, my) {
                        match button { 
                            MouseButton::Left => {
                                game.step_on(( tx.max(0) as u32, ty.max(0) as u32));
                                game.draw(&mut graphic, &mut chrome, &tile_set);
                                graphic.update_texture(&tile_set);
                                chrome.update_texture(&tile_set);
                            },
                            MouseButton::Right => {
                                game.toggle_flag(( tx.max(0) as u32, ty.max(0) as u32));
                                game.draw(&mut graphic, &mut chrome, &tile_set);
                                graphic.update_texture(&tile_set);
                                chrome.update_texture(&tile_set);
                            },
                            _ => {}
                        }            
                    }
                }
                _ => {},
            }
        }
        rate_limiter.delay();
    }
}

fn main() {
    let v : Vec<String> = env::args().collect();

    let width = v.get(1).and_then(|f| f.parse().ok()).unwrap_or(20);
    let height= v.get(2).and_then(|f| f.parse().ok()).unwrap_or(20);
    let mines = v.get(3).and_then(|f| f.parse().ok()).unwrap_or(80);
    let mut act = Some((width,height,mines)) ;
    while let Some((width,height,mines)) = act {
        act = main_loop(width, height, mines)
    }
    
}
