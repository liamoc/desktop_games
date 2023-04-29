extern crate tesserae;
extern crate sdl2;

use rand::Rng;
use tesserae::{*};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use utils::framerate::FPSManager;
use sdl2::render::TextureCreator;
use sdl2::render::RenderTarget;
use sdl2::render::Texture;
use sdl2::render::Canvas;
use utils::color::{*};
use sdl2::rect::{Rect};
use utils::menu::{*};
use rand::thread_rng;
use std::io::Cursor;
use std::collections::VecDeque;
use utils::{*};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Direction { N, S, E, W  }
impl Direction {
    const ALL : [Direction;4] = [Self::N, Self::S, Self::E, Self::W  ];
    fn random_dir() -> Direction {
        let i = thread_rng().gen_range(0,Direction::ALL.len());
        Direction::ALL[i as usize]
    }
    fn move_point_by (&self, p: (i32,i32), i:i32) -> (i32,i32) {
        match *self {
            Self::N => (p.0,p.1-i),
            Self::S => (p.0,p.1+i),
            Self::E => (p.0+i,p.1),
            Self::W => (p.0-i,p.1),
        }
    }
    fn move_point (&self, p: (i32,i32)) -> (i32,i32) {
        match *self {
            Self::N => (p.0,p.1-1),
            Self::S => (p.0,p.1+1),
            Self::E => (p.0+1,p.1),
            Self::W => (p.0-1,p.1),
        }
    }
    fn opposite(&self) -> Direction {
        match *self {
            Self::N => Self::S,
            Self::S => Self::N,
            Self::E => Self::W,
            Self::W => Self::E
        }
    }
    fn to_index(&self) -> usize {
        match *self {
            Self::E => 1,
            Self::N => 2,
            Self::W => 3,
            Self::S => 4,
        }

    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum PowerUpType { Bomb, Speed, Radius, OneUp, Invulnerability, Pass, Kick, Remote, LevelExit }

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Enemy { Mouse(Direction, Direction), Cat(Option<Direction>, u32), Ghost(Option<Direction>, u32), Cylon(Option<Direction>, u32), DangerCylon(Option<Direction>, u32, bool) }
impl Enemy {
    fn kill_score(&self) -> u32 {
        match *self {
            Enemy::Mouse(..) => 10,
            Enemy::Cat(..) => 15,
            Enemy::Ghost(..) => 20,
            Enemy::Cylon(..) => 20,
            Enemy::DangerCylon(..) => 25,
        }
    }
}


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum GameObject { Box(Option<PowerUpType>), Block, Explosion(ExplosionType, i32), Bomb(u32, usize), PowerUp(PowerUpType, u32) }


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ExplosionType { Dir(Direction, usize), Box(Option<PowerUpType>), Centre(usize) }


struct GraphicsSet<'r> {
    player: [OutlinedTile<'r>;4],
    player_hl: [OutlinedTile<'r>;4],
    block: Graphic<Texture<'r>>,
    explosion: [Graphic<Texture<'r>>;8],
    push_box: Graphic<Texture<'r>>,
    cat: OutlinedTile<'r>,
    cylon: OutlinedTile<'r>,
    danger_cylon: OutlinedTile<'r>,
    danger_cylon_hl: OutlinedTile<'r>,
    radius_icon:OutlinedTile<'r>,
    speed_icon: OutlinedTile<'r>,
    bomb: OutlinedTile<'r>,    
    red_bomb: OutlinedTile<'r>,
    red_bomb_hl: OutlinedTile<'r>,
    pu_bomb: Graphic<Texture<'r>>,
    pu_remote: Graphic<Texture<'r>>,
    pu_speed: Graphic<Texture<'r>>,
    pu_radius: Graphic<Texture<'r>>,
    pu_1up: Graphic<Texture<'r>>,
    pu_inv: Graphic<Texture<'r>>,
    pu_pass: Graphic<Texture<'r>>,
    pu_kick: Graphic<Texture<'r>>,
    level_exit: Graphic<Texture<'r>>,
    level_exit_locked: Graphic<Texture<'r>>,
    mouse: OutlinedTile<'r>,
    ghost: OutlinedTile<'r>,
}

struct Game<'r> {
    player: (i32,i32),
    facing: Direction,
    enemies: Vec<((i32,i32), Enemy, u32)>,
    moving_bombs: Vec<((i32,i32),Direction,u32,usize)>,
    tile_set: TileSet,
    gfx: GraphicsSet<'r>,
    tiles: [[Option<GameObject>;HEIGHT as usize];WIDTH as usize],
    speed:u32,
    won_level: bool,
    dead: bool,
    lives: u32,
    score: u32,
    dying: u32,
    bomb_radius: usize,
    bombs: usize,
    iframes: usize,
    invincibility_pu: bool,
    kick_pu: bool,
    pass_pu: bool,
    remote_pu: bool,
    active_tiles: Vec<(i32,i32)>,
    pressed: Vec<Direction>,
    stop_bombs: bool,
    explode_bombs: bool,
    tick: u32,
    enemy_tick: u32
}


const WIDTH : u32 = 23;
const HEIGHT : u32 = 23;

fn load_from_graphic(gfx: Graphic<()>) -> [[Option<GameObject>;HEIGHT as usize];WIDTH as usize] {
    let mut ret = [[None;HEIGHT as usize];WIDTH as usize];
    for i in 0..WIDTH {
        for j in 0..HEIGHT {
            ret[i as usize][j as usize] = match gfx[(i,j)].index {
                1 => Some(GameObject::Block),
                _ => None
            };
        }
    }
    ret
}
struct Level {
    map: [[Option<GameObject>;HEIGHT as usize];WIDTH as usize],
    quotas: [u32;15],
}



fn level(index: u32) -> Level {
    match index {
        1 => Level {
            map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level1")[..])).unwrap()),
            quotas: [
                1, // 1Ups
                2, // Speed Ups
                3, // Radius Ups
                4, // Bomb Ups
                2, // Invulnerability
                0, // Pass Powerups
                0, // Kick Powerups
                0, // Remote Powerups
                100, // Empty Blocks
                10, // Mice
                1, // Cats
                0, // Ghosts
                0, // Cylons
                0, // Danger Cylons                    
                1, // Exits
            ]
        },
        2 => Level {
            map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level1")[..])).unwrap()),
            quotas: [
                1, // 1Ups
                3, // Speed Ups
                4, // Radius Ups
                4, // Bomb Ups
                2, // Invulnerability
                0, // Pass Powerups
                0, // Kick Powerups
                1, // Remote Powerups
                130, // Empty Blocks
                16, // Mice
                2, // Cats
                0, // Ghosts
                0, // Cylons
                0, // Danger Cylons                    
                1, // Exits
            ]
        },
        3 => Level {
            map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level1")[..])).unwrap()),
            quotas: [
                1, // 1Ups
                3, // Speed Ups
                4, // Radius Ups
                4, // Bomb Ups
                3, // Invulnerability
                0, // Pass Powerups
                1, // Kick Powerups
                1, // Remote Powerups
                150, // Empty Blocks
                18, // Mice
                3, // Cats
                4, // Ghosts
                0, // Cylons
                0, // Danger Cylons                    
                1, // Exits
            ]
        },
        4 => Level {
            map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level1")[..])).unwrap()),
            quotas: [
                2, // 1Ups
                3, // Speed Ups
                5, // Radius Ups
                5, // Bomb Ups
                3, // Invulnerability
                1, // Pass Powerups
                1, // Kick Powerups
                1, // Remote Powerups
                160, // Empty Blocks
                20, // Mice
                5, // Cats
                4, // Ghosts
                3, // Cylons
                0, // Danger Cylons                    
                1, // Exits
            ]
        },
        5 => Level {
            map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level1")[..])).unwrap()),
            quotas: [
                2, // 1Ups
                3, // Speed Ups
                5, // Radius Ups
                5, // Bomb Ups
                3, // Invulnerability
                1, // Pass Powerups
                1, // Kick Powerups
                1, // Remote Powerups
                150, // Empty Blocks
                15, // Mice
                8, // Cats
                6, // Ghosts
                7, // Cylons
                0, // Danger Cylons                    
                1, // Exits
            ]
        },
        6 => Level {
            map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level1")[..])).unwrap()),
            quotas: [
                2, // 1Ups
                3, // Speed Ups
                5, // Radius Ups
                5, // Bomb Ups
                3, // Invulnerability
                1, // Pass Powerups
                1, // Kick Powerups
                1, // Remote Powerups
                180, // Empty Blocks
                10, // Mice
                10, // Cats
                8, // Ghosts
                7, // Cylons
                2, // Danger Cylons                    
                1, // Exits
            ]
        },
        7 => Level {
            map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level1")[..])).unwrap()),
            quotas: [
                2, // 1Ups
                3, // Speed Ups
                5, // Radius Ups
                5, // Bomb Ups
                3, // Invulnerability
                1, // Pass Powerups
                1, // Kick Powerups
                1, // Remote Powerups
                160, // Empty Blocks
                12, // Mice
                10, // Cats
                10, // Ghosts
                10, // Cylons
                3, // Danger Cylons                    
                1, // Exits
            ]
        },
        8 => Level {
            map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level1")[..])).unwrap()),
            quotas: [
                2, // 1Ups
                4, // Speed Ups
                6, // Radius Ups
                6, // Bomb Ups
                4, // Invulnerability
                1, // Pass Powerups
                1, // Kick Powerups
                1, // Remote Powerups
                160, // Empty Blocks
                15, // Mice
                10, // Cats
                12, // Ghosts
                13, // Cylons
                4, // Danger Cylons                    
                1, // Exits            
            ]
        },
        9 => Level {
            map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level1")[..])).unwrap()),
            quotas: [
                2, // 1Ups
                4, // Speed Ups
                6, // Radius Ups
                6, // Bomb Ups
                5, // Invulnerability
                1, // Pass Powerups
                1, // Kick Powerups
                1, // Remote Powerups
                165, // Empty Blocks
                10, // Mice
                5, // Cats
                5, // Ghosts
                10, // Cylons
                10, // Danger Cylons                    
                1, // Exits
            ]
        },
        _ => Level {
            map:load_from_graphic(Graphic::load_from(Cursor::new(&include_bytes!("../level1")[..])).unwrap()),
            quotas: [
                2, // 1Ups
                4, // Speed Ups
                6, // Radius Ups
                6, // Bomb Ups
                5, // Invulnerability
                1, // Pass Powerups
                1, // Kick Powerups
                1, // Remote Powerups
                175 - index, // Empty Blocks
                10, // Mice
                5, // Cats
                5, // Ghosts
                10, // Cylons
                index, // Danger Cylons                    
                1, // Exits
            ]
    },
    }
}


impl <'r>Game<'r> {
    
    fn new<T>(texture_creator : &'r TextureCreator<T>) -> Game<'r> {
        let tile_set = TileSet::load_from(Cursor::new(&include_bytes!("../../tiles")[..]));
        let player_gfx = [
            OutlinedTile::new(416,WHITE,&tile_set,texture_creator),
            OutlinedTile::new(417,WHITE,&tile_set,texture_creator),
            OutlinedTile::new(418,WHITE,&tile_set,texture_creator),
            OutlinedTile::new(419,WHITE,&tile_set,texture_creator),
        ];
        let player_gfx_hl = [
            OutlinedTile::new(416,PALE_PURPLE,&tile_set,texture_creator),
            OutlinedTile::new(417,PALE_PURPLE,&tile_set,texture_creator),
            OutlinedTile::new(418,PALE_PURPLE,&tile_set,texture_creator),
            OutlinedTile::new(419,PALE_PURPLE,&tile_set,texture_creator),
        ];
        let mut explosion: [Graphic<Texture<'r>>;8] = [
            Graphic::load_from(Cursor::new(&include_bytes!("../explosion_c")[..])).unwrap().textured(&texture_creator),
            Graphic::load_from(Cursor::new(&include_bytes!("../explosion_ns")[..])).unwrap().textured(&texture_creator),
            Graphic::load_from(Cursor::new(&include_bytes!("../explosion_ew")[..])).unwrap().textured(&texture_creator),
            Graphic::load_from(Cursor::new(&include_bytes!("../explosion_b")[..])).unwrap().textured(&texture_creator), 
            Graphic::load_from(Cursor::new(&include_bytes!("../explosion_n")[..])).unwrap().textured(&texture_creator), 
            Graphic::load_from(Cursor::new(&include_bytes!("../explosion_s")[..])).unwrap().textured(&texture_creator), 
            Graphic::load_from(Cursor::new(&include_bytes!("../explosion_e")[..])).unwrap().textured(&texture_creator),
            Graphic::load_from(Cursor::new(&include_bytes!("../explosion_w")[..])).unwrap().textured(&texture_creator),         
        ];

        for i in &mut explosion {
            i.update_texture(&tile_set);
        }
        let speed_icon = OutlinedTile::new(405,PALE_BLUE,&tile_set,texture_creator);
        let mut block = Graphic::load_from(Cursor::new(&include_bytes!("../garbage_block")[..])).unwrap().textured(&texture_creator);
        block.update_texture(&tile_set);
        let mut pu_bomb = Graphic::load_from(Cursor::new(&include_bytes!("../pu_bomb")[..])).unwrap().textured(&texture_creator);
        pu_bomb.update_texture(&tile_set);
        let mut pu_speed = Graphic::load_from(Cursor::new(&include_bytes!("../pu_speed")[..])).unwrap().textured(&texture_creator);
        pu_speed.update_texture(&tile_set);
        let mut pu_radius = Graphic::load_from(Cursor::new(&include_bytes!("../pu_radius")[..])).unwrap().textured(&texture_creator);
        pu_radius.update_texture(&tile_set);
        let mut pu_1up = Graphic::load_from(Cursor::new(&include_bytes!("../pu_1up")[..])).unwrap().textured(&texture_creator);
        pu_1up.update_texture(&tile_set);
        let mut pu_inv = Graphic::load_from(Cursor::new(&include_bytes!("../pu_inv")[..])).unwrap().textured(&texture_creator);
        pu_inv.update_texture(&tile_set);
        let mut pu_pass = Graphic::load_from(Cursor::new(&include_bytes!("../pu_pass")[..])).unwrap().textured(&texture_creator);
        pu_pass.update_texture(&tile_set);
        let mut level_exit = Graphic::load_from(Cursor::new(&include_bytes!("../level_exit")[..])).unwrap().textured(&texture_creator);
        level_exit.update_texture(&tile_set);
        let mut level_exit_locked = Graphic::load_from(Cursor::new(&include_bytes!("../level_exit_locked")[..])).unwrap().textured(&texture_creator);
        level_exit_locked.update_texture(&tile_set);
        let mut pu_remote = Graphic::load_from(Cursor::new(&include_bytes!("../pu_remote")[..])).unwrap().textured(&texture_creator);
        pu_remote.update_texture(&tile_set);
        let mut pu_kick = Graphic::load_from(Cursor::new(&include_bytes!("../pu_kick")[..])).unwrap().textured(&texture_creator);
        pu_kick.update_texture(&tile_set);
        let mut push_box = Graphic::load_from(Cursor::new(&include_bytes!("../movable_block")[..])).unwrap().textured(&texture_creator);
        push_box.update_texture(&tile_set);
        let cat = OutlinedTile::new(151,ORANGE,&tile_set,texture_creator);

        let cylon = OutlinedTile::new(148,TEAL,&tile_set,texture_creator);
        let danger_cylon = OutlinedTile::new(150,BRIGHT_GREEN,&tile_set,texture_creator);
        let danger_cylon_hl = OutlinedTile::new(150,DARK_RED,&tile_set,texture_creator);
        let bomb = OutlinedTile::new(404,GREEN,&tile_set,texture_creator);
        let red_bomb = OutlinedTile::new(404,ORANGE,&tile_set,texture_creator);
        let red_bomb_hl = OutlinedTile::new(404,PALE_ORANGE,&tile_set,texture_creator);
        let radius_icon = OutlinedTile::new(149,ORANGE,&tile_set,texture_creator);
        let mouse =  OutlinedTile::new(153,NEUTRAL_GRAY,&tile_set,texture_creator);
        let ghost =  OutlinedTile::new(420,WHITE,&tile_set,texture_creator);
        Game {
            tile_set: tile_set,
            player: (12*(WIDTH as i32/2),12*(HEIGHT as i32/2)),
            won_level: false,
            dead: false,
            invincibility_pu: false,
            pass_pu: false,
            kick_pu: false,
            remote_pu: false,
            facing: Direction::S,   
            stop_bombs: false,
            explode_bombs: false,
            enemies: Vec::new(),
            moving_bombs: Vec::new(),
            pressed: Vec::new(),
            tiles: [[None;HEIGHT as usize]; WIDTH as usize],
            lives: 3,
            score: 0,
            speed: 1,
            bombs: 1,
            iframes: 0,
            tick: 0,
            enemy_tick: 0,
            dying: 0,
            bomb_radius: 1,
            gfx: GraphicsSet{ level_exit,level_exit_locked,ghost, cylon,danger_cylon,danger_cylon_hl,cat,mouse, radius_icon,speed_icon,pu_pass,pu_inv,pu_radius,pu_1up,pu_bomb,pu_remote,pu_kick,pu_speed, bomb, red_bomb, red_bomb_hl, explosion, block, push_box, player_hl: player_gfx_hl, player: player_gfx },
            active_tiles: Vec::new(),
        }
    }
    fn player_tile_pos(&self) -> (i32,i32) {
        let x = (self.player.0 + 6) / 12;
        let y = (self.player.1 + 6) / 12;
        (x,y)
    }
    fn enemies_at(&self, pos:(i32,i32)) -> bool {
        self.enemies.iter().any(|x| Self::to_tile_pos(x.0) == pos)
    }
    fn set_up_level(&mut self, lev : u32) {
        let mut l = level(lev);
        self.player =  (12,12);
        self.enemies = Vec::new();
        self.facing = Direction::S;
        for i in 0..l.quotas.len() {
            for _ in 0..l.quotas[i] {
                let mut x = 0;
                let mut y = 0;
                while (x < 4 && y < 4) || l.map[x as usize][y as usize] != None || self.enemies_at((x,y)) {
                    x = thread_rng().gen_range(1,WIDTH as i32-1);
                    y = thread_rng().gen_range(1,HEIGHT as i32-1);
                }
                match i {
                    0 => l.map[x as usize][y as usize] = Some(GameObject::Box(Some(PowerUpType::OneUp))),
                    1 => l.map[x as usize][y as usize] = Some(GameObject::Box(Some(PowerUpType::Speed))),
                    2 => l.map[x as usize][y as usize] = Some(GameObject::Box(Some(PowerUpType::Radius))),
                    3 => l.map[x as usize][y as usize] = Some(GameObject::Box(Some(PowerUpType::Bomb))),
                    4 => l.map[x as usize][y as usize] = Some(GameObject::Box(Some(PowerUpType::Invulnerability))),
                    5 => l.map[x as usize][y as usize] = Some(GameObject::Box(Some(PowerUpType::Pass))),
                    6 => l.map[x as usize][y as usize] = Some(GameObject::Box(Some(PowerUpType::Kick))),
                    7 => l.map[x as usize][y as usize] = Some(GameObject::Box(Some(PowerUpType::Remote))),
                    8 => l.map[x as usize][y as usize] = Some(GameObject::Box(None)),
                    9 => self.enemies.push(((x * 12, y * 12), Enemy::Mouse(Direction::S, Direction::E), 0)),
                    10 => self.enemies.push(((x * 12, y * 12), Enemy::Cat(None,12), 0)),
                    11 => self.enemies.push(((x * 12, y * 12), Enemy::Ghost(None,12), 0)),
                    12 => self.enemies.push(((x * 12, y * 12), Enemy::Cylon(None,12), 0)),
                    13 => self.enemies.push(((x * 12, y * 12), Enemy::DangerCylon(None,12,false), 0)),
                    _ => l.map[x as usize][y as usize] = Some(GameObject::Box(Some(PowerUpType::LevelExit))),
                }
            }
        }
        self.won_level = false;
        self.dead = false;
        self.tiles = l.map;
        self.moving_bombs = Vec::new();
        self.active_tiles = Vec::new();
        self.bombs = 1;
        self.iframes = 100;
        self.bomb_radius = 1;
        self.speed = 0;
        self.explode_bombs = false;
        self.stop_bombs = false;
        self.pass_pu = false;
        self.kick_pu = false;
        self.remote_pu = false;
        self.invincibility_pu = false;
    }

    fn _is_blank_or_cat(i : Option<GameObject>) -> bool {
        match i {
            None => true,
            _ => false
        }
    }

    fn cost_map(&self) -> [[usize;HEIGHT as usize]; WIDTH as usize] {
        let mut ret = [[std::usize::MAX; HEIGHT as usize];WIDTH as usize];
        let selfplayer = self.player_tile_pos();
        ret[selfplayer.0 as usize][selfplayer.1 as usize] = 0;
        let mut q : VecDeque<(i32,i32)> = VecDeque::new();
        q.push_back(selfplayer);
        while let Some((x,y)) = q.pop_front() {
            let c = ret[x as usize][y as usize] + 1;
            for d in Direction::ALL.iter() {
                let p = d.move_point((x,y));
                if Self::clamp_bounds(p) == p && !self.dangerous(p, false) {
                    if ret[p.0 as usize][p.1 as usize] > c {
                        ret[p.0 as usize][p.1 as usize] = c;
                        q.push_back(p);
                    }
                }
            }
        }
        ret
    }
    const EXPLOSION_TIME : i32 =2;
    const POWERUP_TIME : u32 = 1000;
    fn place_explosion(&mut self, p:(i32,i32), typ: ExplosionType) {
        match self.tiles[p.0 as usize][p.1 as usize] {
            Some(GameObject::Block) => return,
            Some(GameObject::PowerUp(PowerUpType::LevelExit,_)) => return,
            Some(GameObject::Box(pu)) => self.tiles[p.0 as usize][p.1 as usize] = Some(GameObject::Explosion(ExplosionType::Box(pu), Self::EXPLOSION_TIME)),
            Some(GameObject::Bomb(_,r)) => self.tiles[p.0 as usize][p.1 as usize] = {
                self.bombs += 1; Some(GameObject::Explosion(ExplosionType::Centre(r), Self::EXPLOSION_TIME))
            },
            _ => self.tiles[p.0 as usize][p.1 as usize] = Some(GameObject::Explosion(typ, Self::EXPLOSION_TIME)),
        };
        self.active_tiles.push(p); 
    }
    fn place_bomb_at_player(&mut self) {
        if self.bombs > 0 {            
            self.place_bomb(self.player_tile_pos(),self.bomb_radius);
        }        
    }
    fn place_bomb(&mut self, p : (i32, i32), radius: usize) {
        if self.obstructed(p, false, false) 
        || match self.tiles[p.0 as usize][p.1 as usize] { Some(GameObject::PowerUp(PowerUpType::LevelExit, _)) => true, _ => false } { 
            return; 
        }
        self.tiles[p.0 as usize][p.1 as usize] = Some(GameObject::Bomb(80, radius));
        self.active_tiles.push(p);
        self.bombs -= 1;
    }
    fn player_tile_collisions(&mut self) {
        let (x,y) = self.player_tile_pos();
        match self.tiles[x as usize][y as usize] {
            Some(GameObject::Explosion(..)) if self.iframes == 0 => {self.dying = 100},
            Some(GameObject::PowerUp(pu,_)) => {
                match pu {
                    PowerUpType::Bomb => {self.bombs += 1},
                    PowerUpType::Speed => {self.speed += 10},
                    PowerUpType::Radius => {self.bomb_radius += 1},
                    PowerUpType::OneUp => {self.lives += 1},
                    PowerUpType::Invulnerability => {self.iframes += 600; self.invincibility_pu = true},
                    PowerUpType::Pass => {self.pass_pu = true},
                    PowerUpType::Kick => {self.kick_pu = true},
                    PowerUpType::Remote => {self.remote_pu = true},
                    PowerUpType::LevelExit => {if self.enemies.is_empty() { self.won_level = true; }; return},
                }
                self.score += 5;
                self.tiles[x as usize][y as usize] = None
            },
            _ => {},
        }
    }
    fn tick_active_tiles(&mut self) {
        let old_active_tiles = self.active_tiles.clone();
        self.active_tiles = Vec::new();
        for p in old_active_tiles {
            match self.tiles[p.0 as usize][p.1 as usize] {
                Some(GameObject::PowerUp(ty, fuse)) => {
                    if fuse == 0 {
                        self.tiles[p.0 as usize][p.1 as usize] = None;
                        continue;
                    } else if ty != PowerUpType::LevelExit {
                        self.tiles[p.0 as usize][p.1 as usize] = Some(GameObject::PowerUp(ty, fuse - 1))
                    }
                }
                Some(GameObject::Bomb(fuse, radius)) => {
                    if fuse == 0 || self.explode_bombs {
                        self.place_explosion(p, ExplosionType::Centre(radius))
                    } else {
                        self.tiles[p.0 as usize][p.1 as usize] = Some(GameObject::Bomb(fuse - 1, radius));
                    }
                }
                Some(GameObject::Explosion(typ, t)) => {
                    if t > 0 {
                        self.tiles[p.0 as usize][p.1 as usize] = Some(GameObject::Explosion(typ, t - 1));
                    } else if t == 0 {
                        match typ {
                            ExplosionType::Dir(d, r) if r > 0 => {
                                self.place_explosion(Self::clamp_bounds(d.move_point(p)), ExplosionType::Dir(d, r-1));
                            },
                            ExplosionType::Centre(r) if r > 0 => {
                                for d in [Direction::N, Direction::S, Direction::E, Direction::W] {
                                    self.place_explosion(Self::clamp_bounds(d.move_point(p)), ExplosionType::Dir(d, r-1));
                                }
                            }
                            _ => {}
                        }
                        self.tiles[p.0 as usize][p.1 as usize] = Some(GameObject::Explosion(typ, t - 1));
                    } else if t < 0 {
                        if t < -Self::EXPLOSION_TIME*10 {
                            match typ {
                                ExplosionType::Box(Some(pu)) => {
                                    self.tiles[p.0 as usize][p.1 as usize] = Some(GameObject::PowerUp(pu, Self::POWERUP_TIME))
                                }
                                _ => { self.tiles[p.0 as usize][p.1 as usize] = None; continue}
                            }
                        } else {
                            self.tiles[p.0 as usize][p.1 as usize] = Some(GameObject::Explosion(typ, t - 1));
                        }
                    }
                },
                _ => continue
             }
            self.active_tiles.push(p);
        }
        self.explode_bombs = false;
    }

    fn draw<T:RenderTarget>(&self, canvas : &mut Canvas<T>) {
        canvas.set_draw_color(rgba(153,119,92,255));
        canvas.clear();
        let mut afters = Vec::new();
        for i in 0..WIDTH as i32 {
            for j in 0..HEIGHT as i32 {
                if let Some(x) = self.tiles[i as usize][j as usize] {
                    match x {
                        GameObject::PowerUp(put,f ) if (f /2) % 2 != 0 || f > 100 => {
                            match put {
                                PowerUpType::Bomb => &self.gfx.pu_bomb,
                                PowerUpType::Speed => &self.gfx.pu_speed,
                                PowerUpType::Radius => &self.gfx.pu_radius,
                                PowerUpType::OneUp => &self.gfx.pu_1up,
                                PowerUpType::Invulnerability => &self.gfx.pu_inv,
                                PowerUpType::Pass => &self.gfx.pu_pass,
                                PowerUpType::Kick => &self.gfx.pu_kick,
                                PowerUpType::Remote => &self.gfx.pu_remote,
                                PowerUpType::LevelExit if self.enemies.len() > 0 => &self.gfx.level_exit_locked,
                                PowerUpType::LevelExit  => &self.gfx.level_exit,
                            }.draw(canvas,(i*12-6,j*12-6+17))
                        },
                        GameObject::Block => self.gfx.block.draw(canvas,(i*12-6,j*12-6+17)),
                        GameObject::Box(_) => self.gfx.push_box.draw(canvas,(i*12-6,j*12-6+17)),
                        GameObject::Bomb(f,_) if f < 50 && (f / 2) % 2 == 0 => self.gfx.red_bomb.draw(canvas,(i*12+1,j*12+1+17)),
                        GameObject::Bomb(f,_) if f < 50 => self.gfx.red_bomb_hl.draw(canvas,(i*12+1,j*12+1+17)),
                        GameObject::Bomb(f,_) if f >= 50 => self.gfx.bomb.draw(canvas,(i*12+1,j*12+1+17)),
                        GameObject::Explosion(ExplosionType::Centre(_),..) => afters.push(((i*12-6,j*12+17-6),true)),
                        GameObject::Explosion(ExplosionType::Dir(dir, _),..) => {
                            let next = Self::clamp_bounds(dir.move_point((i,j)));
                            let prev = Self::clamp_bounds(dir.opposite().move_point((i,j)));
                            let idx = if let Some(GameObject::Explosion(..)) = self.tiles[next.0 as usize][next.1 as usize] {
                                if let Some(GameObject::Explosion(.. )) = self.tiles[prev.0 as usize][prev.1 as usize] {
                                    match dir {
                                        Direction::N | Direction::S => 1,
                                        Direction::E | Direction::W => 2,
                                    }
                                } else {
                                    match dir {
                                        Direction::S => 4, Direction::N => 5, Direction::W => 6, Direction::E => 7
                                    }
                                }
                            } else {
                                if let Some(GameObject::Explosion(.. )) = self.tiles[prev.0 as usize][prev.1 as usize] {
                                    match dir {
                                        Direction::N => 4, Direction::S => 5, Direction::E => 6, Direction::W => 7
                                    }
                                } else { 0 }
                            };
                            self.gfx.explosion[idx].draw(canvas,(i*12-6,j*12+17-6))
                        },
                        GameObject::Explosion(ExplosionType::Box(_),..) => afters.push(((i*12-6,j*12+17-6), false)),
                        _ => {},
                    }
                }
            }
        }
        for p in afters {
            if p.1 { self.gfx.explosion[0].draw(canvas,p.0); }
            else { self.gfx.explosion[3].draw(canvas,p.0); }
        }
        for (p,_t, f,_) in &self.moving_bombs {
            if *f < 50 {
                &self.gfx.red_bomb_hl
            } else { 
                &self.gfx.bomb 
            }.draw(canvas,(p.0+1,p.1+17+1));
        }
        for (p,t, d) in &self.enemies {
            if (d / 2) % 2 == 0 { 
                match *t {
                    Enemy::Mouse(..) => &self.gfx.mouse,
                    Enemy::Cat(..) => &self.gfx.cat,
                    Enemy::Ghost(..) => &self.gfx.ghost,
                    Enemy::Cylon(..) => &self.gfx.cylon,
                    Enemy::DangerCylon(_,_,false) => &self.gfx.danger_cylon,
                    Enemy::DangerCylon(_,_,true) => &self.gfx.danger_cylon_hl,
                }.draw(canvas, (p.0+1,p.1+17+1))
            }
        }
        if (self.dying / 4) % 2 == 0 {
            if (self.iframes / 4) % 2 == 0 {
                self.gfx.player[self.facing.to_index()-1].draw(canvas,(self.player.0+1,self.player.1+17+1)); 
            } else {
                self.gfx.player_hl[self.facing.to_index()-1].draw(canvas,(self.player.0+1,self.player.1+17+1)); 
            }
        }
    }
    fn clamp_bounds(p:(i32,i32)) -> (i32,i32) {
        (p.0.max(0).min(WIDTH as i32 -1), p.1.max(0).min(HEIGHT as i32 - 1))
    }
    fn move_heuristic( p : (i32, i32), player: (i32,i32), costs: [[usize;HEIGHT as usize]; WIDTH as usize]) -> (usize,i32) {
        let (px,py) = player;
        (costs[p.0 as usize][p.1 as usize], (p.0 - px).abs() + (p.1 - py).abs())
    }
    fn random_move(&self, c : (i32,i32), mfacing: Option<Direction>, pass_soft: bool ) -> Option<Direction> {
        let mut moves : Vec<Direction> = Direction::ALL.to_vec();
        if let Some(facing) = mfacing {
            moves.push(facing);
            moves.push(facing);
            moves.push(facing);
        }
        moves = moves.iter().filter(|x| {
            let p = x.move_point(c);
            Self::clamp_bounds(p) == p && !self.dangerous(p,pass_soft)
        }).cloned().collect();
        if moves.len() == 0 {
            None
        } else {
            Some(moves[thread_rng().gen_range(0,moves.len())])
        }
    }
    fn move_for_cat(&self, c : (i32,i32),  cs: [[usize;HEIGHT as usize]; WIDTH as usize] ) -> Option<Direction> {
        let mut moves : Vec<Direction> = Direction::ALL.iter().filter(|x| {
            let p = x.move_point(c);
            Self::clamp_bounds(p) == p && !self.dangerous(p, false)
        }).cloned().collect();
        if moves.len() == 0 {
            return None;
        } else {
            let player = self.player_tile_pos();
            moves.sort_by(|d1, d2| {
                let p = d1.move_point(c);
                let q = d2.move_point(c);
                Self::move_heuristic(p,player,cs).cmp(&Self::move_heuristic(q,player,cs))
            });
            let best_heuristic = Self::move_heuristic(moves.first().unwrap().move_point(c),player,cs);
            let best_moves : Vec<Direction> = moves.into_iter().take_while(|x| Self::move_heuristic(x.move_point(c), player,cs) == best_heuristic ).collect();
            let x = thread_rng().gen_range(0,best_moves.len());
            return Some(best_moves[x]);
        }

    }
    fn dangerous(&self, pos:(i32,i32), pass_soft:bool) -> bool {
        self.obstructed(pos,pass_soft,false) || match self.tiles[pos.0 as usize][pos.1 as usize] {
            Some(GameObject::Explosion(..)) => true,
            _ => false,
        }
    }
    fn obstructed(&self, pos:(i32,i32),pass_soft:bool,pass_bomb:bool) -> bool {
        match self.tiles[pos.0 as usize][pos.1 as usize] {
            None => false,
            Some(GameObject::Block) => true,
            Some(GameObject::Box(_)) => !pass_soft,
            Some(GameObject::Bomb(..)) => !pass_bomb,
            _ => false
        }
    }

    fn is_bomb(&self, pos:(i32,i32)) -> bool {
        match self.tiles[pos.0 as usize][pos.1 as usize] {
            Some(GameObject::Bomb(..)) => true,
            _ => false
        }
    }

    fn tick(&mut self) {
        self.tick_active_tiles();
        self.enemy_movement();
        self.bomb_movement();
        if self.dying > 0 { 
            self.dying -= 1;
            if self.dying == 0 {
                if self.lives == 0 {
                    self.dead = true
                } else {
                    self.lives -= 1;
                    self.player = (12,12);
                    self.facing = Direction::S;
                    self.iframes = 100;
                }
            }
            return ;
        } else {
            self.enemy_player_collisions();
            self.player_tile_collisions();
        }
        self.tick += 1;
        let steps = if self.tick % (20 - (self.speed % 20)) == 0 {
            self.tick = 1; self.speed / 20 + 2
        } else {
            1 + (self.speed / 20)
        };
        if let Some(desired_facing) = self.pressed.last() {
            for _i in 0..steps {
                let tpos = desired_facing.move_point(self.player_tile_pos());
                if self.facing == desired_facing.opposite() {
                    self.facing = *desired_facing;
                } else if !self.obstructed(tpos, self.pass_pu, true) && match *desired_facing {
                    Direction::E | Direction::W => { self.player.1 % 12 < 4 || self.player.1 % 12 >= 8 },
                    Direction::N | Direction::S => { self.player.0 % 12 < 4 || self.player.0 % 12 >= 8 },
                } { self.facing = *desired_facing }
                match self.facing {
                    Direction::E | Direction::W => self.player.1 = self.player_tile_pos().1 * 12,
                    Direction::N | Direction::S => self.player.0 = self.player_tile_pos().0 * 12
                }
                let newpos = self.facing.move_point(self.player);
                let newtpos2 = self.facing.move_point_by(self.player, 6);
                let newtpos = ((newtpos2.0 + 6) / 12,(newtpos2.1 + 6) / 12);
                if !self.obstructed(newtpos,self.pass_pu, false) || (self.is_bomb(self.player_tile_pos()) && self.is_bomb(newtpos))  { 
                    self.player = newpos;
                } else if self.kick_pu {
                    match self.tiles[newtpos.0 as usize][newtpos.1 as usize] {
                        Some(GameObject::Bomb(f,r )) => {
                            self.moving_bombs.push(((newtpos.0*12,newtpos.1*12),self.facing,f,r));
                            self.active_tiles.retain(|x| *x != newtpos);
                            self.tiles[newtpos.0 as usize][newtpos.1 as usize] = None;
                        }
                        _ => {},
                    }
                }
            }
        }
    }
    fn bomb_movement(&mut self) {
        let bombs = self.moving_bombs.clone();
        self.moving_bombs = Vec::new();
        for (p,d,f,r) in bombs {
            let tp = Self::to_tile_pos(p);
            if p == (tp.0*12,tp.1*12) || self.stop_bombs {
                if self.stop_bombs || self.obstructed(d.move_point(tp), false, false) 
                    || self.enemies.iter().any(|t|Self::to_tile_pos(t.0) == d.move_point(tp)) {
                    match  self.tiles[tp.0 as usize][tp.1 as usize] { 
                         Some(GameObject::PowerUp(PowerUpType::LevelExit, _ )) => {},
                         _ => {self.tiles[tp.0 as usize][tp.1 as usize] = Some(GameObject::Bomb(f, r));self.active_tiles.push(tp);},
                    };
                    
                    continue;
                } else { 
                    match self.tiles[d.move_point(tp).0 as usize][d.move_point(tp).1 as usize] {
                        Some(GameObject::Explosion(..))  => {
                            self.place_explosion(tp, ExplosionType::Centre(r));
                            self.bombs += 1;
                            continue;
                        }
                        _ => {},
                    }
                }
            }
            self.moving_bombs.push((d.move_point_by(p,2),d,f,r));
        }
        self.stop_bombs = false;
    }
    fn to_tile_pos(p : (i32, i32)) -> (i32,i32) {
        ((p.0 + 6) / 12,(p.1 + 6) / 12)
    }
    fn enemy_drop_loot(&mut self, e : &Enemy, pos: (i32,i32)) {
        if !self.obstructed(pos,false, false) {
            match e {
                Enemy::DangerCylon(d, _, false ) => {
                    self.enemies.push(((pos.0*12,pos.1*12),Enemy::DangerCylon(*d, 0, true),0));
                },
                _ => {}
            }
        }
    }
    fn enemy_tile_collisions(&mut self) {
        for (p,e,d) in &mut self.enemies {
            let tp = Self::to_tile_pos(*p);
            match self.tiles[tp.0 as usize][tp.1 as usize] {
                Some(GameObject::Explosion(..)) => { if *d == 0 { 
                    self.score += e.kill_score();  *d = 52;
                } },
                _ => {},
            }
        }
    }
    fn enemy_movement(&mut self) {
        self.enemy_tick += 1;
        if self.enemy_tick > 100 { self.enemy_tick = 0 }
        let enemies = self.enemies.clone();
        self.enemies = Vec::new();
        let mut cs: [[usize; HEIGHT as usize]; WIDTH as usize] = [[0; HEIGHT as usize]; WIDTH as usize];
        let mut inited = false;
        for (mut p, mut e, mut d) in enemies {            
            if d > 0 {
                d -= 1;
                if d == 0 {
                    let tp = Self::to_tile_pos(p);
                    self.enemy_drop_loot(&e, tp);
                    continue    
                } else {
                    self.enemies.push((p,e, d));
                    continue
                }
            }
            let is_cylon = match e { Enemy::Cylon(..) => true, Enemy::DangerCylon(_,_,b) => b, _ => false };
            let is_ghost = match e { Enemy::Ghost(..) => true, _ => false };
            match &mut e {
                Enemy::Ghost(mfacing, fuel) |
                Enemy::Cylon(mfacing, fuel)=> {
                    if self.enemy_tick %  if is_cylon { 1 } else { 2 } == 0 {
                        if *fuel > 0 {
                            *fuel -= 1;
                            if let Some(facing) = mfacing {
                                p = facing.move_point(p);
                                if *fuel > 6 {
                                    let tp = facing.move_point(Self::to_tile_pos(p));
                                    if self.dangerous(tp, is_ghost) {
                                        *facing  = facing.opposite();
                                        *fuel = 12 - *fuel;
                                    }
                                }
                            }
                        }
                        if *fuel == 0 {
                            *mfacing = None;
                        }
                        if mfacing.is_none() {
                            *mfacing = self.random_move(Self::to_tile_pos(p), *mfacing, !is_cylon);
                            *fuel = 12;
                        }
                    }
                }
                Enemy::Cat(mfacing, fuel) | 
                Enemy::DangerCylon(mfacing ,fuel,_ )=> {
                    if self.enemy_tick % if is_cylon { 1 } else { 2 } == 0 {
                        if *fuel > 0 {
                            *fuel -= 1;
                            if let Some(facing) = mfacing {
                                p = facing.move_point(p);
                                if *fuel > 6 {
                                    let tp = facing.move_point(Self::to_tile_pos(p));
                                    if self.dangerous(tp, false) {
                                        *facing  = facing.opposite();
                                        *fuel = 12 - *fuel;
                                    }
                                }
                            }
                        }
                        if *fuel == 0 {
                            *mfacing = None;
                        }
                        if mfacing.is_none() {
                            if !inited { cs = self.cost_map(); inited = true }
                            *mfacing = self.move_for_cat(Self::to_tile_pos(p), cs);
                            *fuel = 12;
                        }
                    }
                },
                Enemy::Mouse(facing,desired_facing) => {
                    if self.enemy_tick % 2 != 0 { 
                        let tpos = desired_facing.move_point(Self::to_tile_pos(p));
                        if *facing == desired_facing.opposite() {
                            *facing = *desired_facing;
                        } else if !self.obstructed(tpos, false, false) && match *desired_facing {
                            Direction::E | Direction::W => p.1 % 12 < 4 || p.1 % 12 >= 8,
                            Direction::N | Direction::S => p.0 % 12 < 4 || p.0 % 12 >= 8,
                        } { *facing = *desired_facing }
                        match *facing {
                            Direction::E | Direction::W => p.1 = Self::to_tile_pos(p).1 * 12,
                            Direction::N | Direction::S => p.0 = Self::to_tile_pos(p).0 * 12
                        }
                        let newpos = facing.move_point(p);
                        let newtpos2 = facing.move_point_by(p, 6);
                        let newtpos = Self::to_tile_pos(newtpos2);
                        if !self.dangerous(newtpos, false)  { 
                            p = newpos;
                        } else {
                            *desired_facing = Direction::random_dir()
                        }
                    }
                }
            }
            self.enemies.push((p,e, d));
        }
        self.enemy_tile_collisions();
    }

    fn enemy_player_collisions(&mut self) {
        if self.iframes > 0 {
            self.iframes -= 1;
        }
        if self.iframes == 0 {
            self.invincibility_pu = false;
            for (p,_,_d) in &self.enemies {
                if p.0 +2 + 8 >= self.player.0+2 &&
                    p.0 +2 <= self.player.0 + 2 + 8 &&
                    p.1 + 2 + 8 >= self.player.1 + 2 &&
                    p.1 + 2 <= self.player.1 + 2 + 8 {
                    self.dying = 100;
                }
            }
        }
    }

    fn new_game(&mut self) {
        self.lives = 3;
        self.score = 0;
        self.set_up_level(1);
    }

    fn direction_down(&mut self, d: Direction) {
        self.pressed.push(d);
    }

    fn direction_up(&mut self, d: Direction) {
        self.pressed.retain(|x| *x != d);        
    }
}

enum Splash {
    Level,
    Loss,
    Pause,
}


fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.text_input().start();
    let window = video_subsystem.window("grenadier", WIDTH*12, HEIGHT*12 + 16 + 17 + 17)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    let mut splash = Some(Splash::Level);
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(WIDTH*12, HEIGHT*12 + 16 + 17+17).unwrap();
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut game = Game::new(&texture_creator);
    let mut level = 1;
    game.set_up_level(level);
    let mut status_level = Graphic::blank(8,1).textured(&texture_creator);
    let mut status_score = Graphic::blank(7,1).textured(&texture_creator);
    let mut status_lives = Graphic::blank(2,1).textured(&texture_creator);
    let mut status_bombs = Graphic::blank(2,1).textured(&texture_creator);
    let mut status_speed = Graphic::blank(2,1).textured(&texture_creator);
    let mut status_radius  = Graphic::blank(2,1).textured(&texture_creator);
    let mut lose = Graphic::load_from(Cursor::new(&include_bytes!("../lose")[..])).unwrap().textured(&texture_creator);
    lose.update_texture(&game.tile_set);
    let mut level_splash = Graphic::load_from(Cursor::new(&include_bytes!("../level_splash")[..])).unwrap().textured(&texture_creator);
    level_splash.update_texture(&game.tile_set);
    let mut paused_splash = Graphic::load_from(Cursor::new(&include_bytes!("../paused_splash")[..])).unwrap().textured(&texture_creator);
    paused_splash.update_texture(&game.tile_set);
    let mut rate_limiter = FPSManager::new();    
    rate_limiter.set_framerate(40).unwrap();
    let mut micro_mode = false;    
    let mut menu = MenuBar::new(WIDTH*12)
                    .add(Menu::new("GAME",120,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Restart",15, Keycode::N,&texture_creator,&game.tile_set))
                            .add(MenuItem::new("Skip Level",357, Keycode::F6,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(104, &texture_creator,&game.tile_set))
                            .add(MenuItem::new("Pause", 17, Keycode::P,&texture_creator,&game.tile_set))
                            .add(MenuItem::separator(104, &texture_creator,&game.tile_set))
                            .add(MenuItem::new("Quit", 363, Keycode::F12,&texture_creator,&game.tile_set)))
                    .add(Menu::new("VIEW",112,&texture_creator,&game.tile_set)
                            .add(MenuItem::new("Micro-mode",360, Keycode::F9, &texture_creator, &game.tile_set)));


    loop {
        if splash.is_none() { 
            if game.won_level {
                level += 1;
                game.set_up_level(level);
                splash = Some(Splash::Level);
            }
            if game.dead {
                level = 1;
                game.new_game();
                splash = Some(Splash::Loss);
            }
        }
        if splash.is_none() {
            game.tick();
        }

        game.draw(&mut canvas);
        canvas.set_draw_color(BLACK);
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*12 + 17,WIDTH*12,40)).unwrap();
        canvas.set_draw_color(rgba(85,87,83,255));
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*12 + 18,WIDTH*12,16)).unwrap();
        canvas.set_draw_color(rgba(176,179,172,255));
        canvas.fill_rect(Rect::new(0,HEIGHT as i32*12 + 18 + 17,WIDTH*12,16)).unwrap();
        status_level.draw_rect(0,0,8,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_level.draw_text(&("Level ".to_owned() + &level.to_string()),&game.tile_set,0,0,BLACK,TRANSPARENT);
        status_level.update_texture(&game.tile_set);
        game.gfx.player[3].draw(&mut canvas, (8, HEIGHT as i32 * 12 + 17 +5));
        game.gfx.red_bomb_hl.draw(&mut canvas, (WIDTH as i32 * 12 - 16, HEIGHT as i32 * 12 + 17 + 4));
        status_lives.draw_rect(0,0,2,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_lives.draw_text(&game.lives.to_string(),&game.tile_set,0,0,WHITE,TRANSPARENT);
        status_lives.update_texture(&game.tile_set);
        status_bombs.draw_rect(0,0,2,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_bombs.draw_text(&game.bombs.to_string(),&game.tile_set,0,0,WHITE,TRANSPARENT); 
        status_bombs.update_texture(&game.tile_set);
        status_speed.draw_rect(0,0,2,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_speed.draw_text(&(game.speed/10 + 1).to_string(),&game.tile_set,0,0,WHITE,TRANSPARENT); 
        status_speed.update_texture(&game.tile_set);
        status_radius.draw_rect(0,0,2,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_radius.draw_text(&game.bomb_radius.to_string(),&game.tile_set,0,0,WHITE,TRANSPARENT); 
        status_radius.update_texture(&game.tile_set);
        status_score.draw_rect(0,0,2,1,Tile{index:0,fg:TRANSPARENT,bg:TRANSPARENT});
        status_score.draw_text(&game.score.to_string(),&game.tile_set,0,0,BLACK,TRANSPARENT);
        status_score.update_texture(&game.tile_set);
        game.gfx.speed_icon.draw(&mut canvas, (WIDTH as i32 * 12 - 16 - 16 -4 - 16, HEIGHT as i32 * 12 + 17 + 4));
        status_speed.draw(&mut canvas, (WIDTH as i32 * 12 - 16 - 16 - 4 - 16 - ((game.speed / 10 + 1).to_string().len() as i32) * 8 - 4, HEIGHT as i32 * 12 + 23));
        game.gfx.radius_icon.draw(&mut canvas, (WIDTH as i32 * 12 - 16 - 16 - 4 - 16 - 16 - 4 - 16, HEIGHT as i32 * 12 + 17 + 4 ) );
        status_radius.draw(&mut canvas, (WIDTH as i32 * 12 - 16 - 16 - 4 - 16 - 16 - 4 - 16 - 4 - (game.bomb_radius.to_string().len() as i32 * 8), HEIGHT as i32 * 12 + 23));
        status_level.draw(&mut canvas, (8, HEIGHT as i32 * 12 + 17+ 22 ));
        status_lives.draw(&mut canvas, (22, HEIGHT as i32 * 12 + 23 ));
        status_bombs.draw(&mut canvas, (WIDTH as i32 * 12 - 16 - game.bombs.to_string().len() as i32 * 8 - 4, HEIGHT as i32 * 12 + 23 )); 
        status_score.draw(&mut canvas, (WIDTH as i32 * 12 - 8 - game.score.to_string().len() as i32 * 8, HEIGHT as i32 * 12 + 17+22 ));
        let mut pu_x = 22 + 24;
        if game.invincibility_pu {
            game.gfx.pu_inv.draw(&mut canvas, (pu_x,HEIGHT as i32 * 12 + 14 ));
            pu_x += 20;
        }
        if game.pass_pu {
            game.gfx.pu_pass.draw(&mut canvas, (pu_x,HEIGHT as i32 * 12 + 14 ));
            pu_x += 20;
        }
        if game.kick_pu {
            game.gfx.pu_kick.draw(&mut canvas, (pu_x,HEIGHT as i32 * 12 + 14 ));
            pu_x += 20;
        }
        if game.remote_pu {
            game.gfx.pu_remote.draw(&mut canvas, (pu_x,HEIGHT as i32 * 12 + 14 ));
            //pu_x += 20;
        }
        if let Some(s) = &splash {
            match *s {
                Splash::Level => { 
                    level_splash.draw(&mut canvas, (WIDTH as i32 * 6 - (11*4), HEIGHT as i32 * 6 - 16 + 17 ));
                    status_level.draw(&mut canvas, (WIDTH as i32 * 6 - (8 * 4) + if level < 10 { 4} else {0}, HEIGHT as i32 * 6 - 4 + 17));
                },
                Splash::Loss => lose.draw(&mut canvas, (WIDTH as i32 * 6 - (21*4), HEIGHT as i32 * 6 - (21*4) + 17 )),
                Splash::Pause => paused_splash.draw(&mut canvas, (WIDTH as i32 * 6 - (11*4), HEIGHT as i32 * 6 - 20 + 17 )),
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
                Event::KeyDown { keycode: Some(Keycode::F9), ..} => {
                    if micro_mode {
                        micro_mode = false;
                        canvas.window_mut().set_size(WIDTH*12, HEIGHT*12+16+17 +17).unwrap_or_default();
                    } else {
                        micro_mode = true;
                        canvas.window_mut().set_size(WIDTH*6, (HEIGHT*12+16+17+17)/2).unwrap_or_default();
                    }
                },
                Event::KeyDown {..} if splash.is_some() => {
                    splash = None  
                }
                Event::MouseButtonUp {..} if splash.is_some() => {
                    splash = None
                }
                Event::KeyDown { keycode: Some(Keycode::N), .. } => {
                    level = 1;
                    game.new_game();
                    splash = Some(Splash::Level);
                },
                Event::KeyDown { keycode: Some(Keycode::Z), ..} |
                Event::KeyDown { keycode: Some(Keycode::E), ..} 
                 => game.place_bomb_at_player(),
                Event::KeyDown { keycode: Some(Keycode::X), ..} |
                Event::KeyDown { keycode: Some(Keycode::Q), ..} => {  
                    if game.moving_bombs.len() > 0 {
                        game.stop_bombs = true
                    } else if game.remote_pu {
                        game.explode_bombs = true
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Up), ..} |
                Event::KeyDown { keycode: Some(Keycode::W), ..}
                  => game.direction_down(Direction::N),
                Event::KeyDown { keycode: Some(Keycode::Down), ..} |
                Event::KeyDown { keycode: Some(Keycode::S), ..}
                  => game.direction_down(Direction::S),
                Event::KeyDown { keycode: Some(Keycode::Left), ..} |
                Event::KeyDown { keycode: Some(Keycode::A), ..}
                  => game.direction_down(Direction::W),
                Event::KeyDown { keycode: Some(Keycode::Right), ..} |
                Event::KeyDown { keycode: Some(Keycode::D), ..}
                  => game.direction_down(Direction::E),
                Event::KeyUp { keycode: Some(Keycode::Up), ..} |
                Event::KeyUp { keycode: Some(Keycode::W), ..}
                  => game.direction_up(Direction::N),
                Event::KeyUp { keycode: Some(Keycode::Down), ..} |
                Event::KeyUp { keycode: Some(Keycode::S), ..}
                  => game.direction_up(Direction::S),
                Event::KeyUp { keycode: Some(Keycode::Left), ..} |
                Event::KeyUp { keycode: Some(Keycode::A), ..}
                  => game.direction_up(Direction::W),
                Event::KeyUp { keycode: Some(Keycode::Right), ..} |
                Event::KeyUp { keycode: Some(Keycode::D), ..}
                  => game.direction_up(Direction::E),
                Event::KeyDown { keycode: Some(Keycode::P), ..}
                  => splash = Some(Splash::Pause),
                Event::KeyDown { keycode: Some(Keycode::F6), ..}
                  => game.won_level = true,
                _ => {},
            }
        }
        rate_limiter.delay();
    }
}

