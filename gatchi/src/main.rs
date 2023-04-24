extern crate tesserae;
extern crate sdl2;


use rand::Rng;
use tesserae::{*};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
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

struct Game<'r> {
    tile_set: TileSet,
    board: [[u32;WIDTH as usize];HEIGHT as usize],
    render_board: [[u32;WIDTH as usize];HEIGHT as usize],
    undo_boards: Vec<(u32,[[u32;WIDTH as usize];HEIGHT as usize])>,
    moves: Vec<(u32,(i32,i32),(i32,i32))>,
    pieces: [Graphic<Texture<'r>>;6],
    highlight_piece: Graphic<Texture<'r>>,
    highlighted: Vec<(usize,usize)>,
    pieces_left: u32,
    ticks: u32,
    cursor: (usize,usize),
    downwards: bool,
}


const WIDTH : u32 = 10;
const HEIGHT : u32 = 16;

impl <'r>Game<'r> {
    fn new<T>(texture_creator : &'r TextureCreator<T>) -> Game<'r> {
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
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
        let pieces = [graphic_j,graphic_l,graphic_o,graphic_s,graphic_z,graphic_t];
        Game {
            tile_set,
            ticks:0,
            board: [[0;WIDTH as usize];HEIGHT as usize],
            render_board: [[0;WIDTH as usize];HEIGHT as usize],
            moves: Vec::new(),
            pieces_left: 0,
            pieces,
            highlight_piece: graphic_hl,
            highlighted: Vec::new(),
            undo_boards: Vec::new(),
            downwards: false,
            cursor: (0,0),
        }
    }
    fn draw<T:RenderTarget>(&self, canvas : &mut Canvas<T>) {
        canvas.set_draw_color(rgba(218,218,206,255));
        canvas.clear();
        for i in 0..HEIGHT as usize {
            for j in 0..WIDTH as usize {
                if self.render_board[i][j] > 0 {
                    self.pieces[self.render_board[i][j] as usize - 1].draw(canvas, (j as i32 * 12, i as i32 * 12 + 17));
                }
            }
        }
        if self.highlighted.len() > 1 && self.ticks == 0 {
            for (y,x) in &self.highlighted {
                self.highlight_piece.draw(canvas, (*x as i32 * 12, *y as i32 * 12 +17));
            }
        }
        for (v,start,end) in &self.moves {
            let loc = (start.0 + ((end.0 - start.0)*(10 - self.ticks as i32))/10,
                       start.1 + ((end.1 - start.1)*(10 - self.ticks as i32))/10);
            self.pieces[*v as usize - 1].draw(canvas, loc);
        }
    }
    fn flood_fill(&mut self, c: (usize,usize)) {
        if self.highlighted.contains(&c) { return };
        if self.board[c.0][c.1] == 0 { return };
        let x = self.board[c.0][c.1];
        self.highlighted.push(c);
        if c.0 > 0 {
            if self.board[c.0-1][c.1] == x { self.flood_fill((c.0-1,c.1)) }
        }
        if c.0 < (HEIGHT - 1) as usize {
            if self.board[c.0+1][c.1] == x { self.flood_fill((c.0+1,c.1)) }
        }
        if c.1 > 0 {
            if self.board[c.0][c.1-1] == x { self.flood_fill((c.0,c.1-1)) }
        }
        if c.1 < (WIDTH - 1) as usize {
            if self.board[c.0][c.1+1] == x { self.flood_fill((c.0,c.1+1)) }
        }
    }
    fn rehighlight(&mut self) {
        self.cursor_moved(self.cursor);
    }
    fn cursor_moved(&mut self, c :(usize,usize)) {
        if self.highlighted.contains(&c) { return };
        self.cursor = c;
        self.highlighted = Vec::new();
        self.flood_fill(c);
    }
    fn eliminate_highlighted(&mut self) {
        if self.highlighted.len() > 1 {
            self.undo_boards.push((self.pieces_left,self.board.clone()));
            for (y,x) in &self.highlighted {
                self.board[*y][*x] = 0;
                self.render_board[*y][*x] = 0;
                self.pieces_left -= 1;
            }
            self.highlighted = Vec::new();
        }
    }
    fn undo(&mut self) {
        if self.ticks > 0 { return };
        if let Some((p,b)) = self.undo_boards.pop() {
            self.board = b;
            self.pieces_left = p;
            self.render_board = b;
        }        
    }
    fn cursor_clicked(&mut self) {
        if self.ticks > 0 { return };
        self.eliminate_highlighted();
        self.collapse_down();
    }
    fn col_clear(&self,x : usize) -> bool {
        (0..HEIGHT as usize).all(|y| self.board[y][x] == 0)
    }
    fn move_column(&mut self, fromx : usize, tox: usize) {
        for y in 0..HEIGHT as usize {
            if self.board[y][fromx] > 0 {
                self.board[y][tox] = self.board[y][fromx];
                self.board[y][fromx] = 0;
                self.render_board[y][fromx] = 0;
                self.moves.push((self.board[y][tox],(fromx as i32*12,y as i32*12+17), (tox as i32*12, y as i32*12+17)));
            }
        }
    }
    fn collapse_left(&mut self) {
        for x in 0..WIDTH {
            if self.col_clear(x as usize) {
                for tx in x..WIDTH {
                    if !self.col_clear(tx as usize) {
                        self.move_column(tx as usize,x as usize);
                        break;
                    }
                }
            }
        }
        if self.moves.len() > 0 {
            self.ticks = 10;
        } else { 
            self.rehighlight();
        }
    }
    fn collapse_down(&mut self) {
        for x in 0..WIDTH {
            for y in (0..HEIGHT).rev() {
                if self.board[y as usize][x as usize] == 0 {
                    for ty in (0..y).rev() {
                        if self.board[ty as usize][x as usize] > 0 {
                            self.board[y as usize][x as usize] = self.board[ty as usize][x as usize];
                            self.board[ty as usize][x as usize] = 0;
                            self.render_board[ty as usize][x as usize] = 0;
                            self.moves.push((self.board[y as usize][x as usize],(x as i32*12,ty as i32*12+17), (x as i32*12, y as i32*12+17)));
                            break;
                        }
                    }
                }
            }
        }
        if self.moves.len() > 0 {
            self.downwards = true;
            self.ticks = 10;
        } else {
            self.collapse_left();
        }
    }
    fn tick(&mut self) {
        if self.ticks > 0 {
            self.ticks -= 1;
            if self.ticks == 0 {
                self.moves = Vec::new();
                self.render_board = self.board;
                if self.downwards {
                    self.downwards = false;
                    self.collapse_left();
                } else {
                    self.rehighlight();
                }
            }
        }
    }
    fn new_game(&mut self, diff: u32) {
        for i in 0..HEIGHT as usize {
            for j in 0..WIDTH as usize {
                self.board[i][j] = thread_rng().gen_range(0,diff) + 1;
            } 
        }
        self.render_board = self.board.clone();
        self.pieces_left = WIDTH * HEIGHT;
        self.highlighted = Vec::new();
    }
    fn moves_remaining(&self) -> bool {
        for x in 0..WIDTH as usize {
            for y in 0..HEIGHT as usize  {
                if self.board[y][x] > 0 {
                    if x > 0 && self.board[y][x] == self.board[y][x-1] { return true };
                    if x < WIDTH as usize-1 && self.board[y][x] == self.board[y][x+1] { return true };
                    if y > 0 && self.board[y][x] == self.board[y-1][x] { return true };
                    if y < HEIGHT as usize-1 && self.board[y][x] == self.board[y+1][x] { return true };
                }
            }
        }
        return false;
    }
}

enum Splash {
    Loss, Won,
}


fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.text_input().start();
    let window = video_subsystem.window("gatchi", WIDTH*12 + 12, HEIGHT*12 + 12 + 16 + 17)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    let mut splash : Option<Splash> = None;
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(WIDTH*12+12, HEIGHT*12 + 12 + 16 + 17).unwrap();
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut game = Game::new(&texture_creator);
    let mut lose = Graphic::load_from(Cursor::new(&include_bytes!("../gatchi_lose")[..])).unwrap().textured(&texture_creator);
    let mut won = Graphic::load_from(Cursor::new(&include_bytes!("../gatchi_won")[..])).unwrap().textured(&texture_creator);
    lose.update_texture(&game.tile_set);
    won.update_texture(&game.tile_set);
    let mut status = Graphic::blank(15,1).textured(&texture_creator);    
    let mut rate_limiter = FPSManager::new();
    rate_limiter.set_framerate(40).unwrap();
    let mut micro_mode = false;
    game.new_game(5);
    let mut difficulty = 5;
    let mut menu = MenuBar::new(WIDTH*12+12)
                    .add(Menu::new("MENU",112,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Quick", 352,Keycode::F1,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Easy", 353, Keycode::F2,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Medium", 354, Keycode::F3,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Hard", 355, Keycode::F4,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(96,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Undo", 27,Keycode::Z,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Restart",15, Keycode::N,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(96, &texture_creator,&game.tile_set))
                            .add(MenuItem::new("Micro-mode",360, Keycode::F9, &texture_creator, &game.tile_set))
                            .add(MenuItem::separator(96, &texture_creator,&game.tile_set))
                            .add(MenuItem::new("Quit", 363, Keycode::F12,&texture_creator,&game.tile_set)));
    let mut cx = 0;
    let mut cy = 0;

    loop {
        if splash.is_none() { 
            if game.pieces_left == 0 {
                splash = Some(Splash::Won);
            } else if !game.moves_remaining() {
                splash = Some(Splash::Loss);
            }
            //check for splash
        }
        game.tick();
        game.draw(&mut canvas);
        canvas.set_draw_color(BLACK);
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*12 + 12 + 17,WIDTH*12 + 16,17)).unwrap();
        canvas.set_draw_color(rgba(176,179,172,255));
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*12 + 12 + 18,WIDTH*12 + 16,16)).unwrap();
        status.draw_rect(0,0,15,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status.draw_text(&(game.pieces_left.to_string() + " pieces left"), &game.tile_set,0,0,BLACK,TRANSPARENT);
        status.update_texture(&game.tile_set);
        status.draw(&mut canvas, (6,HEIGHT as i32*12 + 16 + 15 + 4));
        if let Some(s) = &splash {
            match *s {
                Splash::Loss => lose.draw(&mut canvas, (WIDTH as i32 * 6 + 8 - (15*4), HEIGHT as i32 * 6 - (4*4) + 17 )),
                Splash::Won => won.draw(&mut canvas, (WIDTH as i32 * 6 + 8 - (15*4), HEIGHT as i32 * 6 - (4*4) + 17 )),
            }
        }
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
                Event::KeyDown { keycode: Some(Keycode::N), .. } => {
                    splash = None;
                    game.new_game(difficulty);
                },
                Event::KeyDown { keycode: Some(Keycode::Z), .. } => {
                    splash = None;
                    game.undo();
                },
                Event::KeyDown { keycode: Some(Keycode::F1), .. } => {
                    splash = None;
                    game.new_game(3);
                    difficulty = 3;
                },
                Event::KeyDown { keycode: Some(Keycode::F2), .. } => {
                    splash = None;
                    game.new_game(4);
                    difficulty = 4;
                },
                Event::KeyDown { keycode: Some(Keycode::F3), .. } => {
                    splash = None;
                    game.new_game(5);
                    difficulty = 5;
                },
                Event::KeyDown { keycode: Some(Keycode::F4), .. } => {
                    splash = None;
                    game.new_game(6);
                    difficulty = 6;
                },
                Event::MouseButtonUp {y,..} if y > 17 => {
                    game.cursor_clicked()
                }
                Event::KeyDown { keycode: Some(Keycode::F9), ..} => {
                    if micro_mode {
                        micro_mode = false;
                        canvas.window_mut().set_size(WIDTH*12 + 12, HEIGHT*12+12+16+17).unwrap_or_default();
                    } else {
                        micro_mode = true;
                        canvas.window_mut().set_size(WIDTH*6 + 6, (HEIGHT*12+12+16+17)/2).unwrap_or_default();
                    }
                },
                Event::MouseMotion { x,y,..} => {
                    let ax = (x - 8) / 12;
                    let ay = (y - 17 - 8) / 12;
                    if (ax,ay) != (cx,cy) {
                        cx = ax;
                        cy = ay;
                        game.cursor_moved((cy.max(0).min(HEIGHT as i32 - 1) as usize,cx.max(0).min(WIDTH as i32 - 1) as usize));
                    }
                }
                _ => {},
            }
        }
        rate_limiter.delay();
    }
}
