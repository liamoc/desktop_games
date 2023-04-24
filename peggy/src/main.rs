
extern crate tesserae;
extern crate sdl2;
extern crate rand;
extern crate utils;
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

struct GraphicsSet<T> {
    tile_set: TileSet,
    cell: [Graphic<T>;3],
    peg: [Graphic<T>;2],
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

        
        let cell = [
            Self::piece(texture_creator,&tile_set,DARKER_GRAY,264,263,261,262),
            Self::piece(texture_creator,&tile_set,AMBER,264,263,261,262),
            Self::piece(texture_creator,&tile_set,PALE_BLUE,264,263,261,262),            
        ];
        let peg = [
            Self::piece(texture_creator,&tile_set,PALE_BROWN,168,170,200,202),
            Self::piece(texture_creator,&tile_set,YELLOW,168,170,200,202),            
        ];
        
        GraphicsSet {
            tile_set,cell,peg
        }
    }

}

fn draw_peg<'r>(canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'r>>, position : (i32,i32)) {
    graphics.peg[0].draw(canvas,(position.0-1,position.1));
    graphics.peg[0].draw(canvas,(position.0+1,position.1));
    graphics.peg[0].draw(canvas,(position.0-1,position.1+1));
    graphics.peg[0].draw(canvas,(position.0,position.1+1));
    graphics.peg[0].draw(canvas,(position.0+1,position.1+1));
    graphics.peg[0].draw(canvas,(position.0-1,position.1-1));
    graphics.peg[0].draw(canvas,(position.0,position.1-1));
    graphics.peg[0].draw(canvas,(position.0+1,position.1-1));
    graphics.peg[1].draw(canvas,(position.0,position.1));
    graphics.cell[1].draw(canvas,(position.0,position.1));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum EndGame {
    Victory,
    Failure
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Cell {
    location: (i32,i32),
    neighbours: Vec<Option<usize>>
}
struct Animation {
    time: u32,
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
        draw_peg(canvas, graphics, (x,y));
    }
    
}
pub struct Table {    
    animations: Vec<(Animation,u32)>,  
    game_over : Option<EndGame>,
    layout: Vec<Cell>,
    pegs: Vec<bool>,
    history: Vec<(usize,usize,usize)>, // origin over dest
    goal_peg: usize,
    move_count:u32,
}

const OFFSET : (i32,i32) = (4+12+8,20+12);
#[derive(Clone,Copy, Debug, PartialEq, Eq, Hash)]
pub enum Layout {
    English,
    French,
    Wiegleb,
    Asymmetrical,
    Diamond,
    Triangular
}
impl Table {
    pub fn can_undo(&self) -> bool {
        if self.animations.len() > 0 { return false; }
        if self.history.len() > 1 {
            true
        } else { false }
    }
    pub fn undo(&mut self) {
        if let Some((origin,over,dest)) = self.history.pop() {
            self.pegs[origin] = true;
            self.pegs[over] = true;
            self.pegs[dest] = false;
            self.check_won();
        }
    }
    fn animate_move(&mut self, start: (i32,i32), end: (i32,i32), time: u32, and_then: Box<dyn FnOnce(&mut Table)>) {
        self.animations.push((Animation { time: time, start: start, end:end, and_then: and_then},0))
    }
    pub fn add_peg_to(&mut self, origin: usize) {
        self.pegs[origin] = true;
    }
    pub fn animate_peg(&mut self, dest:(i32,i32), start_pos: (i32,i32), and_then: Box<dyn FnOnce(&mut Table)>) {
        //let dest_pos = (dest.0 as i32 * 16 + OFFSET.0, dest.1 as i32 * 16 + OFFSET.1);        
        self.animate_move(start_pos, dest, 14, and_then);
    } 
    fn english_layout() -> (Vec<Cell>,Vec<bool>, usize) {
        let mut temp : [[Option<usize>;9];9] = Default::default();
        let mut i = 0;
        let mut l = Vec::new();
        let mut r = 0;
        let mut pegs = Vec::new();
        for x in 0..7 {
            for y in 0..7 {
                let cx = x < 2 || x >= 5;
                let cy = y < 2 || y >= 5;
                if !(cx && cy) {
                    temp[x][y] = Some(i);
                    i+= 1;
                    pegs.push(true);
                    l.push(Cell {
                        location: (x as i32 * 16+16, y as i32 * 16+16),
                        neighbours: vec![],
                    });
                } else {
                    temp[x][y] = None;
                }
                

            }
        }
        for x in 0..7 {
            for y in 0..7 {
                let n = if y > 0 { temp[x][y-1] } else { None };
                let s = if y < 6 { temp[x][y+1] } else { None };
                let w = if x > 0 { temp[x-1][y] } else { None };
                let e = if x < 6 { temp[x+1][y] } else { None };
                if let Some(v) = temp[x][y] {
                    if x == 3 && y == 3 { pegs[v] = false; r = v };
                    l[v].neighbours = vec![n,s,e,w];
                }
            }
        }
        (l,pegs,r)
    }
    fn wiegleb_layout() -> (Vec<Cell>,Vec<bool>, usize) {
        let mut temp : [[Option<usize>;9];9] = Default::default();
        let mut i = 0;
        let mut l = Vec::new();
        let mut r = 0;
        let mut pegs = Vec::new();
        for x in 0..9 {
            for y in 0..9 {
                let cx = x < 3 || x >= 6;
                let cy = y < 3 || y >= 6;
                if !(cx && cy) {
                    temp[x][y] = Some(i);
                    i+= 1;
                    pegs.push(true);
                    l.push(Cell {
                        location: (x as i32 * 16, y as i32 * 16),
                        neighbours: vec![],
                    });
                } else {
                    temp[x][y] = None;
                }
                

            }
        }
        
        for x in 0..9 {
            for y in 0..9 {
                let n = if y > 0 { temp[x][y-1] } else { None };
                let s = if y < 8 { temp[x][y+1] } else { None };
                let w = if x > 0 { temp[x-1][y] } else { None };
                let e = if x < 8 { temp[x+1][y] } else { None };
                if let Some(v) = temp[x][y] {
                    if x == 4 && y == 4 { pegs[v] = false; r = v };
                    l[v].neighbours = vec![n,s,e,w];
                }
            }
        }
        (l,pegs,r)
    }
    fn asym_layout() -> (Vec<Cell>,Vec<bool>, usize) {
        let mut temp : [[Option<usize>;9];9] = Default::default();
        let mut i = 0;
        let mut l = Vec::new();
        let mut r = 0;
        let mut pegs = Vec::new();
        for x in 1..9 {
            for y in 0..8 {
                let cx = x < 3 || x >= 6;
                let cy = y < 3 || y >= 6;
                if !(cx && cy) {
                    temp[x][y] = Some(i);
                    i+= 1;
                    pegs.push(true);
                    l.push(Cell {
                        location: (x as i32 * 16, y as i32 * 16),
                        neighbours: vec![],
                    });
                } else {
                    temp[x][y] = None;
                }
                

            }
        }
        
        for x in 1..9 {
            for y in 0..8 {
                let n = if y > 0 { temp[x][y-1] } else { None };
                let s = if y < 8 { temp[x][y+1] } else { None };
                let w = if x > 0 { temp[x-1][y] } else { None };
                let e = if x < 8 { temp[x+1][y] } else { None };
                if let Some(v) = temp[x][y] {
                    if x == 4 && y == 4 { pegs[v] = false; r = v };
                    l[v].neighbours = vec![n,s,e,w];
                }
            }
        }
        (l,pegs,r)
    }
    fn french_layout() -> (Vec<Cell>,Vec<bool>, usize) {
        let temp : [[Option<usize>;7];7] = [
            [None, None, Some(0), Some(1), Some(2), None, None],
            [None, Some(3),Some(4),Some(5),Some(6),Some(7),None],
            [Some(8),Some(9),Some(10),Some(11),Some(12),Some(13),Some(14)],
            [Some(15),Some(16),Some(17),Some(18),Some(19),Some(20),Some(21)],
            [Some(22),Some(23),Some(24),Some(25),Some(26),Some(27),Some(28)],
            [None, Some(29),Some(30),Some(31),Some(32),Some(33),None],
            [None, None, Some(34), Some(35), Some(36), None, None],
        ];
        let mut l = Vec::new();
        let mut pegs = Vec::new();
        for y in 0..7 {
            for x in 0..7 {
                if let Some(_) = temp[x][y] {
                    pegs.push(true);
                    l.push(Cell {
                        location: (x as i32 * 16+16, y as i32 * 16+16),
                        neighbours: vec![],
                    });
                }
            }
        }
        
        for x in 0..7 {
            for y in 0..7 {
                let n = if y > 0 { temp[x][y-1] } else { None };
                let s = if y < 6 { temp[x][y+1] } else { None };
                let w = if x > 0 { temp[x-1][y] } else { None };
                let e = if x < 6 { temp[x+1][y] } else { None };
                if let Some(v) = temp[x][y] {
                    if v == 11 { pegs[v] = false; };
                    l[v].neighbours = vec![n,s,e,w];
                }
            }
        }
        (l,pegs,25)
    }
    fn diamond_layout() -> (Vec<Cell>,Vec<bool>, usize) {
        let temp : [[Option<usize>;9];9] = [
            [None, None, None, None,Some(0),None, None, None,None],
            [None, None, None, Some(1), Some(2), Some(3), None, None,None],
            [None, None, Some(4),Some(5),Some(6),Some(7),Some(8),None,None,],
            [None, Some(9),Some(10),Some(11),Some(12),Some(13), Some(14),Some(15), None],
            [Some(16),Some(17),Some(18),Some(19), Some(20),Some(21), Some(22),Some(23),Some(24)],
            [None,Some(25),Some(26),Some(27),Some(28), Some(29),Some(30),Some(31),None],
            [None, None, Some(32),Some(33),Some(34),Some(35),Some(36),None, None],
            [None, None, None,  Some(37), Some(38), Some(39), None, None, None],
            [None, None, None,  None, Some(40),None, None, None, None],
        ];
        let mut l = Vec::new();
        let mut pegs = Vec::new();
        for y in 0..9 {
            for x in 0..9 {
                if let Some(_) = temp[x][y] {
                    pegs.push(true);
                    l.push(Cell {
                        location: (x as i32 * 16, y as i32 * 16),
                        neighbours: vec![],
                    });
                }
            }
        }
        
        for x in 0..9 {
            for y in 0..9 {
                let n = if y > 0 { temp[x][y-1] } else { None };
                let s = if y < 8 { temp[x][y+1] } else { None };
                let w = if x > 0 { temp[x-1][y] } else { None };
                let e = if x < 8 { temp[x+1][y] } else { None };
                if let Some(v) = temp[x][y] {
                    if v == 20 { pegs[v] = false; };
                    l[v].neighbours = vec![n,s,e,w];
                }
            }
        }
        (l,pegs,20)
    }
    fn triangle_layout() -> (Vec<Cell>,Vec<bool>, usize) {
        let temp : [[Option<usize>;9];5] = [
            [None, None, None, None,Some(0),None, None, None,None],
            [None, None,None, Some(1), None, Some(2), None, None,None],
            [None, None, Some(3),None ,Some(4),None,Some(5),None,None,],
            [None, Some(6),None,Some(7),None,Some(8), None,Some(9), None],
            [Some(10),None,Some(11),None, Some(12),None, Some(13),None,Some(14)],            
        ];
        let mut l = Vec::new();
        let mut pegs = Vec::new();
        for y in 0..5 {
            for x in 0..9 {
                if let Some(_) = temp[y][x] {
                    pegs.push(true);
                    l.push(Cell {
                        location: (x as i32 * 8+32, y as i32 * 16+32),
                        neighbours: vec![],
                    });
                }
            }
        }
        
        for x in 0..9 {
            for y in 0..5 {
                let  d1 = if x > 0 && y > 0 { temp[y-1][x-1] } else { None };
                let  d2 = if x < 8 && y > 0 { temp[y-1][x+1] } else { None };
                let  d3 = if x > 1 { temp[y][x-2] } else { None };
                let  d4 = if x < 7 { temp[y][x+2] } else { None };
                let  d5 = if x > 0 && y < 4 { temp[y+1][x-1] } else { None };
                let  d6 = if x < 8 && y < 4 { temp[y+1][x+1] } else { None };

                if let Some(v) = temp[y][x] {
                    if v == 0 { pegs[v] = false; };
                    l[v].neighbours = vec![d1,d2,d3,d4,d5,d6];
                }
            }
        }
        (l,pegs,0)
    }
    fn new(lay: Layout) -> Table {
        let (layout, pegs,r) = match lay {
            Layout::Wiegleb => Self::wiegleb_layout(),
            Layout::Asymmetrical => Self::asym_layout(),
            Layout::French => Self::french_layout(),
            Layout::Diamond => Self::diamond_layout(),
            Layout::Triangular => Self::triangle_layout(),
            _ => Self::english_layout(),
        } ;
        let table = Table {
            animations: Vec::new(),
            game_over: None, history: vec![],
            layout, pegs, goal_peg: r,
            move_count: 0,
        };        
        table
    }
    fn draw<'t>(&self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'t>>) {
        
        for i in 0..self.layout.len() {
            let c = &self.layout[i];
            
            graphics.cell[if i == self.goal_peg {2} else {0}].draw(canvas,(c.location.0 + OFFSET.0, c.location.1 + OFFSET.1));
        }
        for i in 0..self.pegs.len() {
            if self.pegs[i] {
                let c = &self.layout[i];
                draw_peg(canvas, graphics, (c.location.0 + OFFSET.0, c.location.1 + OFFSET.1))
            }
        }
        for (x,i) in &self.animations {
            x.draw(canvas,graphics, *i)
        }
    }
    fn check_won(&mut self) {
        let mut available_moves = false;
        let mut other_pegs = false;
        for i in 0..self.layout.len() {
            if self.pegs[i] {
                if i != self.goal_peg {
                    other_pegs = true;
                }
                if self.can_make_move(i).is_some() {
                    available_moves = true;
                }
            }
        }
        if available_moves {
            self.game_over = None;
        } else if other_pegs {
            self.game_over = Some(EndGame::Failure);
        } else {
            self.game_over = Some(EndGame::Victory);
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
    fn can_pick_up(&self, v: usize) -> bool {
        if self.animations.len() > 0 { 
            return false
        }
        self.pegs[v]
    }
    fn pick_up(&mut self,v: usize) {
        self.pegs[v] = false;
    }
    fn can_make_move(&mut self, origin: usize) -> Option<usize> {
        //todo
        //if self.pegs[origin] {
        for d in 0..self.layout[origin].neighbours.len() {
            if let Some(x) = self.layout[origin].neighbours[d] {
                if self.pegs[x] {
                    if let Some(dest) = self.layout[x].neighbours[d] {
                        if !self.pegs[dest] {
                            return Some(dest);
                        }
                    }
                    
                }
            }
        }
        //}
        None
    }
    fn make_move(&mut self, origin: usize, dest: usize) -> bool {
        if !self.pegs[dest] {
            for d in 0..self.layout[origin].neighbours.len() {
                if let Some(x) = self.layout[origin].neighbours[d] {
                    if self.pegs[x] && self.layout[x].neighbours[d] == Some(dest) {
                        self.pegs[dest] = true;
                        self.pegs[x] = false;
                        self.pegs[origin] = false;
                        self.move_count += 1;
                        self.history.push((origin,x,dest));
                        self.check_won();
                        return true;
                    }
                }
            }
        }
        false
    }
    
    fn collides(&self, position: (i32,i32)) -> Option<(usize,(i32,i32))> {
        for i in 0..self.layout.len() {
            let c = &self.layout[i];
            if   position.0 >= c.location.0 + OFFSET.0 && position.0 < c.location.0 + 16 + OFFSET.0 
              && position.1 >= c.location.1 + OFFSET.1 && position.1 < c.location.1 + 16 + OFFSET.1 {
                let offset_x = (c.location.0 + OFFSET.0) - position.0;
                let offset_y = (c.location.1 + OFFSET.1) - position.1;
                return Some((i, (offset_x,offset_y) ))
            }
        }
        return None;
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameObject {
    Well(usize)
}
const WIDTH : u32 = 196;
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
    let mut layout = Layout::English;
    let mut table = Table::new(layout);
    let mut mx = 0;
    let mut my = 0;
    let mut md = false;
    let mut lose_gfx = Graphic::load_from(Cursor::new(&include_bytes!("../lose")[..])).unwrap().textured(&texture_creator);
    let mut win_gfx = Graphic::load_from(Cursor::new(&include_bytes!("../win")[..])).unwrap().textured(&texture_creator);
    lose_gfx.update_texture(&graphics_set.tile_set);
    win_gfx.update_texture(&graphics_set.tile_set);
    
    let mut attached : Option<usize> = None;    
    let mut grab_offset : (i32,i32) = (0,0);    
    let mut move_count_gfx = Graphic::blank(4,1).textured(&texture_creator);
    let mut move_count_gfx_shadow = Graphic::blank(4,1).textured(&texture_creator);
    
    let mut menu = MenuBar::new(WIDTH)
                    .add(Menu::new("GAME",152-(3*8),&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("English", 352, Keycode::F1,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("French", 353, Keycode::F2,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Wiegleb", 354, Keycode::F3,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Asymmetrical", 355, Keycode::F4,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Diamond", 356, Keycode::F5,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Triangular", 357, Keycode::F6,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(136-(3*8), &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Undo",27, Keycode::Z,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Restart",15, Keycode::N,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(136-(3*8), &texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Quit",363, Keycode::F12,&texture_creator,&graphics_set.tile_set)))
                    .add(Menu::new("VIEW",112,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Micro-mode",360, Keycode::F9,&texture_creator, &graphics_set.tile_set)));
    let mut rate_limiter = FPSManager::new();
    rate_limiter.set_framerate(60).unwrap();
    let mut micro_mode = false;
    loop {
        canvas.set_draw_color(CHARCOAL);
        canvas.clear();
        table.draw(&mut canvas, &graphics_set);

        move_count_gfx.draw_rect(0, 0, 4, 1, Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:0});
        move_count_gfx.draw_text(&table.move_count.to_string(), &graphics_set.tile_set , 0, 0, WHITE, TRANSPARENT);
        move_count_gfx_shadow.draw_rect(0, 0, 4, 1, Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:0});
        move_count_gfx_shadow.draw_text(&table.move_count.to_string(), &graphics_set.tile_set , 0, 0, BLACK, TRANSPARENT);
        move_count_gfx.update_texture(&graphics_set.tile_set);
        move_count_gfx_shadow.update_texture(&graphics_set.tile_set);
        move_count_gfx_shadow.draw(&mut canvas, (10,240-9-8));
        move_count_gfx_shadow.draw(&mut canvas, (10,240-11-8));
        move_count_gfx_shadow.draw(&mut canvas, (9,240-10-8));
        move_count_gfx_shadow.draw(&mut canvas, (11,240-10-8));
        move_count_gfx.draw(&mut canvas, (10,HEIGHT as i32-10-8));
        if let Some(_) = &attached {
            draw_peg(&mut canvas,&graphics_set, (mx + grab_offset.0,my + grab_offset.1));
        }

        let won = table.game_over.is_some();
        if let Some(e) = table.game_over {
            match e {
                EndGame::Victory => {    
                    win_gfx.draw(&mut canvas, (32, 4 * 16 - (21*4) + 17+48 ));            
                }, 
                EndGame::Failure => {
                    lose_gfx.draw(&mut canvas, (32, 4 * 16 - (21*4) + 17+48 ));
                }
            }
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
                            return;
                        },
                        Event::KeyDown { keycode: Some(Keycode::F1), ..} => {
                            layout = Layout::English;
                            table = Table::new(layout);
                            attached = None; 
                            md = false;
                        },
                        Event::KeyDown { keycode: Some(Keycode::F2), ..} => {
                            layout = Layout::French;
                            table = Table::new(layout);
                            attached = None; 
                            md = false;
                        },
                        Event::KeyDown { keycode: Some(Keycode::F3), ..} => {
                            layout = Layout::Wiegleb;
                            table = Table::new(layout);
                            attached = None; 
                            md = false;
                        },
                        Event::KeyDown { keycode: Some(Keycode::F4), ..} => {
                            layout = Layout::Asymmetrical;
                            table = Table::new(layout);
                            attached = None; 
                            md = false;
                        },
                        Event::KeyDown { keycode: Some(Keycode::F5), ..} => {
                            layout = Layout::Diamond;
                            table = Table::new(layout);
                            attached = None; 
                            md = false;
                        },
                        Event::KeyDown { keycode: Some(Keycode::F6), ..} => {
                            layout = Layout::Triangular;
                            table = Table::new(layout);
                            attached = None; 
                            md = false;
                        },
                        Event::KeyDown { keycode: Some(Keycode::F7), ..} => {
                            table = Table::new(layout);
                            attached = None; 
                            md = false;
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
                        
                        Event::KeyDown { keycode: Some(Keycode::Z), ..} => {
                            table.undo()
                        },
                        Event::KeyDown {..} if won => {
                            table = Table::new(layout);
                            attached = None;
                            md = false;
                        },
                        Event::MouseButtonUp { ..} if won => {
                            table = Table::new(layout);
                            attached = None;
                            md = false;
                        },
                        Event::KeyDown { keycode: Some(Keycode::N), ..} => {
                            table = Table::new(layout);
                            attached = None;
                            md = false;
                        },
                        Event::MouseButtonDown { mouse_btn: _, x, y, ..} if !won => {
                            let (sx,sy) = (x,y);//(x * dw as i32 / ww as i32, y * dh as i32 / wh as i32);
                            md = true;
                            mx = sx; my = sy;
                            if let Some((v, offset)) = table.collides((mx,my)) {
                                if table.can_pick_up(v) {
                                    table.pick_up(v);
                                    attached = Some(v);
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
                            if let Some(origin) = attached {
                                if !md {
                                    if let Some((v,_)) = table.collides((mx,my)) {
                                        if origin != v {
                                            if table.make_move(origin,v) {
                                                placed = true;
                                            }
                                        }   
                                    }
                                }
                                if !placed {
                                    if md {
                                        md = false;
                                        
                                        if let Some(dest) = table.can_make_move(origin) {
                                            let pos2 = (table.layout[origin].location.0 + OFFSET.0, table.layout[origin].location.1 + OFFSET.1);
                                            let pos1 = (table.layout[dest].location.0 + OFFSET.0, table.layout[dest].location.1 + OFFSET.1);
                                            table.animate_peg(pos1,pos2,Box::new(move |tbl| { tbl.make_move(origin,dest);}))
                                        } else {
                                            table.add_peg_to(origin);
                                        }
                                    } else {
                                        let pos = (table.layout[origin].location.0 + OFFSET.0, table.layout[origin].location.1 + OFFSET.1);
                                        table.animate_peg(pos, (mx + grab_offset.0,my + grab_offset.1),Box::new(move |tbl| { tbl.add_peg_to(origin)}));
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
    let window = video_subsystem.window("peggy", 320, 240)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    main_loop(window,&sdl_context);
    
    //cards::main_loop::<Spider<OneSuit>>();
    
}