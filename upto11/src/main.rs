extern crate tesserae;
extern crate sdl2;

use tesserae::{*};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use utils::framerate::FPSManager;
use sdl2::render::TextureCreator;
use sdl2::render::RenderTarget;
use sdl2::render::Texture;
use sdl2::render::Canvas;
use utils::color::{*};
use utils::menu::{*};
use std::io::Cursor;
use utils::{*};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Direction { N, S, E, W }
impl Direction {
    fn to_index(&self) -> usize {
        match *self {
            Self::E => 1,
            Self::N => 2,
            Self::W => 3,
            Self::S => 4,
        }
    }
}
struct Game<'r> {
    tile_set: TileSet,
    tiles: [[u32;4];4],
    render_tiles: [[u32;4];4],
    cursor_gfx: [OutlinedTile<'r>;5],
    tile_gfx: [Graphic<Texture<'r>>;17],
    label_gfx : [Graphic<Texture<'r>>;17],
    highlight_tile: Graphic<Texture<'r>>,
    shadow_tile: Graphic<Texture<'r>>,
    dead: bool,
    won: u32,
    anim_ticks: u32,
    moves: Vec<(u32, (usize,usize), (usize,usize))>,
    spawn: Option<(usize,usize)>,
}
const SLIDE_SPEED: u32 = 10;
const SPAWN_SPEED: u32 = 16;
impl <'r>Game<'r> {
    fn tile<T>(c : Color, tile_set: &TileSet, texture_creator : &'r TextureCreator<T>) -> Graphic<Texture<'r>> {
        let mut g = Graphic::solid(4,4,Tile{fg:c,bg:TRANSPARENT,index:1}).textured(texture_creator);
        g[(0,0)] = Tile{index:219,fg:c,bg:TRANSPARENT};
        g[(1,0)] = Tile{index:220,fg:c,bg:TRANSPARENT};
        g[(2,0)] = Tile{index:220,fg:c,bg:TRANSPARENT};
        g[(3,0)] = Tile{index:221,fg:c,bg:TRANSPARENT};
        g[(0,1)] = Tile{index:235,fg:c,bg:TRANSPARENT};
        g[(3,1)] = Tile{index:237,fg:c,bg:TRANSPARENT};
        g[(0,2)] = Tile{index:235,fg:c,bg:TRANSPARENT};
        g[(3,2)] = Tile{index:237,fg:c,bg:TRANSPARENT};
        g[(0,3)] = Tile{index:251,fg:c,bg:TRANSPARENT};
        g[(1,3)] = Tile{index:252,fg:c,bg:TRANSPARENT};
        g[(2,3)] = Tile{index:252,fg:c,bg:TRANSPARENT};
        g[(3,3)] = Tile{index:253,fg:c,bg:TRANSPARENT};
        g.update_texture(tile_set);
        g
    }
    fn label<T>(index : u32, c : Color, tile_set: &TileSet, texture_creator: &'r TextureCreator<T> ) -> Graphic<Texture<'r>> {
        let mut g = Graphic::solid(2,1,Tile{fg:TRANSPARENT,bg:TRANSPARENT,index:1}).textured(texture_creator);
        g.draw_text(&index.to_string(), tile_set, 0,0, c, TRANSPARENT);
        g.update_texture(tile_set);
        g
    }
    fn end_animations(&mut self) {
        self.spawn = None;
        self.moves = Vec::new();
        self.render_tiles = self.tiles;
        self.anim_ticks = 0;
    }
    fn tick(&mut self) {
        if self.anim_ticks > 0 {
            self.anim_ticks -= 1;
            if self.anim_ticks == 0 {
                if self.moves.len() > 0 {
                    self.moves = Vec::new();
                    self.render_tiles = self.tiles;
                    self.anim_ticks = SPAWN_SPEED;
                }
            }
        }
    }
    fn spawn(&mut self) {
        loop{
            let x = rand::random::<usize>();
            if self.tiles[x % 4][(x/4)%4] == 0 {
                if x % 10 == 0 {
                    self.tiles[x % 4][(x/4)%4]= 4;
                    self.render_tiles[x % 4][(x/4)%4]= 4;
                } else{
                    self.tiles[x % 4][(x/4)%4]= 2;
                    self.render_tiles[x % 4][(x/4)%4]= 2;
                }
                self.spawn = Some((x%4,(x/4)%4));
                break;
            }
        }
    }
    fn new<T>(texture_creator : &'r TextureCreator<T>) -> Game<'r> {
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
        let cursor_gfx = [
            OutlinedTile::new(188,WHITE,&tile_set,texture_creator),
            OutlinedTile::new(64,WHITE,&tile_set,texture_creator),
            OutlinedTile::new(65,WHITE,&tile_set,texture_creator),
            OutlinedTile::new(66,WHITE,&tile_set,texture_creator),
            OutlinedTile::new(67,WHITE,&tile_set,texture_creator),
        ];
        let tiles = [[0;4];4];
        let tile_gfx = [
            Self::tile(rgba(186,189,182,255),&tile_set,texture_creator),
            Self::tile(rgba(227,210,150,255),&tile_set,texture_creator),
            Self::tile(rgba(227,202,0,255),&tile_set,texture_creator),
            Self::tile(rgba(196,160,0,255),&tile_set,texture_creator),
            Self::tile(rgba(206,92,0,255),&tile_set,texture_creator),
            Self::tile(rgba(245,121,0,255),&tile_set,texture_creator),
            Self::tile(rgba(252,175,62,255),&tile_set,texture_creator),
            Self::tile(rgba(114,159,207,255),&tile_set,texture_creator),
            Self::tile(rgba(52,101,164,255),&tile_set,texture_creator),
            Self::tile(rgba(32,74,135,255),&tile_set,texture_creator),
            Self::tile(rgba(92,52,102,255),&tile_set,texture_creator),
            Self::tile(rgba(117,80,123,255),&tile_set,texture_creator),
            Self::tile(rgba(172,127,168,255),&tile_set,texture_creator),
            Self::tile(rgba(239,41,41,255),&tile_set,texture_creator),
            Self::tile(rgba(204,0,0,255),&tile_set,texture_creator),
            Self::tile(rgba(164,0,0,255),&tile_set,texture_creator),
            Self::tile(rgba(100,0,0,255),&tile_set,texture_creator),
        ];
        let label_gfx = [
            Self::label(1,BLACK,&tile_set, texture_creator),
            Self::label(2,BLACK,&tile_set, texture_creator),
            Self::label(3,BLACK,&tile_set, texture_creator),
            Self::label(4,BLACK,&tile_set, texture_creator),
            Self::label(5,WHITE,&tile_set, texture_creator),
            Self::label(6,BLACK,&tile_set, texture_creator),
            Self::label(7,BLACK,&tile_set, texture_creator),
            Self::label(8,WHITE,&tile_set, texture_creator),
            Self::label(9,WHITE,&tile_set, texture_creator),
            Self::label(10,WHITE,&tile_set, texture_creator),
            Self::label(11,WHITE,&tile_set, texture_creator),
            Self::label(12,WHITE,&tile_set, texture_creator),
            Self::label(13,BLACK,&tile_set, texture_creator),
            Self::label(14,WHITE,&tile_set, texture_creator),
            Self::label(15,WHITE,&tile_set, texture_creator),
            Self::label(16,WHITE,&tile_set, texture_creator),
            Self::label(17,WHITE,&tile_set, texture_creator),
        ];
        let highlight_tile = Self::tile(WHITE,&tile_set,texture_creator);
        let shadow_tile = Self::tile(BLACK,&tile_set,texture_creator);
        let mut g = Game {
            tile_set: tile_set,
            tiles: tiles,
            render_tiles: tiles.clone(),
            won: 0,
            anim_ticks: 0,
            highlight_tile, shadow_tile,
            dead: false,
            spawn: None,
            moves: Vec::new(),
            cursor_gfx, tile_gfx, label_gfx
        };
        g.spawn();
        g
    }

    fn draw_tile<T:RenderTarget>(&self, canvas : &mut Canvas<T>, val: u32, pos:(i32,i32), hl: bool) {
        if val <= 1 { return; }
        let label_onedigit = (pos.0 + 12, pos.1 + 12);
        let label_twodigit = (pos.0 + 8, pos.1 + 12);
        let (idx,labelpos) = match val {
            2 => (1, label_onedigit),
            4 => (2, label_onedigit),
            8 => (3, label_onedigit),
            16 => (4, label_onedigit),
            32 => (5, label_onedigit),
            64 => (6, label_onedigit),
            128 => (7, label_onedigit),
            256 => (8, label_onedigit),
            512 => (9, label_onedigit),
            1024 => (10, label_twodigit),
            2048 => (11, label_twodigit),
            4096 => (12, label_twodigit),
            8192 => (13, label_twodigit),
            16384 => (14, label_twodigit),
            32768 => (15, label_twodigit),
            65536 => (16, label_twodigit),
            _ => (17, label_twodigit),
        };
        self.shadow_tile.draw(canvas,(pos.0-1,pos.1+1));
        self.shadow_tile.draw(canvas,(pos.0+1,pos.1-1));
        self.shadow_tile.draw(canvas,(pos.0+1,pos.1+1));
        self.shadow_tile.draw(canvas,(pos.0-1,pos.1-1));
        self.shadow_tile.draw(canvas,(pos.0-1,pos.1));
        self.shadow_tile.draw(canvas,(pos.0+1,pos.1));
        self.shadow_tile.draw(canvas,(pos.0,pos.1+1));
        self.shadow_tile.draw(canvas,(pos.0,pos.1-1));
        if hl { &self.highlight_tile } else { &self.tile_gfx[idx-1] }.draw(canvas,pos);
        self.label_gfx[idx-1].draw(canvas,labelpos);
    }

    fn draw<T:RenderTarget>(&self, canvas : &mut Canvas<T>) {
        canvas.set_draw_color(CHARCOAL);
        canvas.clear();
        for r in 0..4 {
            for c in 0..4 {
                if Some((r,c)) != self.spawn || self.anim_ticks == 0 { 
                    self.draw_tile(canvas, self.render_tiles[r][c], (c as i32 * 32, r as i32 *32 +17),false);
                }
            }
        }
        if self.anim_ticks > 0 {
            if self.moves.len() > 0 {
                for (v,sp,ep) in &self.moves {
                    let start = (sp.1 as i32 * 32, sp.0 as i32 * 32 + 17);
                    let end = (ep.1 as i32 * 32, ep.0 as i32 * 32 + 17);
                    let loc = (start.0 + ((end.0 - start.0)*(16 - self.anim_ticks as i32))/16,
                               start.1 + ((end.1 - start.1)*(16 - self.anim_ticks as i32))/16);
                    self.draw_tile(canvas,*v,loc,false);
                }
            } else if let Some((r,c)) = self.spawn {
                self.draw_tile(canvas, self.tiles[r][c], (c as i32 * 32, r as i32 *32 +17),self.anim_ticks /4 % 2 == 0);
            }

        }
    }
    fn dir_for_cursor(&self, ix : i32, iy : i32) -> Option<Direction> {
        let (x,y) = ((ix/32).max(0).min(4) - 2, ((iy - 17) / 32).max(0).min(4) - 2);
            match (x-y,x+y) {
                (a,b) if a > 0 && b > 0 => Some(Direction::E),
                (a,b) if a < 0 && b < 0 => Some(Direction::W),
                (a,b) if a < 0 && b > 0 => Some(Direction::S),
                (a,b) if a > 0 && b < 0 => Some(Direction::N),
                _ => None
            }
    }
    fn can_make_move(&self) -> bool {
        for d in [Direction::N, Direction::S, Direction::E, Direction::W].iter() {
            match d {
                Direction::W =>{
                    for array in self.tiles.iter() {
                        for  col in 0..4 {
                            for testcol in (col+1)..4 {
                                if array[testcol] != 0 {
                                    if array[col] == 0 {
                                        return true;
                                    }
                                    else if array[col] == array[testcol] {
                                        return true;
                                    } else {
                                        break
                                    }
                                }
                            }
                        }
                    }
                } ,
                Direction::E =>{
                    for array in self.tiles.iter() {
                        for  col in (0..4).rev() {
                            for testcol in (0..col).rev() {
                                if array[testcol] != 0 {
                                    if array[col] == 0 {
                                        return true;
                                    }
                                    else if array[col] == array[testcol] {
                                        return true;
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                } ,
                Direction::S =>{
                    for col in 0..4 {
                        for row in (0..4).rev() {
                            for testrow in (0..row).rev() {
                                if self.tiles[testrow][col] != 0 {
                                    if self.tiles[row][col] == 0 {
                                        return true;
                                    } else if self.tiles[row][col] == self.tiles[testrow][col] {
                                        return true;
                                    }else {
                                        break;
                                    }
        
                                }
                            }
                        }
                    }
                } ,
                Direction::N =>{
                    for col in 0..4 {
                        for row in 0..4{
                            for testrow in (row+1)..4 {
                                if self.tiles[testrow][col] != 0 {
                                    if self.tiles[row][col] == 0 {
                                        return true;
                                    } else if self.tiles[row][col] == self.tiles[testrow][col] {
                                        return true;
                                    }else {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                },
            }
        }
        false
    }
    fn make_move(&mut self, d : Direction) {
        let board = self.tiles.clone();        
        self.end_animations();
        match d {
            Direction::W =>{
                let mut r = 0;
                for array in &mut self.tiles.iter_mut() {
                    for  col in 0..4 {
                        for testcol in (col+1)..4 {
                            if array[testcol] != 0 {
                                if array[col] == 0 {
                                    self.moves.push((array[testcol], (r,testcol), (r,col)));
                                    self.render_tiles[r][testcol] = 0;
                                    array[col] += array[testcol];
                                    array[testcol] = 0;
                                }
                                else if array[col] == array[testcol] {
                                    self.moves.push((array[testcol], (r,testcol), (r,col)));
                                    self.render_tiles[r][testcol] = 0;
                                    array[col] += array[testcol];
                                    array[testcol] = 0;
                                    break;
                                } else {
                                    break
                                }
                            }
                        }
                    }
                    r += 1;
                }
            } ,
            Direction::E =>{
                let mut r = 0;
                for array in &mut self.tiles.iter_mut() {
                    for  col in (0..4).rev() {
                        for testcol in (0..col).rev() {
                            if array[testcol] != 0 {
                                if array[col] == 0 {
                                    self.moves.push((array[testcol], (r,testcol), (r,col)));
                                    self.render_tiles[r][testcol] = 0;
                                    array[col] += array[testcol];
                                    array[testcol] = 0;
                                }
                                else if array[col] == array[testcol] {
                                    self.moves.push((array[testcol], (r,testcol), (r,col)));
                                    self.render_tiles[r][testcol] = 0;
                                    array[col] += array[testcol];
                                    array[testcol] = 0;
                                    break;
                                }else {
                                    break;
                                }
                            }
                        }
                    }
                    r += 1;
                }

            } ,
            Direction::S =>{
                for col in 0..4 {
                    for row in (0..4).rev() {
                        for testrow in (0..row).rev() {
                            if self.tiles[testrow][col] != 0 {
                                if self.tiles[row][col] == 0 {
                                    self.moves.push((self.tiles[testrow][col], (testrow,col), (row,col)));
                                    self.render_tiles[testrow][col] = 0;
                                    self.tiles[row][col] += self.tiles[testrow][col];
                                    self.tiles[testrow][col] = 0;
                                } else if self.tiles[row][col] == self.tiles[testrow][col] {
                                    self.moves.push((self.tiles[testrow][col], (testrow,col), (row,col)));
                                    self.render_tiles[testrow][col] = 0;
                                    self.tiles[row][col] += self.tiles[testrow][col];
                                    self.tiles[testrow][col] = 0;
                                    break;
                                }else {
                                    break;
                                }
     
                            }
                        }
                    }
                }
            } ,
            Direction::N =>{
                for col in 0..4 {
                    for row in 0..4{
                        for testrow in (row+1)..4 {
                            if self.tiles[testrow][col] != 0 {
                                if self.tiles[row][col] == 0 {
                                    self.moves.push((self.tiles[testrow][col], (testrow,col), (row,col)));
                                    self.render_tiles[testrow][col] = 0;
                                    self.tiles[row][col] += self.tiles[testrow][col];
                                    self.tiles[testrow][col] = 0;
                                } else if self.tiles[row][col] == self.tiles[testrow][col] {
                                    self.moves.push((self.tiles[testrow][col], (testrow,col), (row,col)));
                                    self.render_tiles[testrow][col] = 0;
                                    self.tiles[row][col] += self.tiles[testrow][col];
                                    self.tiles[testrow][col] = 0;
                                    break;
                                }else {
                                    break;
                                }
                            }
                        }
                    }
                }
            },
        }
        if board == self.tiles {
            return;
        }
        self.anim_ticks = SLIDE_SPEED;
        if self.won == 0 {
            if self.tiles.iter().any(|x| x.iter().any(|z| *z == 2048)) {
                self.won = 1;
            }
        }
        //TODO Check win/loss
        self.spawn();        
        if !self.can_make_move() {
            self.dead = true;            
        }
    }
}

enum Splash {
    Loss, Won
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.text_input().start();
    let window = video_subsystem.window("up to 11", 4*32, 4*32 + 17)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    let mut splash = None;
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(4*32, 4*32 + 17).unwrap();
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut game = Game::new(&texture_creator);
    let mut lose = Graphic::load_from(Cursor::new(&include_bytes!("../lose")[..])).unwrap().textured(&texture_creator);
    lose.update_texture(&game.tile_set);
    let mut win = Graphic::load_from(Cursor::new(&include_bytes!("../win")[..])).unwrap().textured(&texture_creator);
    win.update_texture(&game.tile_set);
    let mut rate_limiter = FPSManager::new();    
    rate_limiter.set_framerate(40).unwrap();
    let mut micro_mode = false;    
    let mut menu = MenuBar::new(4*32)
                    .add(Menu::new("MENU",112,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Restart",15, Keycode::N,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(96,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Micro-mode",360, Keycode::F9, &texture_creator, &game.tile_set))
                            .add(MenuItem::separator(96,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Quit", 363, Keycode::F12,&texture_creator,&game.tile_set)));
    let mut cx = 0;
    let mut cy = 0;

    loop {
        if splash.is_none() { 
            if game.won == 1  {
                splash = Some(Splash::Won);
            } else if game.dead  {
                splash = Some(Splash::Loss);
            } 
        }
        game.tick();
        game.draw(&mut canvas);
        let c = game.dir_for_cursor(cx, cy).map_or(0, |x| x.to_index());
        if let Some(s) = &splash {
            match *s {
                Splash::Won => win.draw(&mut canvas, (4 * 16 - (16*4), 4 * 16 - (21*4) + 17 )),
                _ => lose.draw(&mut canvas, (4 * 16 - (16*4), 4 * 16 - (21*4) + 17 )),
            }
        }
        game.cursor_gfx[c].draw_enlarged(&mut canvas,(cx-10,cy-10));
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
                Event::KeyDown { keycode: Some(Keycode::F9), ..} => {
                    if micro_mode {
                        micro_mode = false;
                        canvas.window_mut().set_size(4*32, 4*32+17).unwrap_or_default();
                    } else {
                        micro_mode = true;
                        canvas.window_mut().set_size(4*16, (4*32+17)/2).unwrap_or_default();
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::N), .. } => {
                    game = Game::new(&texture_creator);
                    splash = None
                },
                Event::KeyDown {..} if splash.is_some() => {
                    if game.won == 1 { game.won = 2} ;
                    splash = None  
                }
                Event::MouseButtonUp {..} if splash.is_some() => {
                    if game.won == 1 { game.won = 2} ;
                    splash = None
                }
                Event::KeyDown { keycode: Some(Keycode::Up), ..} |
                Event::KeyDown { keycode: Some(Keycode::W), ..}
                  => game.make_move(Direction::N),
                Event::KeyDown { keycode: Some(Keycode::Down), ..} |
                Event::KeyDown { keycode: Some(Keycode::S), ..}
                  => game.make_move(Direction::S),
                Event::KeyDown { keycode: Some(Keycode::Left), ..} |
                Event::KeyDown { keycode: Some(Keycode::A), ..}
                  => game.make_move(Direction::W),
                Event::KeyDown { keycode: Some(Keycode::Right), ..} |
                Event::KeyDown { keycode: Some(Keycode::D), ..}
                  => game.make_move(Direction::E),
                Event::MouseButtonUp {..} if cy > 17 => {
                    match game.dir_for_cursor(cx, cy) {
                        None => {},
                        Some(d) => game.make_move(d),
                    }
                }
                Event::MouseMotion { x,y,..} if y < 17 => {
                    cx = x;
                    cy = y;
                    sdl_context.mouse().show_cursor(true);
                }
                Event::MouseMotion { x,y,..} => {
                    cx = x;
                    cy = y;
                    sdl_context.mouse().show_cursor(false);
                }
                _ => {},
            }
        }
        rate_limiter.delay();
    }
}

