extern crate tesserae;
extern crate sdl2;
pub mod menu;
pub mod color;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::render::RenderTarget;
use sdl2::render::Canvas;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use tesserae::Graphic;
use tesserae::Tile;
use tesserae::TileSet;
use self::color::{*};

pub struct OutlinedTile<'r> {
    base: Graphic<Texture<'r>>,
    shadow: Graphic<Texture<'r>>,
}
impl <'r>OutlinedTile<'r> {
    pub fn new<T>(tile: usize,fg:Color, tile_set: &TileSet, texture_creator : &'r TextureCreator<T>) -> Self {
        let mut base = Graphic::solid(1,1,Tile{index: tile, fg, bg:TRANSPARENT}).textured(texture_creator);
        base.update_texture(tile_set);
        let mut shadow = Graphic::solid(1,1,Tile{index: tile, fg:BLACK, bg:TRANSPARENT}).textured(texture_creator);
        shadow.update_texture(tile_set);
        OutlinedTile {
            base,
            shadow
        }
    }
    pub fn draw<T:RenderTarget>(&self, c : &mut Canvas<T>, p:(i32,i32)) {
        let (x,y) = (p.0 + 1, p.1 + 1);
        self.shadow.draw(c,(x-1,y-1));
        self.shadow.draw(c,(x+1,y+1));
        self.shadow.draw(c,(x-1,y+1));
        self.shadow.draw(c,(x+1,y-1));
        self.shadow.draw(c,(x-1,y));
        self.shadow.draw(c,(x+1,y));
        self.shadow.draw(c,(x,y+1));
        self.shadow.draw(c,(x,y-1));
        self.base.draw(c,(x,y));
    }
    pub fn draw_enlarged<T:RenderTarget>(&self, c : &mut Canvas<T>, p:(i32,i32)) {
        let (x,y) = (p.0 + 1, p.1 + 1);
        c.copy(&self.shadow.texture(), None, Rect::new(x-1,y-1,16,16)).unwrap();
        c.copy(&self.shadow.texture(), None, Rect::new(x+1,y-1,16,16)).unwrap();
        c.copy(&self.shadow.texture(), None, Rect::new(x-1,y+1,16,16)).unwrap();
        c.copy(&self.shadow.texture(), None, Rect::new(x+1,y+1,16,16)).unwrap();
        c.copy(&self.shadow.texture(), None, Rect::new(x,y-1,16,16)).unwrap();
        c.copy(&self.shadow.texture(), None, Rect::new(x,y+1,16,16)).unwrap();
        c.copy(&self.shadow.texture(), None, Rect::new(x-1,y,16,16)).unwrap();
        c.copy(&self.shadow.texture(), None, Rect::new(x+1,y,16,16)).unwrap();
        c.copy(&self.base.texture(), None, Rect::new(x,y,16,16)).unwrap();
    }
}
