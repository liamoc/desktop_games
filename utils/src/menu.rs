use std::ops::{Index, IndexMut};
use tesserae::{Graphic,TileSet,Tile};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::render::{Texture, RenderTarget, TextureCreator};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::render::Canvas;
use sdl2::rect::Rect;
use sdl2::{EventSubsystem};

pub struct MenuItem<'r> {
    graphic: Graphic<Texture<'r>>,
    hl_graphic: Graphic<Texture<'r>>,
    keycode: Option<Keycode>
}
impl <'r>MenuItem<'r> {
    pub fn new<T>(title: &str, keycode: Keycode, texture_creator: &'r TextureCreator<T>, tile_set : &TileSet) -> Self {
        let mut g = Graphic::blank(title.len() as u32,1).textured(texture_creator);
        g.draw_text(title, tile_set, 0, 0, Color::RGB(0,0,0), Color::RGBA(0,0,0,0));
        g.update_texture(tile_set);
        let mut hg = Graphic::blank(title.len() as u32,1).textured(texture_creator);
        hg.draw_text(title, tile_set, 0, 0, Color::RGB(255,255,255), Color::RGBA(0,0,0,0));
        hg.update_texture(tile_set);
        MenuItem {
            graphic: g,
            hl_graphic: hg,
            keycode: Some(keycode)
        }
    }
    pub fn separator<T>(width:u32,  texture_creator: &'r TextureCreator<T>, tile_set : &TileSet) -> Self {
        let mut g = Graphic::solid(width/8,1,Tile{index:224, fg:Color::RGB(211,215,207),bg:Color::RGBA(0,0,0,0)}).textured(texture_creator);
        g.update_texture(tile_set);
        let mut hg = Graphic::solid(width/8,1,Tile{index:224, fg:Color::RGB(211,215,207),bg:Color::RGBA(0,0,0,0)}).textured(texture_creator);
        hg.update_texture(tile_set);
        MenuItem {
            graphic: g,
            hl_graphic: hg,
            keycode: None
        }
    }
}

pub struct Menu<'r> {
    graphic: Graphic<Texture<'r>>,
    hl_graphic: Graphic<Texture<'r>>,
    items: Vec<MenuItem<'r>>,
    current: Option<usize>,
    width: u32,
}
impl <'r> Index<usize> for Menu<'r> {
    type Output = MenuItem<'r>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}
impl <'r> IndexMut<usize> for Menu<'r> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}
impl <'r> Menu<'r> {
    pub fn new<T>(title: &str, width: u32, texture_creator: &'r TextureCreator<T>, tile_set : &TileSet) -> Self {
        let mut g = Graphic::blank(title.len() as u32 *8,1).textured(texture_creator);
        g.draw_text(title, tile_set, 0, 0, Color::RGB(0,0,0), Color::RGBA(0,0,0,0));
        g.update_texture(tile_set);
        let mut hg = Graphic::blank(title.len() as u32 *8,1).textured(texture_creator);
        hg.draw_text(title, tile_set, 0, 0, Color::RGB(255,255,255), Color::RGBA(0,0,0,0));
        hg.update_texture(tile_set);
        Menu {
            graphic: g, 
            hl_graphic: hg,
            items: Vec::new(), 
            current: None,
            width: width
        }
    }
    pub fn add(mut self, item: MenuItem<'r>) -> Self {
        self.items.push(item);
        self
    }
}

