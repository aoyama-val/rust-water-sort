use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mixer;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::sys::KeyCode;
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, SystemTime};
mod model;
use crate::model::*;

pub const TITLE: &str = "rust-water-sort";
pub const SCREEN_WIDTH: i32 = 450;
pub const SCREEN_HEIGHT: i32 = 460;
pub const CARD_W: i32 = 124;
pub const CARD_H: i32 = 176;

struct Image<'a> {
    texture: Texture<'a>,
    #[allow(dead_code)]
    w: u32,
    h: u32,
}

impl<'a> Image<'a> {
    fn new(texture: Texture<'a>) -> Self {
        let q = texture.query();
        let image = Image {
            texture,
            w: q.width,
            h: q.height,
        };
        image
    }
}

struct Resources<'a> {
    images: HashMap<String, Image<'a>>,
    chunks: HashMap<String, sdl2::mixer::Chunk>,
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window(TITLE, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    sdl_context.mouse().show_cursor(false);

    init_mixer();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_blend_mode(BlendMode::Blend);

    let texture_creator = canvas.texture_creator();
    let mut resources = load_resources(&texture_creator, &mut canvas);

    let mut event_pump = sdl_context.event_pump()?;

    let mut game = Game::new();
    game.init();

    println!("Keys:");
    println!("  Space   : Restart");

    'running: loop {
        let started = SystemTime::now();

        let mut command = Command::None;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    if code == Keycode::Escape {
                        break 'running;
                    }
                    match code {
                        Keycode::Space => {
                            game = Game::new();
                            game.init();
                        }
                        _ => {}
                    };
                }
                _ => {}
            }
        }
        game.update(command);
        render(&mut canvas, &game, &mut resources)?;

        play_sounds(&mut game, &resources);

        let finished = SystemTime::now();
        let elapsed = finished.duration_since(started).unwrap();
        let frame_duration = Duration::new(0, 1_000_000_000u32 / model::FPS as u32);
        if elapsed < frame_duration {
            ::std::thread::sleep(frame_duration - elapsed)
        }
    }

    Ok(())
}

fn init_mixer() {
    let chunk_size = 1_024;
    mixer::open_audio(
        mixer::DEFAULT_FREQUENCY,
        mixer::DEFAULT_FORMAT,
        mixer::DEFAULT_CHANNELS,
        chunk_size,
    )
    .expect("cannot open audio");
    let _mixer_context = mixer::init(mixer::InitFlag::MP3).expect("cannot init mixer");
}

fn load_resources<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    #[allow(unused_variables)] canvas: &mut Canvas<Window>,
) -> Resources<'a> {
    let mut resources = Resources {
        images: HashMap::new(),
        chunks: HashMap::new(),
    };

    let entries = fs::read_dir("resources/image").unwrap();
    for entry in entries {
        let path = entry.unwrap().path();
        let path_str = path.to_str().unwrap();
        if path_str.ends_with(".bmp") {
            let temp_surface = sdl2::surface::Surface::load_bmp(&path).unwrap();
            let texture = texture_creator
                .create_texture_from_surface(&temp_surface)
                .expect(&format!("cannot load image: {}", path_str));

            let basename = path.file_name().unwrap().to_str().unwrap();
            let image = Image::new(texture);
            resources.images.insert(basename.to_string(), image);
        }
    }

    let entries = fs::read_dir("./resources/sound").unwrap();
    for entry in entries {
        let path = entry.unwrap().path();
        let path_str = path.to_str().unwrap();
        if path_str.ends_with(".wav") {
            let chunk = mixer::Chunk::from_file(path_str)
                .expect(&format!("cannot load sound: {}", path_str));
            let basename = path.file_name().unwrap().to_str().unwrap();
            resources.chunks.insert(basename.to_string(), chunk);
        }
    }

    resources
}

fn render(canvas: &mut Canvas<Window>, game: &Game, resources: &Resources) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    for i in 0..TUBE_COUNT {
        const TUBE_PER_ROW: u32 = 5;
        let column: u32 = i as u32 % TUBE_PER_ROW;
        let row: u32 = i as u32 / TUBE_PER_ROW;
        const PORTION_WIDTH: u32 = 50;
        const PORTION_HEIGHT: u32 = 36;
        let width = PORTION_WIDTH + 2;
        let height = PORTION_HEIGHT * MAX_PORTION as u32 + 15;
        let x = 40 + 80 * column;
        let y = 40 + 210 * row;
        canvas.set_draw_color(Color::WHITE);
        canvas.draw_rect(Rect::new(x as i32, y as i32, width, height))?;

        let tube = &game.tubes[i];
        for (j, portion) in tube.iter().enumerate() {
            let color = match portion {
                1 => Color::RGB(192, 0, 0),
                2 => Color::RGB(0, 192, 0),
                3 => Color::RGB(0, 0, 192),
                4 => Color::RGB(192, 192, 0),
                5 => Color::RGB(0, 192, 192),
                6 => Color::RGB(192, 0, 192),
                7 => Color::RGB(192, 192, 192),
                8 => Color::RGB(255, 192, 0),
                _ => panic!(),
            };
            canvas.set_draw_color(color);
            let x = x + 1;
            let y: u32 = y + 14 + (MAX_PORTION as u32 - j as u32 - 1) * PORTION_HEIGHT;
            canvas.fill_rect(Rect::new(x as i32, y as i32, PORTION_WIDTH, PORTION_HEIGHT))?;
        }
    }

    canvas.present();

    Ok(())
}

fn play_sounds(game: &mut Game, resources: &Resources) {
    for sound_key in &game.requested_sounds {
        let chunk = resources
            .chunks
            .get(&sound_key.to_string())
            .expect("cannot get sound");
        sdl2::mixer::Channel::all()
            .play(&chunk, 0)
            .expect("cannot play sound");
    }
    game.requested_sounds = Vec::new();
}
