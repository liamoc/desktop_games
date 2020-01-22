extern crate tesserae;
extern crate sdl2;


use rand::Rng;
use tesserae::{*};
use sdl2::render::Texture;
use sdl2::render::RenderTarget;
use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::gfx::framerate::FPSManager;
use sdl2::render::TextureCreator;
use sdl2::render::Canvas;
use utils::color::{*};
use sdl2::rect::{Rect};
use utils::menu::{*};
use rand::thread_rng;
use std::io::Cursor;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Direction { NW, SW, NE, SE }
impl Direction {
    fn x_velocity(&self) -> i32 {
        match *self {
            Self::NW => -1,
            Self::SW => -1,
            _ => 1
        }
    }
    fn y_velocity(&self) -> i32 {
        match *self {
            Self::NW => -1,
            Self::NE => -1,
            _ => 1
        }
    }
    fn reflect_y(&self) -> Direction {
        match *self {
            Self::NW => Self::SW,
            Self::NE => Self::SE,
            Self::SE => Self::NE, 
            Self::SW => Self::NW
        }
    }
    fn reflect_x(&self) -> Direction {
        match *self {
            Self::NW => Self::NE,
            Self::NE => Self::NW,
            Self::SE => Self::SW, 
            Self::SW => Self::SE
        }
    }
    fn move_point (&self, p: (i32,i32)) -> (i32,i32) {
        match *self {
            Self::NW => (p.0 - 1, p.1 - 1),
            Self::SW => (p.0 - 1, p.1 + 1),
            Self::NE => (p.0 + 1, p.1 - 1),
            Self::SE => (p.0 + 1, p.1 + 1),
        }
    }
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Orthogonal { N, S, E, W }
impl Orthogonal {
    fn move_point (&self, p:(i32,i32),v: i32) -> (i32,i32) {
        match *self {
            Self::N => (p.0,p.1-v),
            Self::S => (p.0,p.1+v),
            Self::E => (p.0+v,p.1),
            Self::W => (p.0-v,p.1),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum CollisionType { Wall, Ball(usize), WallBuilder(bool)}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Collision {
    collision_type: CollisionType,
    sides:(bool,bool,bool,bool)
}
struct Game<'r> {
    board: Graphic<Texture<'r>>,
    ball_g1: Graphic<Texture<'r>>,
    ball_g2: Graphic<Texture<'r>>,
    cursor_g1: Graphic<Texture<'r>>,
    cursor_g2: Graphic<Texture<'r>>,
    overlay: Graphic<()>,
    blocked: [[bool;HEIGHT as usize];WIDTH as usize],
    tile_set: TileSet,
    subtick: u32,
    balls: Vec<Ball>,
    cursor_vert: bool,
    cursor_x: i32,
    cursor_y: i32,
    red: Option<WallBuilder>,
    blue: Option<WallBuilder>,
    wb_n: (Graphic<Texture<'r>>, Graphic<Texture<'r>>, Graphic<Texture<'r>>),
    wb_s: (Graphic<Texture<'r>>, Graphic<Texture<'r>>, Graphic<Texture<'r>>),
    wb_e: (Graphic<Texture<'r>>, Graphic<Texture<'r>>, Graphic<Texture<'r>>),
    wb_w: (Graphic<Texture<'r>>, Graphic<Texture<'r>>, Graphic<Texture<'r>>),
    overlay_tiles: (Graphic<Texture<'r>>, Graphic<Texture<'r>>, Graphic<Texture<'r>>,Graphic<Texture<'r>>, Graphic<Texture<'r>>,Graphic<Texture<'r>>),
    percentage: u32,
    lives: u32,
    speed: u32,
}


const UNREVEAL_BG : Color = CHARCOAL;
const EDGE_HI : Color = rgba(85,87,83,255);
const EDGE_LO : Color = BLACK ;//rgba(92,52,102,255);
const WIDTH : u32 = 28;
const HEIGHT : u32 = 20;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Ball {
    p : (i32,i32),
    d : Direction,
    reflect_x : bool,
    reflect_y : bool    
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct WallBuilder {
    p : (i32,i32),
    size: i32,
    direction: Orthogonal    
}
impl WallBuilder {
    fn occlusion_rect(&self) -> (i32,i32,i32,i32) {
        let (bx,by) = (self.p.0*4,self.p.1*4);
        match self.direction {
            Orthogonal::N =>
                (bx,by-self.size*2,bx+3,by+3),
            Orthogonal::S =>
                (bx,by,bx+3,by+3+self.size*2),
            Orthogonal::W =>
                (bx-self.size*2,by,bx+3,by+3),
            Orthogonal::E =>
                (bx,by,bx+3+self.size*2,by+3),
        }        
    }
}
impl Ball {
    fn new(x:i32,y:i32,d:Direction) -> Ball {
        Ball {
            p:(x,y),d,reflect_x:false,reflect_y:false
        }
    }
    fn collide(&mut self, col : Collision) {
        let mut vec = (0,0);
        let (ul,ur,ll,lr) = col.sides;
        if ul { vec.0 += 1; vec.1 +=1 };
        if ur { vec.0 -= 1; vec.1 +=1 };
        if ll { vec.0 += 1; vec.1 -=1 };
        if lr { vec.0 -= 1; vec.1 -=1 };        
        if vec.0 > 0 && self.d.x_velocity() < 0 { self.reflect_x = true } 
        if vec.0 < 0 && self.d.x_velocity() > 0 { self.reflect_x = true } 
        if vec.1 > 0 && self.d.y_velocity() < 0 { self.reflect_y = true } 
        if vec.1 < 0 && self.d.y_velocity() > 0 { self.reflect_y = true } 
    }
    fn go_forward(&mut self) {
        if self.reflect_x {
            self.reflect_x = false;
            self.d = self.d.reflect_x()
        }
        if self.reflect_y {
            self.reflect_y = false;
            self.d = self.d.reflect_y()
        }
        self.p = self.d.move_point(self.p);
    }
}

impl <'r>Game<'r> {
    fn get_wb<T,U>(graphic: &Graphic<U>,offset: u32, texture_creator : &'r TextureCreator<T>, tile_set : &TileSet) 
        -> (Graphic<Texture<'r>>, Graphic<Texture<'r>>,Graphic<Texture<'r>>) {
        let mut base = Graphic::blank(2,2).textured(texture_creator);
        let mut mid  = Graphic::blank(2,2).textured(texture_creator);
        let mut tip  = Graphic::blank(2,2).textured(texture_creator);
        base.copy_tiles_from(graphic,0,offset,2,2,0,0);
        mid.copy_tiles_from(graphic,2,offset,2,2,0,0);
        tip.copy_tiles_from(graphic,4,offset,2,2,0,0);
        base.update_texture(tile_set);
        mid.update_texture(tile_set);
        tip.update_texture(tile_set);
        (base,mid,tip)
    }
    fn new<T>(texture_creator : &'r TextureCreator<T>) -> Game<'r> {
        let mut graphic = Graphic::blank(WIDTH*2, HEIGHT*2).textured(texture_creator); 
        for x in 0..WIDTH*2 {
            for y in 0..HEIGHT*2 {
                graphic[(x,y)] = Tile { index: match (x%2 == 0, y%2 ==0) {
                    (true,true) => 125,
                    (true,false) => 109,
                    (false,true) => 124,
                    (false,false) => 108
                }, fg: NEUTRAL_GRAY, bg: DARKER_GRAY };
            }
        }
        let mut ball_g1 = Graphic::blank(2, 2).textured(texture_creator); 
        let mut ball_g2 = Graphic::blank(2, 2).textured(texture_creator); 
        ball_g1[(0,0)] = Tile { bg:DARK_RED, fg: TRANSPARENT, index:302};
        ball_g2[(0,0)] = Tile { bg:WHITE, fg: TRANSPARENT, index:302};
        ball_g2[(1,0)] = Tile { bg:DARK_RED, fg: TRANSPARENT, index:303};
        ball_g1[(1,0)] = Tile { bg:WHITE, fg: TRANSPARENT, index:303};
        ball_g2[(0,1)] = Tile { bg:DARK_RED, fg: TRANSPARENT, index:287};
        ball_g1[(0,1)] = Tile { bg:WHITE, fg: TRANSPARENT, index:287};
        ball_g1[(1,1)] = Tile { bg:DARK_RED, fg: TRANSPARENT, index:286};
        ball_g2[(1,1)] = Tile { bg:WHITE, fg: TRANSPARENT, index:286};
        let mut cursor_g1 = Graphic::load_from(Cursor::new(&include_bytes!("../cursor_v")[..])).unwrap().textured(texture_creator);
        let mut cursor_g2 = Graphic::load_from(Cursor::new(&include_bytes!("../cursor_h")[..])).unwrap().textured(texture_creator);
        let wbs = Graphic::load_from(Cursor::new(&include_bytes!("../wbs")[..])).unwrap();
        
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../tiles")[..]));
        graphic.update_texture(&tile_set);
        ball_g1.update_texture(&tile_set);
        ball_g2.update_texture(&tile_set);
        cursor_g1.update_texture(&tile_set);
        cursor_g2.update_texture(&tile_set);
        let wb_n = Self::get_wb(&wbs,0,texture_creator,&tile_set);
        let wb_s = Self::get_wb(&wbs,2,texture_creator,&tile_set);
        let wb_e = Self::get_wb(&wbs,4,texture_creator,&tile_set);
        let wb_w = Self::get_wb(&wbs,6,texture_creator,&tile_set);
        let overlay = Graphic::blank(WIDTH*2, HEIGHT*2); 
        let mut o1 = Graphic::solid(1,1,Self::EDGE_HI_FILL).textured(texture_creator);
        let mut o2 = Graphic::solid(1,1,Self::EDGE_LO_FILL).textured(texture_creator);
        let mut o3 = Graphic::solid(1,1,Self::OUTER_CORNER).textured(texture_creator);
        let mut o4 = Graphic::solid(1,1,Self::INNER_CORNER).textured(texture_creator);
        let mut o5 = Graphic::solid(1,1,Self::UNREVEAL_BG_FILL).textured(texture_creator);
        let mut o6 = Graphic::solid(1,1,Tile{fg:TRANSPARENT,bg:TRANSPARENT,index:0}).textured(texture_creator);
        o1.update_texture(&tile_set);
        o2.update_texture(&tile_set);
        o3.update_texture(&tile_set);
        o4.update_texture(&tile_set);
        o5.update_texture(&tile_set);
        o6.update_texture(&tile_set);
        Game {
            tile_set: tile_set,
            board: graphic,
            ball_g1: ball_g1,
            ball_g2: ball_g2,
            cursor_g1: cursor_g1,
            cursor_g2: cursor_g2,
            subtick: 0,
            cursor_vert: false,
            cursor_x: 0,
            cursor_y: 0,
            balls: Vec::new(),
            red: None,
            blue: None,
            wb_n, wb_s,wb_e,wb_w,
            blocked: [[false;HEIGHT as usize];WIDTH as usize],
            overlay: overlay,
            overlay_tiles: (o1,o2,o3,o4,o5,o6),
            lives: 0,
            percentage: 0,
            speed: 1
        }
    }
    fn set_up_level(&mut self, level : u32) {
        self.balls = Vec::new();
        self.red = None;
        self.blue = None;
        for _ in 0..level+1 {
            let (mut rx,mut ry) = (thread_rng().gen_range(0,WIDTH as i32*4), thread_rng().gen_range(0,HEIGHT as i32*4));
            let dir = match thread_rng().gen_range(0,4) {
                0 => Direction::NE,
                1 => Direction::NW,
                2 => Direction::SE,
                _ => Direction::SW,
            };
            while self.collision(Ball::new(rx,ry,dir),None).len() > 0 {
                rx = thread_rng().gen_range(0,WIDTH as i32*4);
                ry = thread_rng().gen_range(0,HEIGHT as i32*4);
            } 
            let (rx,ry) = dir.move_point((rx,ry));
            self.balls.push(Ball::new(rx,ry, dir));
        }
        self.blocked = [[false;HEIGHT as usize]; WIDTH as usize];
        self.lives = level+1;
        self.percentage = 0;
        self.update_overlay();

    }
    fn start_wallbuilders(&mut self) {
        if self.red.is_none() {
            self.red = Some(WallBuilder{
                p:(self.cursor_x,self.cursor_y),
                size: 0,
                direction: if self.cursor_vert { Orthogonal::N } else { Orthogonal::W }
            })
        }
        if self.blue.is_none() {
            self.blue= Some(WallBuilder{
                p:if self.cursor_vert { (self.cursor_x,self.cursor_y + 1) } else { (self.cursor_x+1,self.cursor_y) },
                size: 0,
                direction: if self.cursor_vert { Orthogonal::S } else { Orthogonal::E }
            })
        }
    }
    fn flood_fill_from(&self, arr : &mut [[bool;HEIGHT as usize]; WIDTH as usize], x : i32, y: i32) {        
        if self.is_blocked(x,y) {
            return
        } else if !arr[x as usize][y as usize] {
            return
        } else {
            arr[x as usize][y as usize] = false;
            self.flood_fill_from(arr,x-1,y);
            self.flood_fill_from(arr,x+1,y);
            self.flood_fill_from(arr,x,y-1);
            self.flood_fill_from(arr,x,y+1);
        }
    }
    fn flood_fill_balls(&mut self) {
        let mut newblocked = [[true;HEIGHT as usize];WIDTH as usize];
        for Ball { p,..} in &self.balls {
            let (x,y) = (p.0/4,p.1/4);
            self.flood_fill_from(&mut newblocked,x,y);
        }
        self.blocked = newblocked;
    }
    fn collides(b1:(i32,i32),b2:(i32,i32,i32,i32)) -> (bool,bool,bool,bool) {
        let top_left = b1;
        let top_right = (b1.0+3,b1.1);
        let bottom_left= (b1.0,b1.1+3);
        let bottom_right= (b1.0+3,b1.1+3);
        let tl = top_left.0 >= b2.0 && top_left.0 < b2.2
         && top_left.1 >= b2.1 && top_left.1 <  b2.3;
        let tr = top_right.0 >= b2.0 && top_right.0 <  b2.2
         && top_right.1 >= b2.1 && top_right.1 < b2.3;
        let br = bottom_right.0 >= b2.0 && bottom_right.0 <  b2.2
         && bottom_right.1 >= b2.1 && bottom_right.1 < b2.3;
        let bl = bottom_left.0 >= b2.0 && bottom_left.0 <  b2.2
         && bottom_left.1 >= b2.1 && bottom_left.1 < b2.3;
            (tl,tr,bl,br)
        }
    fn free_on_board(&self, p:(i32,i32)) -> bool {
        let (x,y) = p;
        if x < 0 || y < 0 || x >= WIDTH as i32*4 || y >= HEIGHT as i32*4 {
            return false 
        } else {
            return !self.blocked[x as usize/4][y as usize/4]
        }
    }
    fn board_collides(&self, b1:(i32,i32)) -> (bool,bool,bool,bool) {
        let top_left = b1;
        let top_right = (b1.0+3,b1.1);
        let bottom_left= (b1.0,b1.1+3);
        let bottom_right= (b1.0+3,b1.1+3);
        let tl = !self.free_on_board(top_left);
        let tr = !self.free_on_board(top_right);
        let bl = !self.free_on_board(bottom_left);
        let br = !self.free_on_board(bottom_right);
        (tl,tr,bl,br)

    }
    fn collision(&self, ball:Ball, except: Option<usize>) -> Vec<Collision> {
        let mut ret = Vec::new();
        let moved = ball.d.move_point(ball.p);
        let (a,b,c,d) = self.board_collides(moved);
        if a || b || c || d {
            ret.push(Collision {
                collision_type: CollisionType::Wall,
                sides:(a,b,c,d)
            })
        }
        for i in 0..self.balls.len() {
            if Some(i) != except {
                let (nx,ny) = self.balls[i].d.move_point(self.balls[i].p);
                let (a,b,c,d) = Self::collides(moved,(nx,ny,nx+4,ny+4));
                if a || b || c || d {
                    ret.push(Collision {
                        collision_type: CollisionType::Ball(i),
                        sides:(a,b,c,d)
                    })
                }
            }
        }
        if let Some(w) = &self.red {
            let (a,b,c,d) = Self::collides(moved, w.occlusion_rect());
            if a || b || c || d {
                    ret.push(Collision {
                        collision_type: CollisionType::WallBuilder(true),
                        sides:(a,b,c,d)
                    })
            }
        }
        if let Some(w) = &self.blue {
            let (a,b,c,d) = Self::collides(moved, w.occlusion_rect());
            if a || b || c || d {
                    ret.push(Collision {
                        collision_type: CollisionType::WallBuilder(false),
                        sides:(a,b,c,d)
                    })
            }
        }
        ret
    }
    fn tick(&mut self) {
        self.subtick = (self.subtick + 1) % 24;
        if match self.speed {
           1 => self.subtick %6 == 1,
           0 => self.subtick %8 == 1,
           _ => self.subtick %4 == 1,
        }  {
            self.grow_wallbuilders();
        }
        if match self.speed {
            1 => self.subtick % 2 == 0 ,
            0 => self.subtick % 3 == 0 ,
            _ => true,
        } {
            for count in 0..self.balls.len() {
                for i in self.collision(self.balls[count], Some(count)) {
                    match i.collision_type {
                        CollisionType::WallBuilder(is_red) => {
                            if is_red {
                                self.red = None
                            } else {
                                self.blue = None
                            }
                            self.lives -= 1;
                        },
                        _ => self.balls[count].collide(i),
                    }
                }
            }
            for count in 0..self.balls.len() {
                self.balls[count].go_forward();
            }
        }
    }
    fn is_blocked(&self, x: i32, y:i32) -> bool {        
        if x < 0 || x >= WIDTH as i32 || y < 0 || y >= HEIGHT as i32 {
            true
        } else {
            self.blocked[x as usize][y as usize]
        }
    }
    const OUTER_CORNER : Tile = Tile { index: 206, fg: EDGE_LO, bg:EDGE_HI};//Tile { index: 303, fg: REVEAL_BG, bg:EDGE_LO};
    const INNER_CORNER : Tile = Tile { index: 206, fg: EDGE_HI, bg:EDGE_LO};
    const EDGE_HI_FILL : Tile = Tile { index: 0, fg: UNREVEAL_BG, bg:EDGE_HI};
    const EDGE_LO_FILL : Tile = Tile { index: 0, fg: UNREVEAL_BG, bg:EDGE_LO};
    const UNREVEAL_BG_FILL : Tile = Tile { index: 0, fg: UNREVEAL_BG, bg:UNREVEAL_BG};
    fn update_overlay(&mut self) {
        for i in 0..WIDTH as i32  {
            for j in 0..HEIGHT as i32 {
                let (x,y) = (i as u32 *2,j as u32*2);
                if self.blocked[i as usize][j as usize] {
                    if self.is_blocked(i-1,j) && self.is_blocked(i,j-1) && self.is_blocked(i-1, j-1) {
                        self.overlay[(x,y)] = Self::UNREVEAL_BG_FILL;
                    } else if self.is_blocked(i-1,j) && self.is_blocked(i,j-1) {
                        self.overlay[(x,y)] = Self::EDGE_HI_FILL;
                    } else if self.is_blocked(i-1,j) || self.is_blocked(i,j-1) {
                        self.overlay[(x,y)] = Self::EDGE_HI_FILL;
                    } else {
                        self.overlay[(x,y)] = Self::EDGE_HI_FILL;
                    }
                    if self.is_blocked(i+1,j) && self.is_blocked(i,j-1) && self.is_blocked(i+1, j-1) {
                        self.overlay[(x+1,y)] = Self::UNREVEAL_BG_FILL;
                    } else if self.is_blocked(i+1,j) && self.is_blocked(i,j-1) {
                        self.overlay[(x+1,y)] = Self::INNER_CORNER;
                    } else if self.is_blocked(i+1,j) {
                        self.overlay[(x+1,y)] = Self::EDGE_HI_FILL;
                    } else if self.is_blocked(i,j-1) {
                        self.overlay[(x+1,y)] = Self::EDGE_LO_FILL;
                    } else {
                        self.overlay[(x+1,y)] = Self::OUTER_CORNER;
                    }
                    if self.is_blocked(i+1,j) && self.is_blocked(i,j+1) && self.is_blocked(i+1, j+1) {
                        self.overlay[(x+1,y+1)] = Self::UNREVEAL_BG_FILL;
                    } else if self.is_blocked(i+1,j) && self.is_blocked(i,j+1) {
                        self.overlay[(x+1,y+1)] = Self::EDGE_LO_FILL;
                    } else if self.is_blocked(i+1,j) {
                        self.overlay[(x+1,y+1)] = Self::EDGE_LO_FILL;
                    } else if self.is_blocked(i,j+1) {
                        self.overlay[(x+1,y+1)] = Self::EDGE_LO_FILL;
                    } else {
                        self.overlay[(x+1,y+1)] = Self::EDGE_LO_FILL;
                    }
                    if self.is_blocked(i-1,j) && self.is_blocked(i,j+1) && self.is_blocked(i-1, j+1) {
                        self.overlay[(x,y+1)] = Self::UNREVEAL_BG_FILL;
                    } else if self.is_blocked(i-1,j) && self.is_blocked(i,j+1) {
                        self.overlay[(x,y+1)] = Self::INNER_CORNER;
                    } else if self.is_blocked(i-1,j) {
                        self.overlay[(x,y+1)] = Self::EDGE_LO_FILL;
                    } else if self.is_blocked(i,j+1) {
                        self.overlay[(x,y+1)] = Self::EDGE_HI_FILL;
                    } else {
                        self.overlay[(x,y+1)] = Self::OUTER_CORNER;
                    }
                } else {
                    self.overlay[(x,y)] = Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT};
                    self.overlay[(x+1,y)] = Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT};
                    self.overlay[(x+1,y+1)] = Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT};
                    self.overlay[(x,y+1)] = Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT};
                }
            }
        }
    }
    fn block_area(&mut self,tx:i32,ty:i32,bx:i32,by:i32) {
        for i in tx..=bx {
            for j in ty..=by {
                if i < WIDTH as i32 && i >= 0 && j < HEIGHT as i32 && j >= 0 {
                    self.blocked[i as usize][j as usize] = true;
                }
            }
        }
        self.flood_fill_balls();
        self.update_percentage();
        self.update_overlay();
    }
    fn update_percentage(&mut self) {
        let mut count = 0;
        let mut blocked = 0;
        for i in 0..WIDTH as usize {
            for j in 0..HEIGHT as usize {
                if self.blocked[i][j] {
                    blocked += 1;
                }
                count +=1;
            }
        }
        self.percentage = (blocked * 100) / count;
    }
    fn grow_wallbuilders(&mut self) {
        for x in [self.red,self.blue].iter_mut() {
            if let Some(w) = x {
                let (tx,ty,bx,by) = w.occlusion_rect();
                if !self.free_on_board((tx,ty)) || !self.free_on_board((bx,by)) {
                    match w.direction {
                        Orthogonal::N | Orthogonal::W => self.red = None,
                        _ => self.blue = None,
                    }
                    self.block_area(tx/4,ty/4,bx/4,by/4);
                }                
            }
        }
        if let Some(w) = &mut self.red {
            w.size += 1;
            if let Some(other) = self.blue {
                let (tx,ty,bx,by) = w.occlusion_rect();
                let (a,b,c,d) = Self::collides((tx,ty), other.occlusion_rect());
                let (a2,b2,c2,d2) = Self::collides((bx-3,by-3), other.occlusion_rect());
                if a || b || c || d || a2 || b2 || c2 || d2 {
                    w.size -= 1;
                }
            }
        }
        if let Some(w) = &mut self.blue {
            w.size += 1;
            if let Some(other) = self.red {
                let (tx,ty,bx,by) = w.occlusion_rect();
                let (a,b,c,d) = Self::collides((tx,ty), other.occlusion_rect());
                let (a2,b2,c2,d2) = Self::collides((bx-3,by-3), other.occlusion_rect());
                if a || b || c || d || a2 || b2 || c2 || d2 {
                    w.size -= 1;
                }
            }
        }

    }
    fn draw_wallbuilder<T:RenderTarget>(&self,canvas :&mut Canvas<T>, wb : WallBuilder) {
        let (bx,by) = (wb.p.0*16, wb.p.1*16+17);
        let (b,m,t) = match wb.direction {
            Orthogonal::N => &self.wb_n,
            Orthogonal::S => &self.wb_s,
            Orthogonal::E => &self.wb_e,
            Orthogonal::W => &self.wb_w,
        };
        b.draw(canvas,(bx,by));
        let (mut x,mut y) = (bx,by);
        for _ in 0..wb.size {
            let (nx,ny) = wb.direction.move_point((x,y),8);
            x = nx;
            y = ny;
            m.draw(canvas,(x,y));
        }
        t.draw(canvas,(x,y));
    }
    fn draw<T:RenderTarget>(&self, canvas : &mut Canvas<T>, cursor: bool) {
        self.board.draw(canvas,(0,17));
        if cursor { 
            if self.cursor_vert {
                self.cursor_g1.draw(canvas,(self.cursor_x * 16, self.cursor_y*16 - 8 + 17));
           } else {
               self.cursor_g2.draw(canvas,(self.cursor_x * 16 - 8,self.cursor_y*16 + 17))
           }
        }
        if let Some(w) = &self.red {
            self.draw_wallbuilder(canvas, *w)
        }
        if let Some(w) = &self.blue {
            self.draw_wallbuilder(canvas, *w);
            canvas.set_draw_color(TEAL);
        }
        for i in &self.balls {
            if self.subtick / 8 == 0 {
                &self.ball_g1
            } else {
                &self.ball_g2
            }.draw(canvas, (i.p.0  * 4,i.p.1 *4 + 17));
        }
        let (eh,el,oc,ic,bg,tr) = &self.overlay_tiles;
        for i in 0..WIDTH as u32 *2 {
            for j in 0..HEIGHT as u32 *2 {
                if self.overlay[(i,j)] == Self::EDGE_HI_FILL {
                    &eh
                } else if self.overlay[(i,j)] == Self::EDGE_LO_FILL {
                    &el
                } else if self.overlay[(i,j)] == Self::OUTER_CORNER {
                    &oc
                } else if self.overlay[(i,j)] == Self::INNER_CORNER {
                    &ic
                } else if self.overlay[(i,j)] == Self::UNREVEAL_BG_FILL {
                    &bg
                } else {
                    &tr
                }.draw(canvas,(i as i32*8, j as i32 *8+17))
            }
        }
    }
}
enum Splash {
    Level,
    Loss,
    Pause
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.text_input().start();
    let window = video_subsystem.window("corbynball", WIDTH*16, HEIGHT*16 + 16 + 17)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    let mut splash = Some(Splash::Level);
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(WIDTH*16, HEIGHT*16 + 16 + 17).unwrap();
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut game = Game::new(&texture_creator);
    let mut level = 1;
    game.set_up_level(level);
    let mut status_level = Graphic::blank(8,1).textured(&texture_creator);
    let mut status_percent = Graphic::blank(3,1).textured(&texture_creator);
    let mut status_lives = Graphic::blank(8,1).textured(&texture_creator);
    let mut lose = Graphic::load_from(Cursor::new(&include_bytes!("../lose")[..])).unwrap().textured(&texture_creator);
    lose.update_texture(&game.tile_set);
    let mut level_splash = Graphic::load_from(Cursor::new(&include_bytes!("../level_splash")[..])).unwrap().textured(&texture_creator);
    level_splash.update_texture(&game.tile_set);
    let mut paused_splash = Graphic::load_from(Cursor::new(&include_bytes!("../paused_splash")[..])).unwrap().textured(&texture_creator);
    paused_splash.update_texture(&game.tile_set);
    let mut rate_limiter = FPSManager::new();
    rate_limiter.set_framerate(60).unwrap();
    let mut micro_mode = false;
    let mut menu = MenuBar::new(WIDTH*16)
                    .add(Menu::new("GAME",72,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("New", Keycode::N,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Pause", Keycode::P,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(56, &texture_creator,&game.tile_set))
                            .add(MenuItem::new("Quit", Keycode::F12,&texture_creator,&game.tile_set)))
                    .add(Menu::new("SPEED",72,&texture_creator,&game.tile_set) 
                            .add(MenuItem::new("Slow", Keycode::F1,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Medium", Keycode::F2,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Fast", Keycode::F3,&texture_creator,&game.tile_set)))
                    .add(Menu::new("VIEW",104,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Micro-mode", Keycode::F9, &texture_creator, &game.tile_set)));
    loop {
        if splash.is_none() { 
            game.tick();
            if game.percentage >= 75 {
                level += 1;
                game.set_up_level(level);
                splash = Some(Splash::Level);
            }
            if game.lives == 0 {
                level = 1;
                game.set_up_level(level);
                splash = Some(Splash::Loss);
            }
        }
        game.draw(&mut canvas, splash.is_none());
        menu.draw(&mut canvas);
        canvas.set_draw_color(BLACK);
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*16 + 17,WIDTH*16,17)).unwrap();
        canvas.set_draw_color(rgba(176,179,172,255));
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*16 + 18,WIDTH*16,16)).unwrap();
        status_level.draw_rect(0,0,8,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_level.draw_text(&("Level ".to_owned() + &level.to_string()),&game.tile_set,0,0,BLACK,TRANSPARENT);
        status_lives.draw_rect(0,0,8,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_lives.draw_text(&(game.lives.to_string() + " Lives"),&game.tile_set,0,0,BLACK,TRANSPARENT);
        status_percent.draw_rect(0,0,3,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_percent.draw_text(&(game.percentage.to_string() + "%"),&game.tile_set,0,0,BLACK,TRANSPARENT);
        status_level.update_texture(&game.tile_set);
        status_lives.update_texture(&game.tile_set);
        status_percent.update_texture(&game.tile_set);
        status_level.draw(&mut canvas, (8, HEIGHT as i32 *16 + 22 ));
        status_lives.draw(&mut canvas, (WIDTH as i32 * 16 - 8 * 9 + if game.lives < 10 { 8 } else { 0 }, HEIGHT as i32 *16 + 22 ));
        status_percent.draw(&mut canvas, (WIDTH as i32 * 8 - 12 + if game.percentage < 10 { 4 } else { 0 }, HEIGHT as i32 *16 + 22 ));
        if let Some(s) = &splash {
            match *s {
                Splash::Level => { 
                    level_splash.draw(&mut canvas, (WIDTH as i32 * 8 - (11*4), HEIGHT as i32 * 8 - 16 + 17 ));
                    status_level.draw(&mut canvas, (WIDTH as i32 * 8 - (8 * 4) + if level < 10 { 4} else {0}, HEIGHT as i32 * 8 - 4 + 17));
                },
                Splash::Loss => lose.draw(&mut canvas, (WIDTH as i32 * 8 - (20*4), HEIGHT as i32 * 8 - (20*4) + 17 )),
                Splash::Pause => paused_splash.draw(&mut canvas, (WIDTH as i32 * 8 - (11*4), HEIGHT as i32 * 8 - 20 + 17 )),
            }
        }
        canvas.present();        
        for event in event_pump.poll_iter() {
            let h = menu.handle_event(event.clone(), &mut event_subsystem);
            match event {
                _ if h => {},
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::F12), .. } => {
                    return
                },
                Event::KeyDown { keycode: Some(Keycode::P), .. } => {
                    splash = Some(Splash::Pause)
                },
                Event::KeyDown { keycode: Some(Keycode::N), .. } => {
                    level = 1;
                    game.set_up_level(level);
                    splash = Some(Splash::Level);
                },
                Event::KeyDown { keycode: Some(Keycode::F1), ..} => {
                    game.speed = 0;
                },
                Event::KeyDown { keycode: Some(Keycode::F2), ..} => {
                    game.speed = 1;
                },
                Event::KeyDown { keycode: Some(Keycode::F3), ..} => {
                    game.speed = 2;
                },
                Event::KeyDown { keycode: Some(Keycode::F9), ..} => {
                    if micro_mode {
                        micro_mode = false;
                        canvas.window_mut().set_size(WIDTH*16, HEIGHT*16+16+17).unwrap_or_default();
                    } else {
                        micro_mode = true;
                        canvas.window_mut().set_size(WIDTH*8, (HEIGHT*16+16+17)/2).unwrap_or_default();
                    }
                },
                Event::KeyDown {..} if splash.is_some() => {
                    splash = None  
                }
                Event::MouseMotion { x, y, ..} if y > 17 => {
                    game.cursor_x = x /16;
                    game.cursor_y = (y - 17) /16
                },
                Event::MouseButtonDown { mouse_btn: btn ,y,.. } if y > 17 && splash.is_none() => {
                    match btn {
                        MouseButton::Right=> 
                            game.cursor_vert = !game.cursor_vert,
                        MouseButton::Left=>
                            game.start_wallbuilders(),
                        _ => {},
                    }
                }
                Event::MouseButtonUp {..} if splash.is_some() => {
                    splash = None
                }
                _ => {},
            }
        }
        rate_limiter.delay();
    }
}