pub struct MenuBar<'r> {
    menus: Vec<Menu<'r>>,
    current: Option<usize>,
    drawable_width: u32,
    enabled: bool,
}
impl <'r> Index<usize> for MenuBar<'r> {
    type Output = Menu<'r>;
    fn index(&self, index: usize) -> &Menu<'r> {
        &self.menus[index]
    }
}
impl <'r> IndexMut<usize> for MenuBar<'r> {
    fn index_mut(&mut self, index: usize) -> &mut Menu<'r> {
        &mut self.menus[index]
    }
}
impl <'r> MenuBar<'r> {
    pub fn new(drawable_width:u32) -> Self {
        MenuBar {
            enabled:true,
            menus: Vec::new(),
            current: None,
            drawable_width: drawable_width,
        }
    }
    pub fn add(mut self, menu: Menu<'r>) -> Self {
        self.menus.push(menu);
        self
    }
    pub fn draw<T:RenderTarget>(&self, canvas : &mut Canvas<T>) {
        canvas.set_draw_color(Color::RGB(0,0,0));
        canvas.fill_rect(Rect::new(0,0,self.drawable_width,17)).unwrap();
        canvas.set_draw_color(Color::RGB(176,179,172));
        canvas.fill_rect(Rect::new(0,0,self.drawable_width,16)).unwrap();
        let mut x = 8;
        let y = 4;
        for i in 0..self.menus.len() {
            match self.current {
                Some(j) if i == j && self.enabled => {
                    canvas.set_draw_color(Color::RGB(32,74,135));
                    canvas.fill_rect(Rect::new(x - 8,0,self.menus[i].graphic.width()+16,16)).unwrap();
                    self.menus[i].hl_graphic.draw(canvas, (x,y));
                }, 
                _ if !self.enabled => {
                    self.menus[i].hl_graphic.draw(canvas, (x,y));
                }
                _ => {
                    self.menus[i].graphic.draw(canvas, (x,y));
                }
            }
            x += self.menus[i].graphic.width() as i32 + 16
        }
        if let Some(j) = self.current {
            let mut x = 0;
            for i in 0..j {
                x += self.menus[i].graphic.width() as i32 + 16
            }
            let h = self.menus[j].items.len() as u32 * 16;
            canvas.set_draw_color(Color::RGB(0,0,0));
            canvas.fill_rect(Rect::new(x,16,self.menus[j].width+2,h+2)).unwrap();
            canvas.set_draw_color(Color::RGB(176,179,172));
            canvas.fill_rect(Rect::new(x+1,17,self.menus[j].width,h)).unwrap();
            let mut y = 4 + 17;
            for k in 0..self.menus[j].items.len() {
                match self.menus[j].current {
                    Some(l) if k == l => {
                        if self.menus[j].items[k].keycode.is_some() {
                            canvas.set_draw_color(Color::RGB(32,74,135));
                            canvas.fill_rect(Rect::new(x+1,y-4,self.menus[j].width,16)).unwrap();
                        }
                        self.menus[j].items[k].hl_graphic.draw(canvas,(x+8,y));
                        y += 16;
                    },
                    _ => {
                        self.menus[j].items[k].graphic.draw(canvas,(x+8,y));
                        y += 16;
                    }
                }
            }
        }
    }
    fn mouse_on_bar(&mut self, x:i32) -> bool {
        let mut i = 0;
        let mut xx = x;
        while xx >= 0 {
            if i >= self.menus.len() { return false };
            xx -=  self.menus[i].graphic.width() as i32 + 16;
            if xx >= 0 { i += 1 };
        }
        if let Some(j) = self.current {
            self.menus[j].current = None;
        }
        self.current=Some(i);
        true
    } 
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    pub fn disable(&mut self) {
        self.enabled = false;
        if let Some(j) = self.current {
            if let Some(_) = self.menus[j].current {
                self.menus[j].current = None
            } 
            self.current = None;
        }
    }
    pub fn handle_event(&mut self, event: Event, event_subsystem: &mut EventSubsystem) -> bool {
        if !self.enabled  { return false }
        match event {
            Event::MouseButtonDown { x, y,.. } if y < 17 => {
                self.mouse_on_bar(x)
            },
            Event::MouseButtonUp { timestamp, window_id, ..} if self.current.is_some() => {
                if let Some(j) = self.current {
                    if let Some(k) = self.menus[j].current {
                        if self.menus[j].items[k].keycode.is_some() { 
                            let e = Event::KeyDown{
                                keycode: self.menus[j].items[k].keycode, 
                                keymod: Mod::empty(), 
                                scancode: None, 
                                repeat:false, 
                                timestamp:timestamp,
                                window_id:window_id
                            };
                            event_subsystem.push_event(e).unwrap();
                        }
                        self.menus[j].current = None
                    } 
                    self.current = None;
                    true
                } else { false }
            }
            Event::MouseMotion { x: xx , y: yy, ..} if self.current.is_some() => {
                if yy < 17 {
                    return self.mouse_on_bar(xx);
                }
                if let Some(j) = self.current {
                    let mut x = 0;
                    for i in 0..j {
                        x += self.menus[i].graphic.width() as i32 + 16
                    }
                    let h = self.menus[j].items.len() as i32 * 16;
                    if xx >= x && xx <= x + self.menus[j].width as i32 {
                        if yy < h + 17 {
                            let i = (yy - 17) / 16;
                            self.menus[j].current = Some(i as usize);
                            true 
                        } else { 
                            self.menus[j].current = None;
                            true
                        }
                    } else {
                        self.menus[j].current = None;
                        true
                    }
                } else { false }
            },
            _ => false
        }
    }
}

