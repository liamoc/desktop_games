
extern crate tesserae;
extern crate sdl2;
extern crate rand;
extern crate utils;

use rand::seq::SliceRandom;
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
use rand::{thread_rng,Rng};
use utils::color::{*};



struct GraphicsSet<T> {
    tile_set: TileSet,
    star: Graphic<T>,
    planets: [[Graphic<T>;7];6],
    ships: [Graphic<T>;7],
    hover_box: [Graphic<T>;2],
    attack_hover_box: [Graphic<T>;2],
    selection_box: [Graphic<T>;2]
}


impl <'r> GraphicsSet<Texture<'r>> {
    fn planet_graphic<T>(texture_creator: &'r TextureCreator<T>, tile_set : &TileSet, size: usize, fg: Color) -> Graphic<Texture<'r>> {
        if size < 3 {
            let mut t = Graphic::blank(1,1).textured(texture_creator);
            t.set_tile(0, 0, Tile { index: [188,443,457][size],  fg, bg: TRANSPARENT});
            t.update_texture(tile_set);
            t
        } else {
            let mut t = Graphic::blank(2,2).textured(texture_creator);
            let fg2 = if size >= 5 { TRANSPARENT} else { fg };
            let bg = if size >= 5 { fg } else { TRANSPARENT};
            t.set_tile(0, 0, Tile { index: [458,473,302][size-3], fg: fg2, bg });
            t.set_tile(1, 0, Tile { index: [459,442,303][size-3], fg: fg2, bg });
            t.set_tile(0, 1, Tile { index: [474,489,287][size-3], fg: fg2, bg });
            t.set_tile(1, 1, Tile { index: [475,490,286][size-3], fg: fg2, bg });
            t.update_texture(tile_set);
            t            
        }
    }
    fn ship_graphic<T>(texture_creator: &'r TextureCreator<T>, tile_set : &TileSet, fg: Color) -> Graphic<Texture<'r>> {
        let mut t = Graphic::blank(1,1).textured(texture_creator);
        t.set_tile(0, 0, Tile { index: 441,  fg, bg: TRANSPARENT});
        t.update_texture(tile_set);
        t
        
    }
    fn planet_box<T>(texture_creator: &'r TextureCreator<T>, tile_set : &TileSet, fg: Color, w:u32, h:u32) -> Graphic<Texture<'r>> {
        let mut t = Graphic::blank(w,h).textured(texture_creator);
        t.set_tile(0, 0, Tile {index:171,fg, bg: TRANSPARENT});
        t.set_tile(0, h-1, Tile {index:203,fg, bg: TRANSPARENT});
        t.set_tile(w-1, 0, Tile {index:173,fg, bg: TRANSPARENT});
        t.set_tile(w-1, h-1, Tile {index:205,fg, bg: TRANSPARENT});
        for x in 1..=w-2 {
            t.set_tile(x, 0, Tile{index:172,fg, bg:TRANSPARENT});
            t.set_tile(x, h-1, Tile{index:204,fg, bg:TRANSPARENT});
        }
        for y in 1..=h-2 {
            t.set_tile(0, y, Tile{index:187,fg, bg:TRANSPARENT});
            t.set_tile(w-1, y, Tile{index:189,fg, bg:TRANSPARENT});
        }
        t.update_texture(&tile_set);
        t
    }
    fn planet_graphics<T>(size: usize, tile_set : &TileSet, texture_creator: &'r TextureCreator<T>) -> [Graphic<Texture<'r>>;7] {
        [ Self::planet_graphic(texture_creator, tile_set,size,NEUTRAL_GRAY),
          Self::planet_graphic(texture_creator, tile_set,size,WHITE),
          Self::planet_graphic(texture_creator, tile_set,size,ORANGE),
          Self::planet_graphic(texture_creator, tile_set,size,YELLOW),
          Self::planet_graphic(texture_creator, tile_set,size,BRIGHT_GREEN),
          Self::planet_graphic(texture_creator, tile_set,size,PALE_PURPLE),
          Self::planet_graphic(texture_creator, tile_set,size,BLACK) ]
    }
    
    fn new<T>(texture_creator: &'r TextureCreator<T>) -> GraphicsSet<Texture<'r>> {
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
        let mut star = Graphic::blank(3, 3).textured(texture_creator);
        star.set_tile(1, 0, Tile{index:407, fg: YELLOW, bg: TRANSPARENT });
        star.set_tile(0, 1, Tile{index:422, fg: YELLOW, bg: TRANSPARENT });
        star.set_tile(1, 1, Tile{index:423, fg: YELLOW, bg: TRANSPARENT });
        star.set_tile(2, 1, Tile{index:424, fg: YELLOW, bg: TRANSPARENT });
        star.set_tile(0, 2, Tile{index:438, fg: YELLOW, bg: TRANSPARENT });
        star.set_tile(1, 2, Tile{index:439, fg: YELLOW, bg: TRANSPARENT });
        star.set_tile(2, 2, Tile{index:440, fg: YELLOW, bg: TRANSPARENT });
        star.update_texture(&tile_set);
        let planets = [
            Self::planet_graphics(0, &tile_set, texture_creator),
            Self::planet_graphics(1, &tile_set, texture_creator),
            Self::planet_graphics(2, &tile_set, texture_creator),
            Self::planet_graphics(3, &tile_set, texture_creator),
            Self::planet_graphics(4, &tile_set, texture_creator),
            Self::planet_graphics(5, &tile_set, texture_creator),
        ];
        let ships = [ 
            Self::ship_graphic(texture_creator, &tile_set,NEUTRAL_GRAY),
            Self::ship_graphic(texture_creator, &tile_set,WHITE),
            Self::ship_graphic(texture_creator, &tile_set,ORANGE),
            Self::ship_graphic(texture_creator, &tile_set,YELLOW),
            Self::ship_graphic(texture_creator, &tile_set,BRIGHT_GREEN),
            Self::ship_graphic(texture_creator, &tile_set,PALE_PURPLE),
            Self::ship_graphic(texture_creator, &tile_set,BLACK),
        ];
        let selection_box = [
            Self::planet_box(texture_creator, &tile_set, WHITE, 3,3),
            Self::planet_box(texture_creator, &tile_set, WHITE, 4,4)
        ];

        let attack_hover_box = [
            Self::planet_box(texture_creator, &tile_set, rgba(239, 41, 41, 255), 3,3),
            Self::planet_box(texture_creator, &tile_set, rgba(239, 41, 41, 255), 4,4)
        ];

        let hover_box = [
            Self::planet_box(texture_creator, &tile_set, PALE_BLUE, 3,3),
            Self::planet_box(texture_creator, &tile_set, PALE_BLUE, 4,4)
        ];
        GraphicsSet {
            tile_set, star, planets, ships, selection_box, hover_box, attack_hover_box
        }
    }

}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum OrbitOrigin {
    Star, Planet(usize)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Planet {
    origin: OrbitOrigin,
    orbit_distance: f64,
    delta_theta: f64,
    theta: f64,
    cartesian_cache: (f64,f64),
    cartesian_dirty: bool,
    owner: Option<usize>,
    garrison: u32,
    size: usize,
    construction_time: u32,
}
impl Planet {
    fn construction_cap( size: usize) -> u32 {
        750 - 60 * size as u32
    }
    fn planet( size: usize, orbit_distance: f64) -> Planet {
        let mut speed : f64 = 0.0; 
        while speed.abs() < 0.0001 {
            let scale = (WIDTH as f64/2.0 - orbit_distance) / (WIDTH as f64/2.0);
            speed = thread_rng().gen_range(-0.003 * scale, 0.003 * scale);
        }
        Planet {
            origin: OrbitOrigin::Star,
            orbit_distance,
            delta_theta: speed,
            theta: thread_rng().gen_range(0.0, std::f64::consts::TAU ),
            cartesian_cache: (0.0,0.0),
            cartesian_dirty: true,
            owner: None, 
            garrison: thread_rng().gen_range(0, 7 ),
            size, 
            construction_time: Planet::construction_cap(size),
        }
    }
    fn moon( orbiting: usize, size: usize, orbit_distance: f64) -> Planet {
        let mut speed : f64 = 0.0; 
        while speed.abs() < 0.0001 {
            speed = thread_rng().gen_range(-0.006, 0.006);
        }
        Planet {
            origin: OrbitOrigin::Planet(orbiting),
            orbit_distance,
            delta_theta: speed,
            theta: thread_rng().gen_range(0.0, std::f64::consts::TAU ),
            cartesian_cache: (0.0,0.0),
            cartesian_dirty: true,
            owner: None, 
            garrison: 0,
            size, 
            construction_time: Planet::construction_cap(size),

        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct AI {
    proportion_residents: f64, // propotion of ships to stay on planets 0.3 + frand( 0.4 );
    non_moon_priority : f64, // 0.5 + frand( 0.7 ) 
    enemy_priority : f64, // 0.05 + frand( 0.45 );
    preferred_priority : f64, // 0.2 + frand( 0.5 );
    neutral_priority : f64, //0.2 + frand( 0.6 );
    weaker_enemy_priority : f64, // 0.2 + frand( 1.6 );
    stronger_enemy_priority : f64, // 0.2 + frand( 1.6 );
    preferred_planets: Vec<usize>,
    ticks: u32,
}
impl AI {
    fn new() -> AI {
        AI {
            proportion_residents: thread_rng().gen_range(0.3, 0.3+0.4),
            non_moon_priority: thread_rng().gen_range(0.5, 0.5+0.7),
            enemy_priority: thread_rng().gen_range(0.05, 0.05+0.45),
            preferred_priority: thread_rng().gen_range(0.2, 0.2+0.5),
            neutral_priority: thread_rng().gen_range(0.2, 0.2+0.6),
            weaker_enemy_priority: thread_rng().gen_range(0.2, 0.2+1.6),
            stronger_enemy_priority: thread_rng().gen_range(0.2, 0.2+1.6),
            preferred_planets: Vec::new(),
            ticks: 120
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Ship {
    position: (f64,f64),
    target: usize,
    owner: usize,
    prep_time: u32,
}
pub struct SolarSystem {    
    planets: Vec<Planet>,
    ships_in_motion: Vec<Ship>, 
    computer_players: Vec<AI>,
    player_points: Vec<u32>,
    hovered: Option<usize>,
    selected: Vec<usize>,
    dispatch_size: f64,
    drawing_box: Option<(i32,i32)>,
    mouse_pos: (i32,i32),
    fog_of_war: bool,
    num_planets: usize
}

impl SolarSystem {
    fn new(num_planets: usize, computer_players: usize,fog_of_war: bool) -> SolarSystem {
        let mut planets = Vec::new();
        let available_space = WIDTH as f64/2.0 - 30.0;
        let increment = available_space / num_planets as f64;
        for i in 0..num_planets {
            planets.push(Planet::planet(thread_rng().gen_range(2, 6),i as f64*increment+25.0));
        }
        for i in 0..num_planets {
            let num_moons = match thread_rng().gen_range(0, 10) {
                0..=3 => 0,
                4..=8 => 1,
                _     => 2, 
            };
            let offset = if planets[i].size >= 3 { 17.0 } else { 12.0 };
            for j in 0..num_moons {
                planets.push(Planet::moon(i,thread_rng().gen_range(0,2),j as f64*6.0+offset))
            } 
        }
        let players = (1 + computer_players).min(num_planets).min(5);
        let mut players_vec : Vec<usize> = (0..players).into_iter().collect();
        players_vec.shuffle(&mut thread_rng());
        let mut p_index = 0;
        for v in players_vec {
            planets[p_index].garrison = 1;
            planets[p_index].owner = Some(v);
            p_index += 1;
        }
        let system = SolarSystem {
            planets,
            ships_in_motion: Vec::new(),
            computer_players: vec![AI::new();computer_players],
            player_points: vec![0;2+computer_players],
            hovered: None,
            selected: Vec::new(),
            dispatch_size: 0.5,
            drawing_box: None,
            mouse_pos: (0,0),
            fog_of_war,
            num_planets
        };
        system 
    }
    fn cartesian(&mut self, p : OrbitOrigin) -> (f64, f64) {
        match p {
            OrbitOrigin::Star => (WIDTH as f64/2.0, (HEIGHT as f64 - 18.0 )/2.0),
            OrbitOrigin::Planet(i) => {
                if self.planets[i].cartesian_dirty {
                    let x = self.planets[i].orbit_distance * self.planets[i].theta.cos();
                    let y = self.planets[i].orbit_distance * self.planets[i].theta.sin();
                    let origin = self.cartesian(self.planets[i].origin);
                    self.planets[i].cartesian_cache = (origin.0 + x, origin.1 + y);
                    self.planets[i].cartesian_dirty = false;
                }
                self.planets[i].cartesian_cache
            }
        }
    }

    fn distance(p1:(f64,f64), p2: (f64,f64)) -> f64 {
        let dx = p2.0 - p1.0;
        let dy = p2.1 - p1.1;
        (dx * dx + dy * dy).sqrt()
    }
    fn ai_get_random_nearby_planet(&mut self, planet : usize, player: usize) -> usize {
        let mut result : usize = planet;
        while result == planet {
            result = thread_rng().gen_range(0, self.planets.len());
        }

        let mut distance = Self::distance(self.cartesian(OrbitOrigin::Planet(planet)), self.cartesian(OrbitOrigin::Planet(result)));

        for _i in 0..self.planets.len() * 2 {
            let mut other_planet : usize = planet;
            while other_planet == planet {
                other_planet = thread_rng().gen_range(0, self.planets.len());
            }
            let mut other_distance = Self::distance(self.cartesian(OrbitOrigin::Planet(planet)), self.cartesian(OrbitOrigin::Planet(other_planet)));
            if self.planets[other_planet].owner != Some(player+1) {
                other_distance *= self.computer_players[player].enemy_priority;
                if self.computer_players[player].preferred_planets.contains(&other_planet) {
                    other_distance *= self.computer_players[player].preferred_priority;
                }
                if let Some(o) = self.planets[other_planet].owner {
                    if self.player_points[o] > self.player_points[player+1] {
                        other_distance *= self.computer_players[player].stronger_enemy_priority
                    } else {
                        other_distance *= self.computer_players[player].weaker_enemy_priority
                    }
                } else {
                    other_distance *= self.computer_players[player].neutral_priority
                }
            }
            if self.planets[other_planet].origin == OrbitOrigin::Star {
                other_distance *= self.computer_players[player].non_moon_priority
            }
            if other_distance < distance {
                result = other_planet;
                distance = other_distance;
            }
        }
        return result
    }
    fn ships_owned_by(&self, player: usize) -> u32 {
        let mut count = 0;
        count += self.garrisons_owned_by(player);
        for p in &self.ships_in_motion {
            if p.owner == player { count += 1 }
        }
        count
    }
    fn garrisons_owned_by(&self, player:usize) -> u32 {
        let mut count = 0;
        for i in &self.planets {
            if i.owner == Some(player) { count += i.garrison }
        }
        count
    }
    fn get_garrisons(&self, player: usize) -> Vec<usize> {
        let mut candidates = Vec::new();
        for i in 0..self.planets.len() {
            if self.planets[i].owner == Some(player) && self.planets[i].garrison > 0 { candidates.push(i) }
        }
        candidates
    }
    fn get_random_garrison(&self, player : usize) -> Option<usize> {
        let mut v = self.get_garrisons(player);
        v.shuffle(&mut thread_rng());
        v.pop()
    }
    fn get_random_nearby_garrison(&mut self, player: usize, planet: usize) -> Option<usize> {
        let mut candidates : Vec<usize> = self.get_garrisons(player).into_iter().filter(|x| *x != planet).collect();
        candidates.shuffle(&mut thread_rng());
        if candidates.len() <= 1 {
            return candidates.pop();
        } else {
            let mut result = planet;
            let mut distance = 0.0;
            for _i in 0..10 {
                if let Some(other_planet) = candidates.pop() {
                    let other_distance 
                        = Self::distance(self.cartesian(OrbitOrigin::Planet(other_planet)),
                                         self.cartesian(OrbitOrigin::Planet(planet)));
                    if other_planet != planet && other_distance < distance  {
                        distance = other_distance;
                        result = other_planet;
                    }
                }
            } 
            Some(result)
        }
    }
    fn ai_make_move(&mut self, player : usize) {
        let number_of_ships = self.ships_owned_by(player+1) as i32;
        let number_of_residents = self.garrisons_owned_by(player+1) as i32;
        let number_of_launches = number_of_residents - (number_of_ships as f64 * self.computer_players[player].proportion_residents ) as i32 - 1;
        if number_of_launches > 0 {
            let squadron_size = thread_rng().gen_range(1, number_of_launches + 1).min(10);
            let mut source_planet = self.get_random_garrison(player+1).unwrap();
            let mut destination_planet = self.ai_get_random_nearby_planet(source_planet, player);
            if self.planets[destination_planet].owner == Some(player+1) {
                if self.planets[destination_planet].garrison > self.planets[source_planet].garrison {
                    destination_planet = source_planet;
                }
            } else {
                if !self.computer_players[player].preferred_planets.contains(&destination_planet) {
                    self.computer_players[player].preferred_planets.push(destination_planet);
                }
            }
            if self.computer_players[player].preferred_planets.len() > 3 {
                self.computer_players[player].preferred_planets = Vec::new();
            }
            
            let mut launched_ships = 0;            
            for _i in 0..squadron_size {
                if self.planets[source_planet].garrison == 0 {
                    break;
                }
                self.planets[source_planet].garrison -= 1;
                launched_ships += 1;
                let pos = self.cartesian(OrbitOrigin::Planet(source_planet));                
                self.ships_in_motion.push(Ship {owner: player+1,position:pos, target: destination_planet, prep_time: thread_rng().gen_range(0, 30)});
                if let Some(next_source) = self.get_random_nearby_garrison(player+1, source_planet) {    
                    if Self::distance(self.cartesian(OrbitOrigin::Planet(next_source)), pos) < WIDTH as f64 / 4.0 {
                        source_planet = next_source;
                    }
                }
            }
            let wait = (( ( 1.0 + launched_ships as f64 ) / number_of_launches as f64 ) * thread_rng().gen_range(120.0, 180.0)) as u32;
            self.computer_players[player].ticks = wait;
            return
        }
        let wait = thread_rng().gen_range(60*3, 60*4+30);
        self.computer_players[player].ticks = wait;
    }

    fn draw<'t>(&mut self, canvas: &mut Canvas<Window>, graphics: &GraphicsSet<Texture<'t>>) {
        //canvas.set_draw_color(rgba(238,238,236,255));
        let (sx,sy) = self.cartesian(OrbitOrigin::Star);
        
        canvas.set_draw_color(CHARCOAL);
        canvas.clear();
        graphics.star.draw(canvas, (sx as i32-12,sy as i32+9-12));
        for i in 0..self.planets.len() {
            let (x,y) = self.cartesian(OrbitOrigin::Planet(i));
            let offset = if self.planets[i].size >= 3 { 8 } else { 4 };
            let col_index = self.planets[i].owner.map_or(0,|x| x + 1);
            graphics.planets[self.planets[i].size][6].draw(canvas, (x as i32-offset+1,y as i32+9-offset));
            graphics.planets[self.planets[i].size][6].draw(canvas, (x as i32-offset-1,y as i32+9-offset));
            graphics.planets[self.planets[i].size][6].draw(canvas, (x as i32-offset,y as i32+9-offset+1));
            graphics.planets[self.planets[i].size][6].draw(canvas, (x as i32-offset,y as i32+9-offset-1));
            graphics.planets[self.planets[i].size][col_index].draw(canvas, (x as i32-offset,y as i32+9-offset));
            if !self.fog_of_war || self.planets[i].owner==Some(0) {
                let increment = std::f64::consts::TAU / self.planets[i].garrison as f64;
                let r = 7.0 + self.planets[i].size as f64;
                for j in 0..self.planets[i].garrison {
                    let theta = j as f64 * increment;
                    let shx = theta.cos() * r + x;
                    let shy = theta.sin() * r + y;
                    graphics.ships[6].draw(canvas, (shx as i32 -4+1, shy as i32+9-4));
                    graphics.ships[6].draw(canvas, (shx as i32 -4-1, shy as i32+9-4));
                    graphics.ships[6].draw(canvas, (shx as i32 -4, shy as i32+9-4+1));
                    graphics.ships[6].draw(canvas, (shx as i32 -4, shy as i32+9-4-1));
                    graphics.ships[col_index].draw(canvas, (shx as i32 -4, shy as i32+9-4));
                }
            }
        }
        for s in 0..self.ships_in_motion.len() {  
            let (shx,shy) = self.ships_in_motion[s].position;
            graphics.ships[6].draw(canvas, (shx as i32 -4+1, shy as i32+9-4));
            graphics.ships[6].draw(canvas, (shx as i32 -4-1, shy as i32+9-4));
            graphics.ships[6].draw(canvas, (shx as i32 -4, shy as i32+9-4+1));
            graphics.ships[6].draw(canvas, (shx as i32 -4, shy as i32+9-4-1));
            graphics.ships[self.ships_in_motion[s].owner+1].draw(canvas, (shx as i32 -4, shy as i32+9-4));
        }
        if let Some(s) = self.hovered {
            let (x,y) = self.cartesian(OrbitOrigin::Planet(s));
            let offset = if self.planets[s].size >= 3  { 16 } else {  12 };
            let xx = x as i32 - offset;
            let yy = y as i32 + 9 - offset;
            if self.planets[s].owner == Some(0) {
                graphics.hover_box[if self.planets[s].size >= 3 { 1 } else { 0}].draw(canvas, (xx,yy));
            } else if !self.selected.is_empty() {
                graphics.attack_hover_box[if self.planets[s].size >= 3 { 1 } else { 0}].draw(canvas, (xx,yy));
            }
            
        }
        for s in self.selected.clone() {
            let (x,y) = self.cartesian(OrbitOrigin::Planet(s));
            let offset = if self.planets[s].size >= 3  { 16 } else {  12 };
            let xx = x as i32 - offset;
            let yy = y as i32 + 9 - offset;
            graphics.selection_box[if self.planets[s].size >= 3 { 1 } else { 0}].draw(canvas, (xx,yy));
        }
        if let Some(k) = self.drawing_box {
            canvas.set_draw_color(PALE_BLUE);
            let x = k.0.min(self.mouse_pos.0);
            let y = k.1.min(self.mouse_pos.1);
            canvas.draw_rect(Rect::new(x,y+9,(k.0-self.mouse_pos.0).abs() as u32, (k.1-self.mouse_pos.1).abs() as u32)).unwrap();
        }
        
    }
    fn mouse_over(&mut self, x : i32, y : i32) {
        self.mouse_pos = (x,y);
        if let Some((x0,y0)) = self.drawing_box {
            self.selected = Vec::new();
            for i in 0..self.planets.len() {
                let pos = self.cartesian(OrbitOrigin::Planet(i));
                let px = pos.0 as i32;
                let py = pos.1 as i32;
                let in_x = (x0..x).contains(&px) || (x..x0).contains(&px);
                let in_y = (y0..y).contains(&py) || (y..y0).contains(&py);
                if in_x && in_y {
                    if self.planets[i].owner == Some(0)  {
                        self.selected.push(i);
                    }
                }
            }
        } else {
            self.hovered = None;
            let mut d = 10000.0;
            for i in 0..self.planets.len() {
                let pos = self.cartesian(OrbitOrigin::Planet(i));
                let d2 = Self::distance(pos, (x as f64, y as f64));
                if d2 < d {
                    if d2 < (self.planets[i].size as f64 + 1.0).max(4.0) * 2.0 {
                        self.hovered = Some(i);
                        d = d2 ;
                    }
                }
            }
        }
    }
    fn mouse_up(&mut self, x : i32, y : i32) {
        if let Some(_) = self.drawing_box {
            self.mouse_over(x, y);
        }
        self.drawing_box = None;
    }
    fn select_all(&mut self) {
        self.selected = Vec::new();
        for i in 0..self.planets.len() {
            if self.planets[i].owner == Some(0)  {
                self.selected.push(i);
            }
        }
    }
    fn mouse_down(&mut self, x : i32, y : i32) {
        if let Some(p) = self.hovered {
            if self.planets[p].owner == Some(0) && !self.selected.contains(&p) {
                self.selected.push(p);
            } else {
                for k in self.selected.clone() {
                    let amount = (self.planets[k].garrison as f64 * self.dispatch_size) as u32;
                    self.planets[k].garrison -= amount;
                    let pos = self.cartesian(OrbitOrigin::Planet(k));
                    for _i in 0..amount {
                        self.ships_in_motion.push(Ship {owner: 0,position:pos, target: p, prep_time: thread_rng().gen_range(0, 30)});
                    }
                }
                self.selected = Vec::new();
            }
        } else {
            self.drawing_box = Some((x,y));
        }
    }
    fn tick(&mut self) {
        for i in 0..self.planets.len() {
            self.planets[i].theta += self.planets[i].delta_theta;
            self.planets[i].cartesian_dirty = true;

            self.planets[i].construction_time -= 1;
            if self.planets[i].construction_time == 0 {
                if self.planets[i].owner.is_some() {
                    self.planets[i].garrison += 1;
                }
                self.planets[i].construction_time = Planet::construction_cap(self.planets[i].size);
            }
        }
        for player in 0..self.player_points.len() {
            self.player_points[player] = 0;
            for p in &self.planets {
                self.player_points[player] += p.garrison + (p.size as u32 +1) * 10;
            }
        }
        for p in 0..self.computer_players.len() {
            if self.computer_players[p].ticks == 0 {
                self.ai_make_move(p);
            } else {
                self.computer_players[p].ticks -= 1;
            }
        }
        let mut to_remove = Vec::new();
        for s in 0..self.ships_in_motion.len() {
            if self.ships_in_motion[s].prep_time > 0 {
                self.ships_in_motion[s].prep_time -= 1;
                continue
            }
            let target = self.ships_in_motion[s].target;
            let target_pos = self.cartesian(OrbitOrigin::Planet(target));
            let hyp = Self::distance(target_pos, self.ships_in_motion[s].position);
            let dx = (target_pos.0 - self.ships_in_motion[s].position.0) / (hyp * 2.0);
            let dy = (target_pos.1 - self.ships_in_motion[s].position.1) / (hyp * 2.0);
            self.ships_in_motion[s].position.0 += dx;
            self.ships_in_motion[s].position.1 += dy;
            if hyp < 4.0 {
                if self.planets[target].garrison > 0 && self.planets[target].owner != Some(self.ships_in_motion[s].owner) {
                    self.planets[target].garrison -= 1;
                } else if self.planets[target].owner != Some(self.ships_in_motion[s].owner) {
                    self.planets[target].owner = Some(self.ships_in_motion[s].owner);
                    self.selected.retain(|x| *x != target);
                } else {
                    self.planets[target].garrison += 1;
                }
                to_remove.push(s)
            }
        }
        to_remove.sort();
        for j in 0..to_remove.len() {
            self.ships_in_motion.remove(to_remove[j]-j);
        }
    }
}
const WIDTH:u32=328;
const HEIGHT:u32=328+18;
fn main_loop(mut window:Window, sdl_context: &Sdl) {
    window.set_size(WIDTH,HEIGHT).unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    
    canvas.set_logical_size(WIDTH,HEIGHT).unwrap();
    canvas.set_blend_mode(BlendMode::Blend);
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let graphics_set = GraphicsSet::new(&texture_creator);
    let mut table = SolarSystem::new(11,2, false);
    let mut paused: bool = false;
    let mut deploy_gfx = Graphic::blank(4,1).textured(&texture_creator);
    let mut menu = MenuBar::new(WIDTH)
                    .add(Menu::new("GAME",132-(6*8)+16,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::submenu("2 Player", &texture_creator, &graphics_set.tile_set, 
                                 Menu::new(" ", 132-(2*8), &texture_creator,&graphics_set.tile_set)
                                .add(MenuItem::new("5 Planets", 352,Keycode::F1,&texture_creator,&graphics_set.tile_set))
                                .add(MenuItem::new("8 Planets", 353,Keycode::F2,&texture_creator,&graphics_set.tile_set))
                                .add(MenuItem::new("11 Planets", 354,Keycode::F3,&texture_creator,&graphics_set.tile_set))))
                            .add(MenuItem::submenu("3 Player", &texture_creator, &graphics_set.tile_set, 
                                Menu::new(" ", 132-(2*8), &texture_creator,&graphics_set.tile_set)
                                .add(MenuItem::new("5 Planets", 355,Keycode::F4,&texture_creator,&graphics_set.tile_set))
                                .add(MenuItem::new("8 Planets", 356,Keycode::F5,&texture_creator,&graphics_set.tile_set))
                                .add(MenuItem::new("11 Planets", 357,Keycode::F6,&texture_creator,&graphics_set.tile_set))))
                            .add(MenuItem::submenu("4 Player", &texture_creator, &graphics_set.tile_set, 
                                Menu::new(" ", 132-(2*8), &texture_creator,&graphics_set.tile_set)
                                    .add(MenuItem::new("5 Planets", 27,Keycode::Z,&texture_creator,&graphics_set.tile_set))
                                    .add(MenuItem::new("8 Planets", 25,Keycode::X,&texture_creator,&graphics_set.tile_set))
                                    .add(MenuItem::new("11 Planets", 4,Keycode::C,&texture_creator,&graphics_set.tile_set))))
                            .add(MenuItem::submenu("5 Player", &texture_creator, &graphics_set.tile_set, 
                                Menu::new(" ", 132-(2*8), &texture_creator,&graphics_set.tile_set)
                                    .add(MenuItem::new("5 Planets", 22,Keycode::U,&texture_creator,&graphics_set.tile_set))
                                    .add(MenuItem::new("8 Planets", 10,Keycode::I,&texture_creator,&graphics_set.tile_set))
                                    .add(MenuItem::new("11 Planets", 16,Keycode::O,&texture_creator,&graphics_set.tile_set))))
                            .add(MenuItem::separator(132-(5*8)+2, &texture_creator, &graphics_set.tile_set))                            
                            .add(MenuItem::new("Pause", 17, Keycode::P,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Restart",15, Keycode::N,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::separator(132-(5*8)+2, &texture_creator, &graphics_set.tile_set))                            
                            .add(MenuItem::new("Quit",363, Keycode::F12,&texture_creator,&graphics_set.tile_set)))
                    .add(Menu::new("FORCES",92+16+16,&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Deploy 33%", 52, Keycode::Num1,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Deploy 50%", 53, Keycode::Num2,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Deploy 66%", 54, Keycode::Num3,&texture_creator,&graphics_set.tile_set))
                            .add(MenuItem::new("Deploy 100%", 55, Keycode::Num4,&texture_creator,&graphics_set.tile_set))
                        )
                    .add(Menu::new("VIEW",72+(5*8),&texture_creator,&graphics_set.tile_set)
                            .add(MenuItem::new("Fog of war",7, Keycode::F,&texture_creator, &graphics_set.tile_set))
                            .add(MenuItem::separator(72+5*8-14, &texture_creator, &graphics_set.tile_set))
                            .add(MenuItem::new("Micro-mode",360, Keycode::F9,&texture_creator, &graphics_set.tile_set))
                            );
    let mut rate_limiter = FPSManager::new();
    rate_limiter.set_framerate(60).unwrap();
    let mut micro_mode = false;
    let mut paused_splash = Graphic::load_from(Cursor::new(&include_bytes!("../paused_splash")[..])).unwrap().textured(&texture_creator);
    paused_splash.update_texture(&graphics_set.tile_set);

    loop {

        if !paused { table.tick(); };
        table.draw(&mut canvas, &graphics_set);
        if paused {
            paused_splash.draw(&mut canvas,((WIDTH as i32/2)- (11*4),HEIGHT as i32-48));
        }
        deploy_gfx.draw_rect(0, 0, 4, 1, Tile {fg: TRANSPARENT, bg: TRANSPARENT, index:0});
        deploy_gfx.draw_text(&(((table.dispatch_size * 100.0) as i32).to_string() + "%"), &graphics_set.tile_set , 0, 0, WHITE, TRANSPARENT);
        deploy_gfx.update_texture(&graphics_set.tile_set);
        menu.draw(&mut canvas);
        deploy_gfx.draw(&mut canvas, (4 as i32,HEIGHT as i32-12));
        canvas.present();
        for event in event_pump.poll_iter() {
            let h = menu.handle_event(event.clone(), &mut event_subsystem);
            match event {
                _ if h => {},
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::F12), .. } => {
                    return
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
                Event::KeyDown { keycode: Some(Keycode::A), ..} => {
                    table.select_all()
                }
                Event::KeyDown { keycode: Some(Keycode::F1), .. } => { table = SolarSystem::new(5,1,table.fog_of_war); paused = true }
                Event::KeyDown { keycode: Some(Keycode::F2), .. } => { table = SolarSystem::new(8,1,table.fog_of_war); paused = true }
                Event::KeyDown { keycode: Some(Keycode::F3), .. } => { table = SolarSystem::new(11,1,table.fog_of_war); paused = true }
                Event::KeyDown { keycode: Some(Keycode::F4), .. } => { table = SolarSystem::new(5,2,table.fog_of_war); paused = true}
                Event::KeyDown { keycode: Some(Keycode::F5), .. } => { table = SolarSystem::new(8,2,table.fog_of_war); paused = true}
                Event::KeyDown { keycode: Some(Keycode::F6), .. } => { table = SolarSystem::new(11,2,table.fog_of_war); paused = true}
                Event::KeyDown { keycode: Some(Keycode::Z), .. } => { table = SolarSystem::new(5,3,table.fog_of_war); paused = true}
                Event::KeyDown { keycode: Some(Keycode::X), .. } => { table = SolarSystem::new(8,3,table.fog_of_war); paused = true}
                Event::KeyDown { keycode: Some(Keycode::C), .. } => { table = SolarSystem::new(11,3,table.fog_of_war); paused = true}
                Event::KeyDown { keycode: Some(Keycode::U), .. } => { table = SolarSystem::new(5,4,table.fog_of_war); paused = true}
                Event::KeyDown { keycode: Some(Keycode::I), .. } => { table = SolarSystem::new(8,4,table.fog_of_war); paused = true}
                Event::KeyDown { keycode: Some(Keycode::O), .. } => { table = SolarSystem::new(11,4,table.fog_of_war); paused = true}
                Event::KeyDown { keycode: Some(Keycode::N), .. } => { table = SolarSystem::new(table.num_planets,table.computer_players.len(),table.fog_of_war); paused = true}
                Event::KeyDown { keycode: Some(Keycode::F), ..} => {
                    table.fog_of_war = !table.fog_of_war;
                }
                Event::MouseMotion { x, y, .. } => {
                    let yy = y - 9;
                    table.mouse_over(x,yy);
                }

                Event::MouseButtonDown { x, y, .. } => {
                    if paused {
                        paused = false
                    } else {
                        let yy = y - 9;
                        table.mouse_down(x,yy);
                    }
                }

                Event::MouseButtonUp { x,y,.. } => {
                    let yy = y - 9;
                    table.mouse_up(x,yy);
                }
                Event::KeyDown { keycode: Some(Keycode::Num1), .. } => {
                    table.dispatch_size = 0.33
                }
                Event::KeyDown { keycode: Some(Keycode::Num2), .. } => {
                    table.dispatch_size = 0.5
                }
                Event::KeyDown { keycode: Some(Keycode::Num3), .. } => {
                    table.dispatch_size = 0.66
                }
                Event::KeyDown { keycode: Some(Keycode::Num4), .. } => {
                    table.dispatch_size = 1.0
                }
                Event::KeyDown { .. } if paused => {
                    paused = !paused;
                }
                Event::KeyDown { keycode: Some(Keycode::P),.. } => {
                    paused = !paused;
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
    let window = video_subsystem.window("bellum", WIDTH, HEIGHT)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    main_loop(window, &sdl_context);
}