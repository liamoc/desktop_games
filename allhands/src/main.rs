extern crate tesserae;
extern crate sdl2;

use rand::seq::SliceRandom;
use std::collections::VecDeque;
use rand::Rng;
use tesserae::{*};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::gfx::framerate::FPSManager;
use sdl2::render::TextureCreator;
use sdl2::render::Texture;
use sdl2::render::RenderTarget;
use sdl2::mouse::MouseButton;
use sdl2::render::Canvas;
use utils::color::{*};
use sdl2::rect::{Rect};
use utils::menu::{*};
use rand::thread_rng;
use std::io::Cursor;



#[derive(Eq,PartialEq,Debug,Copy,Clone)]
struct CellContents {
    piece: PieceType,
    part: usize,
    vertical: bool
}
#[derive(Eq,PartialEq,Debug,Copy,Clone)]
struct Cell {
    checked: bool,
    contents: Option<CellContents>
}
#[derive(Eq,PartialEq,Debug,Copy,Clone)]
enum PieceType {
    Carrier, Battleship, Cruiser, Destroyer, PatrolBoat, Sub
}
#[derive(Eq,PartialEq,Debug,Copy,Clone)]
enum Status {
    Placing, PlayerMove, PlayerBomb, CPUBomb, CPUMove, GameOver
}

impl PieceType {
    const ALL : [PieceType;6] = [Self::Carrier, Self::Battleship, Self::Cruiser, Self::Destroyer, Self::PatrolBoat, Self::Sub];
    fn name(&self) -> &'static str {
        match *self {
            Self::Carrier => "CARRIER",
            Self::Battleship => "BATTLESHIP",
            Self::Cruiser => "CRUISER",
            Self::Destroyer => "DESTROYER",
            Self::PatrolBoat => "PATROL BOAT",
            Self::Sub => "SUBMARINE",
        }
    }
    fn from_index(x : usize) -> Option<PieceType> {
        match x {
            0 => Some(Self::Carrier),
            1 => Some(Self::Battleship),
            2 => Some(Self::Cruiser),
            3 => Some(Self::Destroyer),
            4 => Some(Self::PatrolBoat),
            5 => Some(Self::Sub),
            _ => None
        }
    }
    fn index(&self) -> usize {
        match *self {
            Self::Carrier => 0,
            Self::Battleship => 1,
            Self::Cruiser => 2,
            Self::Destroyer => 3,
            Self::PatrolBoat => 4,
            Self::Sub => 5,
        }
    }
    fn length(&self) -> usize {
        match *self {
            Self::Carrier => 5,
            Self::Battleship => 4,
            Self::Cruiser => 3,
            Self::Destroyer => 3,
            Self::PatrolBoat => 2,
            Self::Sub => 1,
        }
    }
}

struct ShipDisplay<'r> {
    graphic: Graphic<Texture<'r>>,
    color: Color
}
impl <'r> ShipDisplay<'r> {
    fn new<T>(chrome: &Graphic<Texture<'r>>, color: Color, tile_set: &TileSet, texture_creator : &'r TextureCreator<T>) -> ShipDisplay<'r> {
        let mut g=Graphic::blank(20,6).textured(texture_creator);
        g.copy_tiles_from(chrome,1,24,20,6,0,0);
        for i in 0..20 {
            for j in 0..6 {
                g[(i,j)].fg = TRANSPARENT;
                g[(i,j)].bg = TRANSPARENT;
            }
        }
        g.update_texture(&tile_set);
        ShipDisplay {
            color,
            graphic:g
        }
    }
    fn clear_illumination(&mut self) {
        for i in 0..20 {
            for j in 0..6 {
                self.graphic[(i,j)].fg = TRANSPARENT;
            }
        }
    }
    fn cover_everything(&mut self, tile_set:&TileSet) {
        self.graphic.draw_rect(0,0,20,6,Tile{index:0,fg:DARK_RED,bg:TRANSPARENT});
        self.graphic.draw_text("FOG OF WAR",tile_set,5,2,DARK_RED,TRANSPARENT);
    }
    fn rect_for(ship: PieceType) -> ((u32,u32),(u32,u32)) {
        match ship {
            PieceType::Carrier => ((1,1),(8,2)),
            PieceType::Battleship => ((1,3),(8,4)),
            PieceType::Sub => ((9,1),(12,2)),
            PieceType::PatrolBoat => ((9,3),(12,4)),
            PieceType::Destroyer => ((13,1),(19,2)),
            PieceType::Cruiser=> ((13,3),(19,4)),
        }
    }
    fn illuminate(&mut self, ship: PieceType) {
        let (s,e) = Self::rect_for(ship);
        for i in s.0..=e.0 {
            for j in s.1..=e.1 {
                self.graphic[(i,j)].fg = self.color;
            }
        }
    }
    fn update_texture(&mut self, tile_set: &TileSet) {
        self.graphic.update_texture(tile_set);
    }
    fn draw<T:RenderTarget>(&self, c : &mut Canvas<T>, pos:(i32,i32)) {
        self.graphic.draw(c, pos);
    }
    fn highlight(&mut self, ship: PieceType) {
        let (s,e) = Self::rect_for(ship);
        for i in s.0..=e.0 {
            for j in s.1..=e.1 {
                self.graphic[(i,j)].fg = WHITE;
            }
        }

    }
}

