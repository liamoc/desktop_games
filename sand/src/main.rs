
extern crate tesserae;
extern crate sdl2;
extern crate rand;
extern crate utils;
extern crate bresenham;
use rand::seq::SliceRandom;
use sdl2::mouse::MouseButton;
use utils::OutlinedTile;
use utils::menu::{*};

use tesserae::{Graphic,Tile,TileSet};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::render::{Texture, BlendMode, TextureCreator};
use sdl2::video::Window;
use sdl2::Sdl; 
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use utils::framerate::FPSManager;
use sdl2::rect::Rect;
use std::io::Cursor;
use bresenham::Bresenham;

use rand::{thread_rng,Rng};
use utils::color::{*};
const WIDTH:u32=328;
const HEIGHT:u32=328+18;
const BOARD_WIDTH:usize=328;
const BOARD_HEIGHT:usize=328;
fn elem_name(e : u32) -> &'static str {
    match e {
        E_WALL => "Wall",
        E_WATER => "Water",
        E_STEAM => "Steam",
        E_STEAM_CONDENSED => "Steam", 
        E_OIL => "Oil",
        E_SAND => "Sand",
        E_SPACE => "Empty",
        E_SALT => "Salt",
        E_STONE => "Stone",
        E_SALTWATER => "Saltwater",
        E_LAVA => "Lava",
        E_METAL => "Metal",
        E_SPOUT => "Spout",
        E_TORCH => "Torch",
        E_PLANT => "Plant",
        E_WOOD => "Wood",
        E_CLONE => "Clone",
        E_ICE => "Ice",
        E_ACID => "Acid",
        e if is_clone(e) => "Clone",
        e if is_fire(e) => "Fire",
        _ => "???"
    }
}
const E_WALL:u32=200;
const E_WATER:u32=20;
const E_STEAM:u32=22;
const E_STEAM_CONDENSED:u32=23;
const E_OIL:u32=25;
const E_ACID:u32=24;
const E_SAND:u32=40;
const E_SPACE:u32=0;
const E_SALT:u32=30;
const E_ASH:u32 = 32;
const E_STONE:u32=31;
const E_SALTWATER:u32=21;
const E_LAVA:u32=10;
const E_WOOD:u32 = 206;
const E_FIRE_START:u32=50;
const E_FIRE_END:u32=59;
const E_METAL:u32= 202;
const E_SPOUT:u32=204;
const E_TORCH:u32 = 201;
const E_PLANT:u32 = 203;
const E_ICE:u32 = 205;
const E_EMBER:u32 = 300;
const E_EMBER_END:u32 = 308;
const E_CLONE:u32 = 0x0001_0000;

fn random_element() -> u32 {
    let elements : Vec<u32> = vec![E_WALL,E_WATER,E_STEAM,E_STEAM_CONDENSED,E_SAND,E_SALT,E_SALTWATER, E_FIRE_START, E_FIRE_START+1,E_FIRE_START+2,E_FIRE_START+3,E_FIRE_START+4,E_FIRE_START+5,E_FIRE_START+6,E_FIRE_START+7,E_FIRE_START+8,E_FIRE_END,E_TORCH,E_SPACE];
    *elements.choose(&mut rand::thread_rng()).unwrap_or(&E_SPACE)
}
fn is_wall(element : u32) -> bool {
    element >= E_WALL
}
fn is_ember(element : u32) -> bool {
    element >= E_EMBER && element <= E_EMBER_END
}
fn is_clone(element : u32) -> bool {
    element & E_CLONE != 0
}
fn is_gas(element : u32) -> bool {
    element == E_SPACE || element == E_STEAM || is_fire(element) || element == E_STEAM_CONDENSED
}
fn is_fluid(element : u32) -> bool {
    element <= 25 && element > 0
}
fn is_fire(element : u32) -> bool {
    element >= E_FIRE_START && element <= E_FIRE_END
}
fn is_stone(element :u32) -> bool {
    element == E_STONE
}
fn is_permeable(e1 : u32, e2 : u32) -> bool {
    !is_wall(e1) && !is_wall(e2) && (is_fluid(e1) || is_fluid(e2) || is_fire(e1) || is_fire(e2) || weight(e1) <= weight(E_SPACE) || weight(e2) <= weight(E_SPACE))
}
fn weight(element : u32) -> u32 {
    match element {
        e if is_wall(e) => 100000,
        E_WATER => 60000,
        E_SALTWATER => 65000,
        E_LAVA => 70000,
        E_OIL => 55000,
        E_ACID => 66000,
        E_ASH => 70000,
        E_SAND => 80000,
        E_SALT => 85000,
        E_SPACE => 10000,
        E_STONE => 90000,
        E_STEAM => 5000,
        E_STEAM_CONDENSED => 5000,

        e if is_fire(e) => 0,
        _ => element
    }
}
fn color_for(element : u32) -> Color {
    match element {
        E_WALL => NEUTRAL_GRAY,
        E_METAL => rgba(102,102,102,255),
        E_WATER => rgba(52,101,164,255),
        E_SAND => YELLOW,
        E_SALT => WHITE,
        E_LAVA => DARK_RED,
        E_ACID => BRIGHT_GREEN,
        E_STONE => DARKER_GRAY,
        E_SALTWATER => PALE_BLUE, // 
        E_STEAM => rgba(174,199,227,255),
        E_STEAM_CONDENSED => rgba(174,199,227,255),
        E_TORCH => PALE_ORANGE,
        E_OIL => DARK_YELLOW,
        E_SPOUT => BLUE,
        E_ICE => rgba(204,229,255,255),
        E_PLANT => GREEN,
        E_SPACE => CHARCOAL,
        E_WOOD => BROWN,
        E_ASH => DARK_CHARCOAL,
        e if is_clone(e) => LIGHT_BROWN,
        e if is_fire(e) => {
            rgba(255,(((e - E_FIRE_START) * 255) / (E_FIRE_END - E_FIRE_START)) as u8, 10,255)
        }
        e if is_ember(e) => {
            rgba(255,(((e - E_EMBER) * 255) / 8) as u8, 10,255)
        }
        _ => GREEN
    }
}

