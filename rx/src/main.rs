#[macro_use]
extern crate lazy_static;
extern crate sdl2;
extern crate rand;
extern crate byteorder;
extern crate app_dirs;
extern crate tesserae;
extern crate utils;
mod imprint;
mod game;
mod drawing;
use game::{Game, Status,TickResult};
use drawing::{BaseDrawingContext};

use app_dirs::{AppDataType, app_root, AppInfo};
use sdl2::EventSubsystem;
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use utils::framerate::FPSManager;
use sdl2::render::{BlendMode,Canvas};
use sdl2::EventPump;
use utils::menu::{*};
const FRAMERATE: u32 = 20;

pub fn game_loop(
    game: &mut Game,
    ctx: &mut BaseDrawingContext,
    canvas: &mut Canvas<Window>,
    event_pump: &mut EventPump,
    menu: &mut MenuBar,
    event_subsystem: &mut EventSubsystem
) { 
    let mut rate_limiter = FPSManager::new();
    let mut micro_mode = false;
    rate_limiter.set_framerate(FRAMERATE).unwrap();
    loop {
        match game.status { Status::Menu(_) => menu.enable(), _ => menu.disable() };
        for event in event_pump.poll_iter() {
            let input = game.input_state();
            let h = menu.handle_event(event.clone(), event_subsystem);
            match event {
                _ if h => {},
                Event::Quit { .. } => {
                    return;
                }
                Event::KeyDown { keycode: Some(Keycode::F9), ..} => {
                    if micro_mode {
                        micro_mode = false;
                        canvas.window_mut().set_size(184,225).unwrap_or_default();
                    } else {
                        micro_mode = true;
                        canvas.window_mut().set_size(184/2,225/2).unwrap_or_default();
                    }
                }
                Event::KeyDown { keycode: Some(Keycode::F5), ..} => {
                    game.clear_current_score();
                }
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => input.escape = true,
                Event::KeyUp { keycode: Some(Keycode::Escape), .. } => input.escape = false,
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => input.escape = true,
                Event::KeyUp { keycode: Some(Keycode::Q), .. } => input.escape = false,
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => input.left = true,
                Event::KeyUp { keycode: Some(Keycode::Left), .. } => {
                    input.left = false;
                    input.skip = 0;
                }
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => input.right = true,
                Event::KeyUp { keycode: Some(Keycode::Right), .. } => {
                    input.right = false;
                    input.skip = 0;
                }
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => input.down = true,
                Event::KeyUp { keycode: Some(Keycode::Down), .. } => input.down = false,
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => input.up = true,
                Event::KeyUp { keycode: Some(Keycode::Up), .. } => input.up = false,
                Event::KeyDown { keycode: Some(Keycode::Z), .. } => input.button_a = true,
                Event::KeyUp { keycode: Some(Keycode::Z), .. } => input.button_a = false,
                Event::KeyDown { keycode: Some(Keycode::X), .. } => input.button_b = true,
                Event::KeyUp { keycode: Some(Keycode::X), .. } => input.button_b = false,
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => input.drop = true,
                Event::KeyUp { keycode: Some(Keycode::Space), .. } => input.drop = false,
                _ => {}
            }
        }
        match game.tick() {
            TickResult::Continue => {}
            _ => return,
        }
        ctx.draw_game(canvas, &game).unwrap();
        menu.draw(canvas);
        canvas.present();
        rate_limiter.delay();
    }
}



const APP_INFO: AppInfo = AppInfo {
    name: "Rx",
    author: "Liam O'Connor",
};

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut path = app_root(AppDataType::UserData, &APP_INFO).unwrap();
    let window = video_subsystem
        .window("rx", 184, 225)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();
    path.push("rx");

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut event_subsystem = sdl_context.event().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(184,225).unwrap();
    canvas.set_blend_mode(BlendMode::Blend);
    let texture_creator = canvas.texture_creator();
    let mut game = Game::new(&path).unwrap();
    let mut ctx = BaseDrawingContext::new(&texture_creator);

    let mut menu_bar = MenuBar::new(184)
                        .add(Menu::new("GAME",142, &texture_creator, &ctx.tile_set)
                             .add(MenuItem::new("Next Level",65,Keycode::Up,&texture_creator,&ctx.tile_set))
                             .add(MenuItem::new("Previous Level",67,Keycode::Down,&texture_creator,&ctx.tile_set))
                             .add(MenuItem::separator(128,&texture_creator,&ctx.tile_set))
                             .add(MenuItem::new("Faster",64,Keycode::Right,&texture_creator,&ctx.tile_set))
                             .add(MenuItem::new("Slower",66,Keycode::Left,&texture_creator,&ctx.tile_set))
                             .add(MenuItem::separator(128,&texture_creator,&ctx.tile_set))
                             .add(MenuItem::new("Start",364,Keycode::Space,&texture_creator,&ctx.tile_set))
                             .add(MenuItem::new("Clear Score",356, Keycode::F5, &texture_creator, &ctx.tile_set))
                             .add(MenuItem::new("Quit",18,Keycode::Q,&texture_creator,&ctx.tile_set)))
                    .add(Menu::new("VIEW",112,&texture_creator,&ctx.tile_set)
                            .add(MenuItem::new("Micro-mode",360, Keycode::F9, &texture_creator, &ctx.tile_set)));
    game_loop(&mut game, &mut ctx, &mut canvas, &mut event_pump, &mut menu_bar, &mut event_subsystem)
}