struct Game<'r,C> {
    chrome : Graphic<Texture<'r>>,
    tile_set: TileSet,
    cursor_x: i32,
    cursor_y: i32,
    mdx: i32,
    mdy: i32,
    difficulty:u32,
    player_field: [[Cell;10];10],
    cpu_field : [[Cell;10];10],
    player_cursor: Graphic<Texture<'r>>,
    cpu_cursor : Graphic<Texture<'r>>,
    player_ship_disp: ShipDisplay<'r>,
    cpu_ship_disp: ShipDisplay<'r>,
    status: Status,
    placing_ship: PieceType,
    placing_vertical: bool,
    cpu_ships: Vec<PieceType>,
    player_ships: Vec<PieceType>,
    current_tile: Option<(usize,usize)>,
    message_queue: VecDeque<(String, Color, u32)>,
    status_message: Graphic<TileCache<'r,C>>,
    ship_drawer: Graphic<TileCache<'r,C>>,
    overlay_drawer: Graphic<TileCache<'r,C>>,
    cursor_rmb: bool,
    ticks: u32,
    bomb_coords: (usize,usize)
}

impl <'r,T: 'r> Game<'r,T> {
    const HIBLU : Color = rgba(114,159,207,255);
    const HIRED : Color = rgba(204,0,0,255);

    fn cursor_graphic(c : Color, tile_set : &TileSet, texture_creator : &'r TextureCreator<T>) -> Graphic<Texture<'r>> {
        let mut g = Graphic::blank(2,2).textured(texture_creator);
        g[(0,0)] = Tile {index: 108, fg: TRANSPARENT, bg: c};
        g[(1,0)] = Tile {index: 109, fg: TRANSPARENT, bg: c};
        g[(0,1)] = Tile {index: 124, fg: TRANSPARENT, bg: c};
        g[(1,1)] = Tile {index: 125, fg: TRANSPARENT, bg: c};
        g.update_texture(tile_set);
        g
    }
    fn new(texture_creator : &'r TextureCreator<T>) -> Game<'r,T> {
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
        let mut chrome = Graphic::load_from(Cursor::new(&include_bytes!("../chrome")[..])).unwrap().textured(&texture_creator);
        chrome.update_texture(&tile_set);
        let player_cursor = Self::cursor_graphic(Self::HIBLU, &tile_set, texture_creator);
        let cpu_cursor = Self::cursor_graphic(Self::HIRED, &tile_set, texture_creator);
        let player_ship_disp = ShipDisplay::new(&chrome,Self::HIBLU,&tile_set,texture_creator);
        let cpu_ship_disp = ShipDisplay::new(&chrome,Self::HIRED,&tile_set,texture_creator);
        Game {
            chrome,
            tile_set,
            placing_ship: PieceType::Carrier,
            cursor_x: 0,cursor_y:0,mdx:0,mdy:0,
            status: Status::GameOver,
            player_ship_disp,cpu_ship_disp,
            player_field: [[Cell {checked:false, contents:None};10];10],
            cpu_field: [[Cell {checked:false, contents:None};10];10],
            player_cursor, cpu_cursor,
            current_tile: None,
            cursor_rmb: false,
            ticks:0,
            difficulty:0,
            bomb_coords:(0,0),
            cpu_ships: Vec::new(),
            player_ships: Vec::new(),
            placing_vertical: false,
            message_queue: VecDeque::new(),
            status_message: Graphic::blank(80,1).tile_cache_textured(texture_creator),
            ship_drawer: Graphic::blank(10,10).tile_cache_textured(texture_creator),
            overlay_drawer: Graphic::blank(2,2).tile_cache_textured(texture_creator)
        }
    }
    
    fn can_place(field: &[[Cell;10];10], ship: PieceType, vertical: bool, x: usize, y: usize) -> bool {
        for m in 0..ship.length() {
            let cx = if vertical { x } else { x + m };
            let cy = if vertical { y + m } else { y };
            if cx >= 10 || cy >= 10 { return false; }
            if let Some(_) = field[cx][cy].contents {
                return false;
            }
        }
        true
    }
    fn draw_overlay<C:RenderTarget>(&mut self, canvas : &mut Canvas<C>, hit:bool,size: u32, color: Color, position: (i32,i32)) {

        self.overlay_drawer.draw_rect(0,0,2,2,Tile{index:0,fg:color,bg:if color == Self::HIRED { rgba(41,0,0,255)} else { TRANSPARENT }});
        self.overlay_drawer[(0,0)].index=if hit { 185} else {117};
        if size > 0 {
            self.overlay_drawer[(1,1)].index=if hit { 185} else {117};
            if size > 1 {
                self.overlay_drawer[(0,1)].index=if hit { 185} else {117};
                if size > 2 {
                    self.overlay_drawer[(1,0)].index=if hit { 185} else {117};
                }
            }
        }
        self.overlay_drawer.update_texture(&self.tile_set);
        self.overlay_drawer.draw(canvas, position);
    }
    fn draw_ship<C:RenderTarget>(&mut self, canvas : &mut Canvas<C>, ship: PieceType, vertical: bool, color: Color, position: (i32,i32)) {

        self.ship_drawer.draw_rect(0,0,10,10,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        self.ship_drawer.draw_rect(0,0,if vertical { 2 } else {ship.length() as u32*2}, if vertical {ship.length() as u32* 2} else { 2 }, Tile{index:0,fg:TRANSPARENT,bg:color} );
        self.ship_drawer[(0,0)].index = 302;
        if vertical {
            self.ship_drawer[(1,0)].index = 303;
            self.ship_drawer[(0,ship.length() as u32*2-1)].index = 287;
            self.ship_drawer[(1,ship.length() as u32*2-1)].index = 286;
        } else {
            self.ship_drawer[(0,1)].index = 287;
            self.ship_drawer[(ship.length() as u32*2-1,0)].index = 303;
            self.ship_drawer[(ship.length() as u32*2-1,1)].index = 286;
        }
        self.ship_drawer.update_texture(&self.tile_set);
        self.ship_drawer.draw(canvas, position);

    }
    fn player_place_ship(&mut self) {
        if let Some((x,y)) = self.current_tile {
            if Self::can_place(&self.player_field,self.placing_ship,self.placing_vertical,x,y) {
                for m in 0..self.placing_ship.length() {
                    let cx = if self.placing_vertical { x } else { x + m };
                    let cy = if self.placing_vertical { y + m } else { y };
                    self.player_field[cx][cy].contents = Some(CellContents { piece: self.placing_ship, part: m, vertical: self.placing_vertical});
                }
                if self.placing_ship == PieceType::Sub {
                    let dice = thread_rng().gen_range(0,6);
                    if dice % 2 == 1 {
                        self.message_queue.push_back(("Rolled a ".to_string() + &(dice + 1).to_string() + ". Player goes first.", Self::HIBLU,80));
                        self.status = Status::PlayerMove;
                    } else {
                        self.message_queue.push_back(("Rolled a ".to_string() + &(dice + 1).to_string() + ". CPU goes first.", Self::HIRED,80));
                        self.status = Status::CPUMove;
                        self.ticks = 80;
                    }
                    
                } else {
                    self.placing_ship = PieceType::from_index(self.placing_ship.index()+1).unwrap();
                }
            }

        }
    }
    fn cpu_place_ships(&mut self) {
        for t in PieceType::ALL.iter() {
            let mut x = 100;
            let mut y = 100;
            let mut v = true;
            while !Self::can_place(&self.cpu_field,*t,v,x,y) {
                x = thread_rng().gen_range(0,10);
                y = thread_rng().gen_range(0,10);
                v = thread_rng().gen_range(0,2) == 0;
            }
            for m in 0..t.length() {
                let cx = if v { x } else { x + m };
                let cy = if v { y + m } else { y };
                self.cpu_field[cx][cy].contents = Some(CellContents { piece: *t, part: m, vertical: v});
            }
        }
    }
    fn new_game<C>(&mut self, difficulty: u32, texture_creator : &'r TextureCreator<C>) {
        self.status = Status::Placing;
        self.message_queue = VecDeque::new();
        self.player_field =[[Cell {checked:false, contents:None};10];10]; 
        self.cpu_field = [[Cell {checked:false, contents:None};10];10];
        self.placing_ship = PieceType::Carrier;
        self.placing_vertical = false;
        self.difficulty = difficulty;
        self.cpu_place_ships();
        self.cpu_ship_disp = ShipDisplay::new(&self.chrome,Self::HIRED,&self.tile_set,texture_creator);
        if self.difficulty >= 2 {
            self.cpu_ship_disp.cover_everything(&self.tile_set)
        } 
    }
    fn draw<C:RenderTarget>(&mut self, canvas : &mut Canvas<C>) {
        canvas.set_draw_color(CHARCOAL);
        canvas.clear();
        self.chrome.draw(canvas,(0,17));

        for i in 0..10 {
            for j in 0..10 {
                if let Some(c) = self.player_field[i][j].contents {
                    if c.part == 0 {
                        self.draw_ship(canvas,c.piece,c.vertical,BLUE,(i as i32 *16+192,j as i32*16+8+17))
                    }
                }
                if self.status == Status::GameOver { 
                    if let Some(c) = self.cpu_field[i][j].contents {
                        if c.part == 0 {
                            self.draw_ship(canvas,c.piece,c.vertical,CRIMSON,(i as i32 *16+8,j as i32*16+8+17))
                        }
                    }
                }
            }
        }
        if self.status == Status::Placing {
            if let Some(p) = self.current_tile {
                if Self::can_place(&self.player_field,self.placing_ship,self.placing_vertical,p.0,p.1) {
                    self.draw_ship(canvas, self.placing_ship, self.placing_vertical, Self::HIBLU,(p.0 as i32*16+192,p.1 as i32*16+8+17));
                } else { 
                    self.player_cursor.draw(canvas,(p.0 as i32*16+192,p.1 as i32*16+8+17));
                }
            }
        }

        if self.status == Status::PlayerMove {
            if let Some(p) = self.current_tile {
                self.cpu_cursor.draw(canvas,(p.0 as i32*16+8,p.1 as i32*16+8+17));
            }
        }
        for i in 0..10 {
            for j in 0..10 {
                if self.player_field[i][j].checked {
                    let p = (i,j);
                    self.draw_overlay(canvas, self.player_field[i][j].contents.is_some(), 3, Self::HIBLU,(p.0 as i32*16+192,p.1 as i32*16+8+17))
                }
                if self.cpu_field[i][j].checked {
                    let p = (i,j);
                    self.draw_overlay(canvas, self.cpu_field[i][j].contents.is_some(), 3, Self::HIRED,(p.0 as i32*16+8,p.1 as i32*16+8+17))
                }
            }
        }
        if self.status == Status::PlayerBomb {
            let p = self.bomb_coords;
            self.draw_overlay(canvas, self.cpu_field[p.0][p.1].contents.is_some(), (40 - self.ticks)/10, Self::HIRED,(p.0 as i32*16+8,p.1 as i32*16+8+17))
        }
        if self.status == Status::CPUBomb {
            let p = self.bomb_coords;
            self.draw_overlay(canvas, self.player_field[p.0][p.1].contents.is_some(), (40 - self.ticks)/10, Self::HIBLU,(p.0 as i32*16+192,p.1 as i32*16+8+17))
        }
        self.player_ship_disp.draw(canvas,(192,192 + 17));
        self.cpu_ship_disp.draw(canvas,(8,192 + 17));
    }
    
    fn player_bombs(&mut self, x: usize, y:usize) {
        self.status = Status::PlayerBomb;
        self.bomb_coords = (x,y);
        self.ticks = 40;
    }
    fn cpu_bombs(&mut self, x: usize, y:usize) {
        self.status = Status::CPUBomb;
        self.bomb_coords = (x,y);
        self.ticks = 40;
    }
    fn mouse_up(&mut self) {
        if self.status == Status::Placing {
            if self.cursor_rmb {
                self.placing_vertical = !self.placing_vertical;
            } else {
                if let Some(_) = self.current_tile {
                    self.player_place_ship();
                    self.current_tile = None;
                }
            }
        }
        if self.status == Status::PlayerMove {
            if let Some(p) = self.current_tile {
                if !self.cpu_field[p.0][p.1].checked {
                    self.player_bombs(p.0,p.1);
                    self.current_tile = None;
                }
            }
        }
    }
    fn mouse_down(&mut self, rmb : bool) {
        self.mdx = self.cursor_x;
        self.mdy = self.cursor_y;
        self.cursor_rmb = rmb;
    }
    fn collides_cpu_field(x:i32, y:i32) -> Option<(usize,usize)> {
        if x >= 8 && y > 8 {
            if y <= 160 && x <= 168 {
                return Some(((x as usize-8)/16, (y as usize-8)/16));
            }
        }
        None
    }
    fn collides_player_field(x:i32, y:i32) -> Option<(usize,usize)> {
        if x >= 192 && y > 8 {
            if y <= 160 && x <= (44*8) {
                return Some(((x as usize-192)/16, (y as usize-8)/16));
            }
        }
        None
    }
    fn mouse_moved(&mut self, x : i32, y : i32) {
        self.cursor_x = x - 2;
        self.cursor_y = y - 4;
        if self.status == Status::Placing {
            self.current_tile = Self::collides_player_field(self.cursor_x,self.cursor_y);
        } else if self.status == Status::PlayerMove {
            self.current_tile = Self::collides_cpu_field(self.cursor_x,self.cursor_y);
        }
    }
    
    fn update_status(&mut self) {
        self.status_message.draw_rect(0,0,80,1,Tile{fg:TRANSPARENT,bg:TRANSPARENT,index:0});
        if self.message_queue.len() > 0 {
            let mut pop_please = false;
            if let Some((s,c,i)) = self.message_queue.front_mut() {
                self.status_message.draw_text(s, &self.tile_set,0,0,*c,TRANSPARENT);
                *i -= 1;
                if *i == 0 {
                    pop_please = true;
                }
            }
            if pop_please { self.message_queue.pop_front(); }
        } else {
            if self.status == Status::Placing {
                self.status_message.draw_text("Place your ", &self.tile_set,0,0,NEUTRAL_GRAY,TRANSPARENT);
                self.status_message.draw_text(self.placing_ship.name(),&self.tile_set,11,0,WHITE,TRANSPARENT);
                self.status_message.draw_text("(RMB to rotate)",&self.tile_set,12 + self.placing_ship.name().len() as u32,0,DARKER_GRAY,TRANSPARENT);
            }
            if self.status == Status::PlayerMove {
                self.status_message.draw_text("Choose where to strike.", &self.tile_set,0,0,Self::HIBLU,TRANSPARENT);
            }
            if self.status == Status::CPUMove {
                self.status_message.draw_text("CPU's move... thinking...", &self.tile_set,0,0,Self::HIRED,TRANSPARENT);
            }
            if self.status == Status::GameOver {
                self.status_message.draw_text("Press N to start a new game.", &self.tile_set,0,0,NEUTRAL_GRAY,TRANSPARENT);
            }
            if self.status == Status::CPUBomb {
                self.status_message.draw_text("CPU", &self.tile_set,0,0,Self::HIRED,TRANSPARENT);
                self.status_message.draw_text(" is targeting ", &self.tile_set,3,0,NEUTRAL_GRAY,TRANSPARENT);
                let s = Self::display_coords(self.bomb_coords.0, self.bomb_coords.1);
                self.status_message.draw_text(&s, &self.tile_set,17,0,WHITE,TRANSPARENT);
                self.status_message.draw_text(". ", &self.tile_set,17 + s.len() as u32,0,NEUTRAL_GRAY,TRANSPARENT);
                if self.player_field[self.bomb_coords.0][self.bomb_coords.1].contents.is_some() {
                    self.status_message.draw_text("Hit! ", &self.tile_set,17 + s.len() as u32 + 2,0,Self::HIRED,TRANSPARENT);
                } else {
                    self.status_message.draw_text("Miss! ", &self.tile_set,17 + s.len() as u32 + 2,0,Self::HIBLU,TRANSPARENT);
                }
            }
            if self.status == Status::PlayerBomb {
                self.status_message.draw_text("Player", &self.tile_set,0,0,Self::HIBLU,TRANSPARENT);
                self.status_message.draw_text(" is targeting ", &self.tile_set,6,0,NEUTRAL_GRAY,TRANSPARENT);
                let s = Self::display_coords(self.bomb_coords.0, self.bomb_coords.1);
                self.status_message.draw_text(&s, &self.tile_set,20,0,NEUTRAL_GRAY,TRANSPARENT);
                self.status_message.draw_text(". ", &self.tile_set,20 + s.len() as u32,0,NEUTRAL_GRAY,TRANSPARENT);
                if self.cpu_field[self.bomb_coords.0][self.bomb_coords.1].contents.is_some() {
                    self.status_message.draw_text("Hit! ", &self.tile_set,20 + s.len() as u32 + 2,0,Self::HIBLU,TRANSPARENT);
                } else {
                    self.status_message.draw_text("Miss! ", &self.tile_set,20 + s.len() as u32 + 2,0,Self::HIRED,TRANSPARENT);
                }
            }
        }
        self.status_message.update_texture(&self.tile_set);
    }
    fn tick(&mut self) {
        let mut old_cpu_ships = self.cpu_ships.clone();
        let mut old_player_ships = self.player_ships.clone();
        self.cpu_ships = Vec::new();
        self.player_ships = Vec::new();
        self.player_ship_disp.clear_illumination();
        if self.difficulty < 2 { self.cpu_ship_disp.clear_illumination(); }
        for i in 0..10 {
            for j in 0..10 {
                if let Some(contents) = self.player_field[i][j].contents {
                    if !self.player_field[i][j].checked {
                        self.player_ship_disp.illuminate(contents.piece);
                        self.player_ships.push(contents.piece);
                    }
                }
                if let Some(contents) = self.cpu_field[i][j].contents {
                    if !self.cpu_field[i][j].checked {
                        if self.difficulty < 2 { 
                            self.cpu_ship_disp.illuminate(contents.piece);
                        }
                        self.cpu_ships.push(contents.piece);
                    }
                }
            }
        }
        if self.status != Status::Placing && self.status != Status::GameOver {
            if self.difficulty < 2 {
                old_cpu_ships =  old_cpu_ships.drain(..).filter(|x| !self.cpu_ships.contains(x)).collect();
                for i in old_cpu_ships {
                    self.message_queue.push_back(("You sunk their ".to_string() + i.name() + "!", Self::HIBLU,60));
                }
            }
            old_player_ships = old_player_ships.drain(..).filter(|x| !self.player_ships.contains(x) ).collect();
            for i in old_player_ships {
                self.message_queue.push_back(("They sunk your ".to_string() + i.name() + "!", Self::HIRED,60));
            }
            if self.player_ships.len() == 0 {
                self.message_queue.push_back(("Game over, you lost!".to_string(), Self::HIRED,80));
                self.status = Status::GameOver;
            } else if self.cpu_ships.len() == 0 {
                self.message_queue.push_back(("Game over, you won!".to_string(), Self::HIBLU,80));
                self.status = Status::GameOver;
            }
        }
        self.update_status();
        if self.status == Status::Placing {
            self.player_ship_disp.highlight(self.placing_ship);            
        } else if self.status == Status::CPUMove {
            self.ticks -= 1;
            if self.ticks == 0 {
                let mov = self.cpu_get_move();
                self.cpu_bombs(mov.0,mov.1)
            }
        } else if self.status == Status::PlayerBomb {
            self.ticks -= 1;
            if self.ticks == 0 {
                self.cpu_field[self.bomb_coords.0][self.bomb_coords.1].checked = true;
                self.status = Status::CPUMove;
                self.ticks = 40;
            } 
        } else if self.status == Status::CPUBomb {
            self.ticks -= 1;
            if self.ticks == 0 {
                self.player_field[self.bomb_coords.0][self.bomb_coords.1].checked = true;
                self.status = Status::PlayerMove;
            } 
        }
        self.cpu_ship_disp.update_texture(&self.tile_set);
        self.player_ship_disp.update_texture(&self.tile_set);

    }
    fn is_unchecked(&self,x:i32,y:i32) -> Option<(usize,usize)> {
        if x >= 0 && x < 10 {
            if y >= 0 && y < 10 {
                let i = x as usize;
                let j = y as usize;
                if !self.player_field[i][j].checked {
                    return Some((i,j))
                }
            }
        }
        None
    }
    fn cpu_get_move(&self) -> (usize,usize) {
        if self.difficulty > 0 {
            let mut known_hits = Vec::new();
            let mut candidates = Vec::new();
            for i in 0..10 as i32 {
                for j in 0..10 as i32 {
                    if self.player_field[i as usize][j as usize].checked && self.player_field[i as usize][j as usize].contents.is_some() {
                        known_hits.push((i,j));
                    }
                }
            }
            //find lines
            for (i,j) in &known_hits {
                if known_hits.contains(&(*i-1,*j)) {
                    if let Some(p) = self.is_unchecked(*i-2, *j) {
                        candidates.push(p)
                    }
                } 
                if known_hits.contains(&(*i,*j-1)) {
                    if let Some(p) = self.is_unchecked(*i, *j-2) {
                        candidates.push(p)
                    }
                } 
                if known_hits.contains(&(*i+1,*j)) {
                    if let Some(p) = self.is_unchecked(*i+2, *j) {
                        candidates.push(p)
                    }
                } 
                if known_hits.contains(&(*i,*j+1)) {
                    if let Some(p) = self.is_unchecked(*i, *j+2) {
                        candidates.push(p)
                    }
                } 
            } 
            if candidates.len() == 0 {
                for (i,j) in &known_hits {
                    if     !known_hits.contains(&(*i-1,*j)) 
                        && !known_hits.contains(&(*i+1,*j))
                        && !known_hits.contains(&(*i,*j-1))
                        && !known_hits.contains(&(*i,*j+1)) {
                        if let Some(p) = self.is_unchecked(*i+1, *j) {
                            candidates.push(p)
                        }
                        if let Some(p) = self.is_unchecked(*i-1, *j) {
                            candidates.push(p)
                        }
                        if let Some(p) = self.is_unchecked(*i, *j-1) {
                            candidates.push(p)
                        }
                        if let Some(p) = self.is_unchecked(*i, *j+1) {
                            candidates.push(p)
                        }
                    }
                } 
            }
            if candidates.len() > 0 {
                candidates.shuffle(&mut thread_rng());
                return *candidates.first().unwrap();
            }
        }
        let mut x = 1000;
        let mut y = 1000;
        while x >= 10 || y >= 10 || self.player_field[x][y].checked {
            x = thread_rng().gen_range(0,10);
            y = thread_rng().gen_range(0,10);
        } 
        return (x,y);
    }
    fn display_coords(x:usize,y:usize) -> String {
        match y {
            0 => "A", 1 => "B", 2=>"C",
            3 => "D", 4 => "E", 5=>"F",
            6 => "G", 7 => "H", 8=>"I",
            9 => "J", _ => "?"
        }.to_string() + &(x + 1).to_string()
    }
}
const WIDTH : u32 =45;
const HEIGHT : u32 =31;
fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.text_input().start();
    let window = video_subsystem.window("all hands", WIDTH*8, HEIGHT*8 + 16 + 17)
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
    let mut difficulty = 1;
    //game.new_game(1, &texture_creator);
    rate_limiter.set_framerate(40).unwrap();
    let mut micro_mode = false;
    let mut menu = MenuBar::new(WIDTH*8)
                    .add(Menu::new("GAME",96,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Easy", 352, Keycode::F1,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Medium", 353, Keycode::F2,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Hard", 354, Keycode::F3,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(80, &texture_creator,&game.tile_set))
                            .add(MenuItem::new("Restart", 15,Keycode::N,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(80, &texture_creator,&game.tile_set))
                            .add(MenuItem::new("Quit", 363, Keycode::F12,&texture_creator,&game.tile_set)))
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
        game.status_message.draw(&mut canvas,(4, HEIGHT as i32*8 + 21));
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
                Event::MouseButtonDown {y,mouse_btn,..} if y > 17 => {
                    game.mouse_down(mouse_btn == MouseButton::Right)
                }
                Event::KeyDown{ keycode: Some(Keycode::F1),..} => {
                    difficulty = 0;
                    game.new_game(0,&texture_creator);
                }
                Event::KeyDown{ keycode: Some(Keycode::F2),..} => {
                    difficulty = 1;
                    game.new_game(1,&texture_creator);
                }
                Event::KeyDown{ keycode: Some(Keycode::F3),..} => {
                    difficulty = 2;
                    game.new_game(2,&texture_creator);
                }
                Event::KeyDown{ keycode: Some(Keycode::N),..} => {
                    game.new_game(difficulty,&texture_creator);
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