fn alchemy(e1 : u32, e2 : u32) -> (u32,u32) {
    match (e1, e2) {
        
        (E_SALT,E_WATER) => (E_SALTWATER, E_SALTWATER),
        (E_WATER,E_SALT) => (E_SALTWATER, E_SALTWATER),
        (E_TORCH,E_SPACE) => (E_TORCH,E_FIRE_START),
        (E_SPACE,E_TORCH) => (E_FIRE_START,E_TORCH),
        (E_SPOUT,E_SPACE) => (E_SPOUT,E_WATER),
        (E_SPACE,E_SPOUT) => (E_WATER,E_SPOUT),
        (E_WATER, E_METAL) if rand::thread_rng().gen_range(0,100) < 2 => (E_WATER, E_SAND),
        (E_SALTWATER, E_METAL) if rand::thread_rng().gen_range(0,100) < 5 => (E_SALTWATER, E_SAND),
        (E_METAL, E_WATER) if rand::thread_rng().gen_range(0,100) < 2  => (E_SAND, E_WATER),
        (E_METAL, E_SALTWATER) if rand::thread_rng().gen_range(0,100) < 5  => (E_SAND, E_SALTWATER),
        (E_LAVA, E_STONE) if rand::thread_rng().gen_range(0,100) < 1  => (E_LAVA,E_LAVA),
        (E_STONE, E_LAVA) if rand::thread_rng().gen_range(0,100) < 1  => (E_LAVA,E_LAVA),
        (E_METAL, E_LAVA) if rand::thread_rng().gen_range(0,100) < 1  => (E_LAVA,E_LAVA),
        (E_LAVA, E_METAL) if rand::thread_rng().gen_range(0,100) < 1  => (E_LAVA,E_LAVA),
        (E_SALT, E_LAVA) if rand::thread_rng().gen_range(0,100) < 3  => (E_LAVA,E_LAVA),
        (E_LAVA, E_SALT) if rand::thread_rng().gen_range(0,100) < 3  => (E_LAVA,E_LAVA),
        (E_SAND, E_LAVA) if rand::thread_rng().gen_range(0,100) < 5  => (E_LAVA,E_LAVA),
        (E_LAVA, E_SAND) if rand::thread_rng().gen_range(0,100) < 5  => (E_LAVA,E_LAVA),
        (E_LAVA, E_OIL) if rand::thread_rng().gen_range(0,100) < 80  => (E_LAVA,E_FIRE_START),
        (E_OIL, E_LAVA) if rand::thread_rng().gen_range(0,100) < 80  => (E_FIRE_START,E_LAVA),
        (E_LAVA, E_PLANT) if rand::thread_rng().gen_range(0,100) < 80  => (E_LAVA,E_FIRE_START),
        (E_PLANT, E_LAVA) if rand::thread_rng().gen_range(0,100) < 80  => (E_FIRE_START,E_LAVA),
        (E_WATER, E_ICE) if rand::thread_rng().gen_range(0,100) < 2  => (E_ICE, E_ICE),
        (E_ICE, E_WATER) if rand::thread_rng().gen_range(0,100) < 2  => (E_ICE, E_ICE),
        (E_WATER,E_LAVA) => (E_STEAM,E_STONE),
        (E_LAVA,E_WATER) => (E_STONE,E_STEAM),
        (E_ICE,E_LAVA) => (E_WATER,E_LAVA),
        (E_LAVA,E_ICE) => (E_LAVA,E_WATER),
        (E_ASH,E_LAVA) => (E_FIRE_START,E_LAVA),
        (E_LAVA,E_ASH) => (E_LAVA,E_FIRE_START),
        (E_PLANT,E_WATER) => (E_PLANT,E_PLANT),
        (E_WATER,E_PLANT) => (E_PLANT,E_PLANT),
        (E_WOOD, f) if is_fire(f) => (E_EMBER_END,E_SPACE),
        (f, E_WOOD) if is_fire(f) => (E_SPACE,E_EMBER_END),
        (e,E_ICE) if is_fire(e) => (E_SPACE,E_WATER),
        (E_ICE,e) if is_fire(e) => (E_WATER,E_SPACE),
        (E_SALTWATER,E_LAVA) => if rand::thread_rng().gen_range(0,100) < 50 { (E_STEAM,E_STONE) } else {(E_STEAM,E_SALT) },
        (E_LAVA,E_SALTWATER) => if rand::thread_rng().gen_range(0,100) < 50 { (E_STONE,E_STEAM) } else {(E_SALT,E_STEAM) },
        (E_LAVA,E_WOOD) => (E_LAVA,E_EMBER_END),
        (E_WOOD,E_LAVA) => (E_EMBER_END,E_LAVA),
        (E_PLANT,a) if is_fire(a) && rand::thread_rng().gen_range(0,100) < 30  => {
            let r = rand::thread_rng().gen_range(0,100);
             if r < 20 { (E_FIRE_START,E_FIRE_START) } 
             else if r > 80 { (E_FIRE_START,E_ASH) }
             else { (E_FIRE_START,E_SPACE) }                
        }
        (a, E_PLANT) if is_fire(a) && rand::thread_rng().gen_range(0,100) < 30  => {
            let r = rand::thread_rng().gen_range(0,100);
             if r < 20 { (E_FIRE_START,E_FIRE_START) } 
             else if r > 80 { (E_ASH,E_FIRE_START) }
             else { (E_SPACE,E_FIRE_START) }
        }
        (a, E_OIL) if is_fire(a) => (E_FIRE_START, E_FIRE_START),
        (E_OIL, a) if is_fire(a) => (E_FIRE_START, E_FIRE_START),
        (a, E_WATER) if is_fire(a) => (E_SPACE, E_STEAM),
        (E_WATER, a) if is_fire(a) => (E_STEAM, E_SPACE),
        (E_SALTWATER, a) if is_fire(a) => (E_STEAM, E_SALT),
        (a, E_SALTWATER) if is_fire(a) => (E_SALT, E_STEAM),
        (a, E_STEAM) if is_wall(a) || a == E_STEAM_CONDENSED => (a, E_STEAM_CONDENSED),
        (E_STEAM,a) if is_wall(a) || a == E_STEAM_CONDENSED => (E_STEAM_CONDENSED,a),
        (E_CLONE,a) if a != E_CLONE && a != E_SPACE && a != E_WALL => (E_CLONE | a, a),
        (a,E_CLONE) if a != E_CLONE && a != E_SPACE && a != E_WALL => (a, E_CLONE | a),
        (c, E_SPACE) if is_clone(c) => (c, c & !E_CLONE),
        (E_SPACE, c) if is_clone(c) => (c & !E_CLONE, c),
        (E_EMBER,E_WOOD) => (E_EMBER,E_EMBER_END),
        (E_WOOD,E_EMBER) => (E_EMBER_END,E_EMBER),
        (E_EMBER,E_SPACE) => (E_FIRE_START, if rand::thread_rng().gen_range(0,100) < 90 { E_SPACE } else { E_ASH }),
        (e,E_SPACE)  if is_ember(e) => (e - 1, e-1),
        (E_SPACE,E_EMBER) => (if rand::thread_rng().gen_range(0,100) < 90 { E_SPACE } else { E_ASH },E_FIRE_START),
        (E_SPACE,e)  if is_ember(e) => (e - 1, e-1),
        (e,E_WATER) if is_ember(e) => (E_ASH,E_STEAM),
        (E_WATER,e) if is_ember(e) => (E_STEAM,E_ASH),
        (e,E_SALTWATER) if is_ember(e) => (if rand::thread_rng().gen_range(0,100) >= 90 { E_SALT } else { E_ASH },E_STEAM),
        (E_SALTWATER,e) if is_ember(e) => (E_STEAM,if rand::thread_rng().gen_range(0,100) >= 90 { E_SALT } else { E_ASH }),
        (E_ACID,e) if (!is_clone(e) && !is_fire(e) && e != E_WALL && e != E_SPACE && e != E_ACID) => (if rand::thread_rng().gen_range(0,100) < 90 { E_SPACE } else { E_ACID}, E_SPACE),
        (e,E_ACID) if (!is_clone(e) && !is_fire(e) &&  e != E_WALL && e != E_SPACE && e != E_ACID) => (E_SPACE, if rand::thread_rng().gen_range(0,100) < 90 { E_SPACE } else { E_ACID}),
        (a, b) => (a,b)
    }
} 
fn age(e1 : u32) -> u32 {
    if is_fire(e1) {
        if rand::thread_rng().gen_range(0,100) > 50 {
            if e1 == E_FIRE_END {
                E_SPACE
            } else {
                e1 + 1
            }
        } else { e1 }
    } else if is_ember(e1) {
        if rand::thread_rng().gen_range(0,100) > 50 {
            if e1 == E_EMBER {
                if rand::thread_rng().gen_range(0,100) < 90 { E_SPACE } else { E_ASH }
            } else {
                e1 - 1
            }
        } else { e1 }
    } else {
        if e1 == E_STEAM_CONDENSED {
            if rand::thread_rng().gen_range(0,1000) < 5 {
                E_WATER
            } else { e1 }
        } else if e1 == E_STEAM { 
            if rand::thread_rng().gen_range(0,5000) < 1 {
                E_WATER
            } else { e1 }
        } else { e1 }
    }
}
struct World {
    world: [[u32;BOARD_HEIGHT];BOARD_WIDTH],
    odd_tick: bool,
    gravity_table: [[u8;4];256],
    mouse_down: bool, 
    mouse_coords: (isize,isize),
    old_mouse_coords: (isize,isize),
    brush_size: usize,
    current_elem: u32,
    replacing:bool, right_clicking:bool
}
fn combine_bits(elems : [u8;4]) -> u8 {
    elems[0] | elems[1] << 1 | elems[2] << 2 | elems[3] << 3
}
impl World {
    fn gravity_func(wenv : u8) -> [u8;4] {
        match wenv {
            // L L --> L ~
            // L ~     L L
            0b1000_0111 => [0,3,2,1],
            // L L --> ~ L
            // ~ L     L L
            0b1000_1011 => [2,1,0,3],
            // ~ L --> ~ ~
            // ~ L     L L
          //  0b1000_0011 => [0,2,1,3],
            // L ~ --> ~ ~
            // L ~     L L
          //  0b1000_1100 => [3,1,2,0],
            // L ~ --> ~ L
            // * *     * *
            0b1000_1101 => [1,0,2,3],
            // ~ L --> L ~
            // * *     * *
            0b1000_1110 => [1,0,2,3],
            // ~ ~ --> ~ ~
            // L ~     ~ L
            0b1000_0100 => [0,1,3,2],
            // ~ ~ --> ~ ~
            // ~ L     L ~
            0b1000_1000 => [0,1,3,2],
            // stone reactions
            //* ~ --> ~ ~
            //* ~     * *
            c if c & 0b0100_1111 == 0b0101 => [3,1,2,0],
            //~ * --> ~ ~
            //~ *     * *
            c if c & 0b0100_1111 == 0b1010 => [0,2,1,3],
            c => match c & 0b0000_1111 {
                //* ~ --> ~ ~
                //~ ~     * ~
                0b0001 => [2,1,0,3],
                //* * --> * ~
                //* ~     * *
                0b0111 => [0,3,2,1],
                //* * --> ~ ~
                //~ ~     * *
                0b0011 => [2,3,0,1],
                //~ * --> ~ ~
                //* ~     * *
                0b0110 => [0,3,2,1],
                //~ * --> ~ ~
                //~ ~     ~ *
                0b0010 => [0,3,2,1],
                //* * --> ~ *
                //~ *     * *
                0b1011 => [2,1,0,3],
                //* ~ --> ~ ~
                //~ *     * *
                0b1001 => [2,1,0,3],
                _ => [0,1,2,3],
            }
        }
    }
    fn gen_grav_perm(&self, neigh:&[u32;4], heaviest: u32) -> [u8;4] {
        let w1 : [u8;4] = neigh.map(|x| (weight(x) >= heaviest) as u8);
        let w1c = combine_bits(w1) as usize 
                       | neigh.map(|x| match x {
                            _ if weight(x) != heaviest => 0,
                            _ if is_fluid(x) => 0b1000_0000,
                            _ if is_stone(x) => 0b0100_0000,
                            _ => 0
                       }).iter().fold(0,|x,y| x | *y);
        let mut p1 = self.gravity_table[w1c];
        for i in 0..4 {
            if !is_permeable(neigh[p1[i] as usize], neigh[i])  {
                p1[i] = i as u8;
            } else {
                let e1 = neigh[p1[i] as usize];
                let e2 = neigh[i];

                let diff : i32 = weight(e1) as i32 - weight(e2) as i32;
                let n : u32 = if is_gas(e1) && is_gas(e2) {
                    rand::thread_rng().gen_range(0,weight(E_SPACE))
                } else if is_gas(e1) && is_fluid(e2) || is_gas(e2) && is_fluid(e1) {
                    rand::thread_rng().gen_range(0,weight(E_LAVA) + 1000 - weight(E_SPACE))
                } else if is_gas(e1) || is_gas(e2) {
                    rand::thread_rng().gen_range(0,weight(E_STONE) - weight(E_SPACE))
                } else if is_fluid(e1) && is_fluid(e2) {
                    rand::thread_rng().gen_range(0,weight(E_LAVA) - weight(E_OIL))
                } else {
                    rand::thread_rng().gen_range(0,weight(E_STONE) - weight(E_OIL))
                };
                 
                if n >= diff.abs() as u32 {
                    p1[p1[i] as usize] = p1[i];
                    p1[i] = i as u8;
                }
            }
        }
        p1
    }
    fn weigh(&self, neigh: &[u32;4]) -> [u8;4] {
        let weights : [u32;4] = neigh.map(weight);
        let heaviest : u32 = *weights.iter().max().unwrap_or(&0);
        if let Some(&next_heaviest) = weights.iter().filter(|&x| *x != heaviest).max() {
            let p1 = self.gen_grav_perm(neigh,heaviest);
            let neigh2 = [neigh[p1[0] as usize], neigh[p1[1] as usize], neigh[p1[2] as usize], neigh[p1[3] as usize]];
            let p2 = self.gen_grav_perm(&neigh2, next_heaviest);
            [p2[p1[0] as usize], p2[p1[1] as usize],p2[p1[2] as usize],p2[p1[3] as usize]] 
        } else {
            return [0,1,2,3];
        }
    }
    fn new() -> World {        
        let mut gravity_table = [[0;4];256];
        for i in 0..256 {
            gravity_table[i] = World::gravity_func(i as u8);
        }
        let w = World {
            world: [[0;BOARD_HEIGHT];BOARD_WIDTH],
            gravity_table,
            odd_tick: false,
            mouse_down: false,
            right_clicking: false,
            mouse_coords: (0,0),
            old_mouse_coords: (0,0),
            brush_size: 10,
            replacing: false,
            current_elem: 0
        };
        //w.test();
        w
    }
    fn test(&self) {
        for i in 0..1000 {
            let neigh = [random_element(), random_element(),random_element(),random_element()];
            let mut weigh = self.weigh(&neigh);
            weigh.sort();
            if !(weigh[0] == 0 && weigh[1] == 1 && weigh[2] == 2 && weigh[3] == 3) {
                println!("{:?} {:?}",neigh,weigh);
            }
        }
    }
    fn  draw(&self, c : &mut Canvas<Window>, t: & TileSet) {
        c.set_draw_color(CHARCOAL);
        let _ = c.fill_rect(Rect::new(0,0,WIDTH,HEIGHT));
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                c.set_draw_color(color_for(self.world[x][y]));
                let _ = c.draw_point((x as i32, y as i32+19));
            }
        }
        c.set_draw_color(if self.right_clicking { DARK_RED } else if self.replacing { TEAL } else { DARK_TEAL } );
        let r = self.brush_size as i32 / 2;
        let _ = c.draw_rect(Rect::new(self.mouse_coords.0 as i32 - r, 19 + self.mouse_coords.1 as i32 - r,self.brush_size as u32,self.brush_size as u32));
    }
    
    fn mouse_over(&mut self, x :i32, y : i32) {
        self.mouse_coords = (x as isize,y as isize);
    }
    fn mouse_down(&mut self, x :i32, y : i32, left: bool) {
        self.mouse_coords = (x as isize,y as isize);
        if !left {
            self.right_clicking = true;
        }
        self.mouse_down = true;
    }
    fn mouse_up(&mut self, x :i32, y : i32) {
        self.mouse_coords = (x as isize,y as isize);
        self.mouse_down = false;
        self.right_clicking = false;
    }
    fn plot_cursor(&mut self, mx : isize, my: isize) {
        let r = (self.brush_size as isize/2);
        let x  = std::cmp::min(BOARD_WIDTH as isize,std::cmp::max(mx-r,0));
        let y  = std::cmp::min(BOARD_HEIGHT as isize,std::cmp::max(my-r,0));
        let x2 = std::cmp::min(BOARD_WIDTH as isize,std::cmp::max(mx+r,0));
        let y2 = std::cmp::min(BOARD_HEIGHT as isize,std::cmp::max(my+r,0));
        for ix in x as usize..x2 as usize {
            for iy in y as usize..y2 as usize {
                if self.replacing  || self.right_clicking || self.current_elem == E_SPACE|| self.world[ix][iy] == E_SPACE {
                    self.world[ix][iy] = if self.right_clicking { E_SPACE } else { self.current_elem };
                }
            }
        }
    } 
    fn tick(&mut self,paused : bool) {
        self.odd_tick = !self.odd_tick;        
        if self.mouse_down {
            if self.mouse_coords != self.old_mouse_coords {
                for (mx,my) in Bresenham::new(self.mouse_coords,self.old_mouse_coords) {
                    self.plot_cursor(mx, my);
                }
            } else {
                self.plot_cursor(self.mouse_coords.0, self.mouse_coords.1);
            }
        }
        self.old_mouse_coords = self.mouse_coords;
        if paused { return; }
        let mut y : i32 = if self.odd_tick { -1 } else { 0 };
        let mut neigh : [u32;4] = [0;4];
        while y < BOARD_HEIGHT as i32 {
            let mut x = if self.odd_tick { -1 } else { 0 };
            while x < BOARD_WIDTH as i32 {
                neigh[0] = if x >= 0 && y >= 0 { self.world[x as usize][y as usize] } else { 0 };
                neigh[1] = if y >= 0 && ((x + 1) as usize) < BOARD_WIDTH { self.world[(x + 1) as usize][y as usize] } else { 0 };
                neigh[2] = if x >= 0 && ((y + 1) as usize) < BOARD_HEIGHT { self.world[x as usize][(y + 1) as usize] } else { 0 };
                neigh[3] = if ((x + 1) as usize) < BOARD_WIDTH && ((y + 1) as usize) < BOARD_HEIGHT { self.world[(x + 1) as usize][(y + 1) as usize] } else { 0 };
                let (ul,ur) = alchemy(neigh[0],neigh[1]);
                neigh[0] = ul;
                neigh[1] = ur;
                let (ur,dr) = alchemy(neigh[1],neigh[3]);
                neigh[1] = ur;
                neigh[3] = dr;
                let (dr,dl) = alchemy(neigh[3],neigh[2]);
                neigh[3] = dr;
                neigh[2] = dl;
                let (dl,ul) = alchemy(neigh[2],neigh[0]);
                neigh[2] = dl;
                neigh[0] = ul;
                neigh[0] = age(neigh[0]);
                neigh[1] = age(neigh[1]);
                neigh[2] = age(neigh[2]);
                neigh[3] = age(neigh[3]);
                let weigh = self.weigh(&neigh);
                if x >= 0 && y >= 0 { self.world[x as usize][y as usize] = neigh[weigh[0] as usize] };
                if y >= 0 && ((x + 1) as usize) < BOARD_WIDTH { self.world[(x + 1) as usize][y as usize] = neigh[weigh[1] as usize]};
                if x >= 0 && ((y + 1) as usize) < BOARD_HEIGHT { self.world[x as usize][(y + 1) as usize] = neigh[weigh[2] as usize]};
                if ((x + 1) as usize) < BOARD_WIDTH && ((y + 1) as usize) < BOARD_HEIGHT { self.world[(x + 1) as usize][(y + 1) as usize] = neigh[weigh[3] as usize]};
                x = x + 2
                
            }
            y = y + 2
        }
    }

}

