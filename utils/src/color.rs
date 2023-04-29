use sdl2::pixels::Color;

pub const fn rgba(r:u8,g:u8,b:u8,a:u8) -> Color {
    Color { r:r, g:g, b:b, a:a}
}
pub const BLACK : Color = rgba(0,0,0,255);
pub const TRANSPARENT : Color = rgba(0,0,0,0);
pub const NEUTRAL_GRAY : Color = rgba(186,189,182,255);
pub const DARKER_GRAY : Color = rgba(156,159,152,255);
pub const WHITE : Color = rgba(255,255,255,255);
pub const BLUE : Color = rgba(32,74,135,255);
pub const PALE_BLUE : Color = rgba(114,159,207,255);
pub const BRIGHT_GREEN : Color = rgba(136,226,52,255);
pub const ORANGE : Color = rgba(245,121,0,255);
pub const PALE_ORANGE : Color = rgba(252,175,62,255);
pub const PALE_PURPLE : Color = rgba(255,202,247,255);
pub const GREEN : Color = rgba(78,108,6,255);
pub const PURPLE : Color = rgba(92,52,102,255);
pub const TEAL : Color = rgba(27,128,120,255);
pub const DARK_TEAL : Color = rgba(17,108,100,255);
pub const CRIMSON : Color = rgba(141,0,0,255);
pub const DARK_RED : Color = rgba(164,0,0,255);
pub const AMBER : Color = rgba(159,76,0,255);
pub const BROWN : Color = rgba(119,90,5,255);
pub const PALE_BROWN : Color = rgba(233,185,110,255);
pub const LIGHT_BROWN : Color = rgba(193,175,17,255);
pub const CHARCOAL : Color = rgba(46,52,54,255);
pub const DARK_CHARCOAL : Color = rgba(16,22,24,255);
pub const YELLOW : Color = rgba(252,233,39,255);
pub const DARK_YELLOW : Color = rgba(196,160,0,255);