fn main_loop(mut window:Window, sdl_context: &Sdl) {
    window.set_size(WIDTH,HEIGHT).unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    
    canvas.set_logical_size(WIDTH,HEIGHT).unwrap();
    canvas.set_blend_mode(BlendMode::Blend);
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
    let mut table = World::new();
    let mut paused: bool = false;
    let mut elem_gfx= Graphic::blank(6,1).textured(&texture_creator);
    let mut elem_gfx_shadow = Graphic::blank(6,1).textured(&texture_creator);
    let mut menu = MenuBar::new(WIDTH)
                    .add(Menu::new("TOOLS",132-(6*8)+16,&texture_creator,&tile_set)
                            .add(MenuItem::submenu("Powders", &texture_creator, &tile_set, 
                                 Menu::new(" ", 132-(7*8), &texture_creator,&tile_set)
                                .add(MenuItem::new("Sand", 352,Keycode::F1,&texture_creator,&tile_set))
                                .add(MenuItem::new("Salt", 353,Keycode::F2,&texture_creator,&tile_set))
                                .add(MenuItem::new("Stone", 354,Keycode::F3,&texture_creator,&tile_set))
                            ))
                            .add(MenuItem::submenu("Liquids", &texture_creator, &tile_set, 
                                Menu::new(" ", 132-(7*8), &texture_creator,&tile_set)
                                .add(MenuItem::new("Oil", 355,Keycode::F4,&texture_creator,&tile_set))
                                .add(MenuItem::new("Lava", 356,Keycode::F5,&texture_creator,&tile_set))
                                .add(MenuItem::new("Acid", 357,Keycode::F6,&texture_creator,&tile_set))
                                .add(MenuItem::new("Water", 358,Keycode::F7,&texture_creator,&tile_set))))
                            .add(MenuItem::submenu("Solids", &texture_creator, &tile_set, 
                                Menu::new(" ", 132-(7*8), &texture_creator,&tile_set)
                                .add(MenuItem::new("Metal", 359,Keycode::F8,&texture_creator,&tile_set))
                                .add(MenuItem::new("Ice", 360,Keycode::F9,&texture_creator,&tile_set))
                                .add(MenuItem::new("Plant", 361,Keycode::F10,&texture_creator,&tile_set))
                                .add(MenuItem::new("Wood", 362,Keycode::F11,&texture_creator,&tile_set))
                                .add(MenuItem::new("Torch", 52,Keycode::Num1,&texture_creator,&tile_set))
                                .add(MenuItem::new("Spout", 53,Keycode::Num2,&texture_creator,&tile_set))
                                .add(MenuItem::new("Clone", 54,Keycode::Num3,&texture_creator,&tile_set))
                                .add(MenuItem::new("Wall", 55,Keycode::Num4,&texture_creator,&tile_set))))
                            .add(MenuItem::new("Fire", 363,Keycode::F12,&texture_creator,&tile_set))
                            .add(MenuItem::new("Erase", 51,Keycode::Num0,&texture_creator,&tile_set))
                            .add(MenuItem::separator(132-(5*8)+2, &texture_creator, &tile_set))                            
                            .add(MenuItem::new("Pause", 17, Keycode::P,&texture_creator,&tile_set))
                            .add(MenuItem::new("Restart",15, Keycode::N,&texture_creator,&tile_set))
                            .add(MenuItem::separator(132-(5*8)+2, &texture_creator, &tile_set))                            
                            .add(MenuItem::new("Quit",18, Keycode::Q,&texture_creator,&tile_set)))
                    .add(Menu::new("BRUSH",92+16+16+16+8,&texture_creator,&tile_set)
                            .add(MenuItem::new("Increase Size", 41, Keycode::RightBracket,&texture_creator,&tile_set))
                            .add(MenuItem::new("Decrease Size", 42, Keycode::LeftBracket,&texture_creator,&tile_set))
                            .add(MenuItem::separator(92+16+16+16, &texture_creator, &tile_set))                            
                            .add(MenuItem::new("Toggle Replace", 19, Keycode::R,&texture_creator,&tile_set))
                        )
                    .add(Menu::new("VIEW",72+32 + 8,&texture_creator,&tile_set)
                            .add(MenuItem::new("Micro-mode",14, Keycode::M,&texture_creator, &tile_set))
                            );
    let mut rate_limiter = FPSManager::new();
    rate_limiter.set_framerate(60).unwrap();
    let mut micro_mode = false;
    let paused_splash = OutlinedTile::new(389, WHITE, &tile_set, &texture_creator);
    let mut old_elem = table.current_elem + 1;
    loop {
        table.tick(paused);
        table.draw(&mut canvas, &tile_set);
        if paused {
            paused_splash.draw(&mut canvas,(WIDTH as i32 - 16, HEIGHT as i32 - 16))
        }
        if old_elem != table.current_elem {
            elem_gfx.draw_rect(0,0,6,1,Tile {fg : TRANSPARENT, bg :TRANSPARENT, index: 0});
            elem_gfx_shadow.draw_rect(0,0,6,1,Tile {fg : TRANSPARENT, bg :TRANSPARENT, index: 0});
            elem_gfx.draw_text(elem_name(table.current_elem), &tile_set,0,0,color_for(table.current_elem),TRANSPARENT);
            let c = color_for(table.current_elem);
            let m = (c.r as u32 + c.g as u32 + c.b as u32) > (81 * 3);
            elem_gfx_shadow.draw_text(elem_name(table.current_elem), &tile_set,0,0,if m { 
                rgba(std::cmp::max(c.r as i32 - 100,0) as u8, std::cmp::max(c.g as i32 - 100,0) as u8, std::cmp::max(c.b as i32 - 100,0) as u8, 255 )
            } else { 
                rgba(std::cmp::min(c.r as u32 + 100,255) as u8, std::cmp::min(c.g as u32 + 100,255) as u8, std::cmp::min(c.b as u32 + 100,255) as u8, 255 )
             },TRANSPARENT);
            elem_gfx.update_texture(&tile_set);
            elem_gfx_shadow.update_texture(&tile_set);
            old_elem = table.current_elem;
        }
        menu.draw(&mut canvas);
        elem_gfx_shadow.draw(&mut canvas, (3 as i32,HEIGHT as i32-12));
        elem_gfx_shadow.draw(&mut canvas, (4 as i32,HEIGHT as i32-11));
        elem_gfx_shadow.draw(&mut canvas, (4 as i32,HEIGHT as i32-13));
        elem_gfx_shadow.draw(&mut canvas, (5 as i32,HEIGHT as i32-12));
        elem_gfx.draw(&mut canvas, (4 as i32,HEIGHT as i32-12));
        canvas.present();
        
        for event in event_pump.poll_iter() {
            let h = menu.handle_event(event.clone(), &mut event_subsystem);
            match event {
                _ if h => { sdl_context.mouse().show_cursor(true)}
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    return
                },
                Event::KeyDown { keycode: Some(Keycode::M), ..} => {
                    if micro_mode {
                        micro_mode = false;
                        canvas.window_mut().set_size(WIDTH,HEIGHT).unwrap_or_default();
                    } else {
                        canvas.window_mut().set_size(WIDTH/2,HEIGHT/2).unwrap_or_default();
                        micro_mode = true;
                    }
                }
                Event::KeyDown { keycode: Some(Keycode::F1), .. } => { table.current_elem = E_SAND }
                Event::KeyDown { keycode: Some(Keycode::F2), .. } => { table.current_elem = E_SALT }
                Event::KeyDown { keycode: Some(Keycode::F3), .. } => { table.current_elem = E_STONE }
                Event::KeyDown { keycode: Some(Keycode::F4), .. } => { table.current_elem = E_OIL }  
                Event::KeyDown { keycode: Some(Keycode::F5), .. } => { table.current_elem = E_LAVA }  
                Event::KeyDown { keycode: Some(Keycode::F6), .. } => { table.current_elem = E_ACID }  
                Event::KeyDown { keycode: Some(Keycode::F7), .. } => { table.current_elem = E_WATER }  
                Event::KeyDown { keycode: Some(Keycode::F8), .. } => { table.current_elem = E_METAL }  
                Event::KeyDown { keycode: Some(Keycode::F9), .. } => { table.current_elem = E_ICE }  
                Event::KeyDown { keycode: Some(Keycode::F10), .. } => { table.current_elem = E_PLANT }  
                Event::KeyDown { keycode: Some(Keycode::F11), .. } => { table.current_elem = E_WOOD }  
                Event::KeyDown { keycode: Some(Keycode::Num1), .. } => { table.current_elem = E_TORCH }  
                Event::KeyDown { keycode: Some(Keycode::Num2), .. } => { table.current_elem = E_SPOUT }  
                Event::KeyDown { keycode: Some(Keycode::Num3), .. } => { table.current_elem = E_CLONE }  
                Event::KeyDown { keycode: Some(Keycode::Num4), .. } => { table.current_elem = E_WALL }  
                Event::KeyDown { keycode: Some(Keycode::F12), .. } => { table.current_elem = E_FIRE_START }  
                Event::KeyDown { keycode: Some(Keycode::Num0), .. } => { table.current_elem = E_SPACE }  
                Event::KeyDown { keycode: Some(Keycode::R), .. } => { table.replacing = !table.replacing }
                Event::KeyDown { keycode: Some(Keycode::RightBracket), .. } => { table.brush_size = std::cmp::min(table.brush_size + 2, 20) }  
                Event::KeyDown { keycode: Some(Keycode::LeftBracket), .. } => { table.brush_size = std::cmp::max(table.brush_size as i32 - 2, 1) as usize }
                Event::MouseMotion { x, y, .. } => {
                    if y >= 19 {
                        sdl_context.mouse().show_cursor(false);
                    } else {
                        sdl_context.mouse().show_cursor(true);
                    }
                    let yy = y - 19;
                    table.mouse_over(x,yy);
                }

                Event::MouseButtonDown { x, y, mouse_btn: button,.. } => {
                    let yy = y - 19;
                    table.old_mouse_coords = (x as isize,yy as isize);
                    table.mouse_down(x,yy,button == MouseButton::Left);
                }
                Event::MouseButtonUp { x,y,.. } => {
                    let yy = y - 19;
                    table.mouse_up(x,yy);
                }
                Event::KeyDown { keycode: Some(Keycode::P),.. } if paused => {
                    paused = !paused;
                }
                Event::KeyDown { keycode: Some(Keycode::P),.. } => {
                    paused = !paused;
                } 
                Event::KeyDown { keycode: Some(Keycode::N),.. } => {
                    table = World::new()
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
    let window = video_subsystem.window("sand", WIDTH, HEIGHT)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    main_loop(window, &sdl_context);
}