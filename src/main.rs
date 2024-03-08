use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mixer;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, SystemTime};
mod model;
use crate::model::*;

pub const TITLE: &str = "rust-water-sort";
pub const SCREEN_WIDTH: i32 = 400;
pub const SCREEN_HEIGHT: i32 = 460;
const TUBE_PER_ROW: u32 = 5;
const PORTION_WIDTH: u32 = 40;
const PORTION_HEIGHT: u32 = 36;

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
    fonts: HashMap<String, sdl2::ttf::Font<'a, 'a>>,
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

    init_mixer();

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_blend_mode(BlendMode::Blend);

    let texture_creator = canvas.texture_creator();
    let mut resources = load_resources(&texture_creator, &mut canvas, &ttf_context);

    let mut event_pump = sdl_context.event_pump()?;

    let mut game = Game::new();

    println!("Select tube with mouse");
    println!("Keys:");
    println!("  Space   : Restart");

    let mut clickable_rects: Vec<(usize, Rect)> = Vec::new();

    for i in 0..TUBE_COUNT {
        let rect = get_rect(i);
        clickable_rects.push((i, rect));
    }

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
                        }
                        _ => {}
                    };
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    if mouse_btn == MouseButton::Left {
                        for (index, rect) in &clickable_rects {
                            if rect.contains_point(Point::new(x, y)) {
                                command = Command::Select(*index);
                                break;
                            }
                        }
                    }
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
    ttf_context: &'a Sdl2TtfContext,
) -> Resources<'a> {
    let mut resources = Resources {
        images: HashMap::new(),
        chunks: HashMap::new(),
        fonts: HashMap::new(),
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

    load_font(
        &mut resources,
        &ttf_context,
        "./resources/font/boxfont2.ttf",
        26,
        "boxfont",
    );

    resources
}

fn render(
    canvas: &mut Canvas<Window>,
    game: &Game,
    resources: &mut Resources,
) -> Result<(), String> {
    let clear_color = Color::BLACK;
    let font = resources.fonts.get_mut("boxfont").unwrap();

    canvas.set_draw_color(clear_color);
    canvas.clear();

    for i in 0..TUBE_COUNT {
        let rect = get_rect(i);
        if game.from_tube == Some(i) {
            canvas.set_draw_color(Color::RGB(255, 255, 0));
            canvas.draw_rect(rect)?;
            canvas.draw_rect(Rect::new(
                rect.x - 1,
                rect.y - 1,
                rect.width() + 2,
                rect.height() + 2,
            ))?;
        } else {
            canvas.set_draw_color(Color::WHITE);
            canvas.draw_rect(rect)?;
        }

        let tube = &game.tubes[i];
        for (j, portion) in tube.iter().enumerate() {
            let color = get_color(*portion);
            canvas.set_draw_color(color);
            let x = rect.x + 1;
            let y: u32 = rect.y as u32 + 14 + (MAX_PORTION as u32 - j as u32 - 1) * PORTION_HEIGHT;
            canvas.fill_rect(Rect::new(x as i32, y as i32, PORTION_WIDTH, PORTION_HEIGHT))?;
        }

        if game.state == GameState::Transfering {
            if Some(i) == game.from_tube {
                let color = get_color(game.transfering_color);
                // FIXME: アニメーションはやっつけ
                // 移動中のportionの全体を描く
                let j = game.tubes[i].len() - 1 + game.transferred_count as usize;
                println!("{} {}", game.tubes[i].len(), game.transferred_count);
                let x = rect.x + 1;
                let y: i32 =
                    rect.y + 14 + (MAX_PORTION as i32 - j as i32 - 1) * PORTION_HEIGHT as i32;
                canvas.set_draw_color(color);
                canvas.fill_rect(Rect::new(
                    x as i32,
                    y as i32,
                    PORTION_WIDTH,
                    PORTION_HEIGHT * game.transferred_count as u32,
                ))?;
                // 移動中のportionの移動済みの部分を黒で塗りつぶす
                canvas.set_draw_color(clear_color);
                canvas.fill_rect(Rect::new(
                    x as i32,
                    y as i32,
                    PORTION_WIDTH,
                    (PORTION_HEIGHT as f32
                        * game.transferred_count as f32
                        * ((TRANSFERING_WAIT + 1 - game.transfering_wait) as f32
                            / TRANSFERING_WAIT as f32)) as u32,
                ))?;
            }
        }
    }

    if game.is_clear {
        render_font(
            canvas,
            font,
            "Congraturations!".to_string(),
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT / 2 - 20,
            match (game.frame / 3) % 6 {
                0 => Color::RGB(255, 0, 0),
                1 => Color::RGB(0, 255, 0),
                2 => Color::RGB(0, 0, 255),
                3 => Color::RGB(255, 255, 0),
                4 => Color::RGB(0, 255, 255),
                5 => Color::RGB(255, 0, 255),
                _ => panic!(),
            },
            true,
        );
    }

    canvas.present();

    Ok(())
}

fn get_rect(index: usize) -> Rect {
    let column: u32 = index as u32 % TUBE_PER_ROW;
    let row: u32 = index as u32 / TUBE_PER_ROW;
    let width = PORTION_WIDTH + 2;
    let height = PORTION_HEIGHT * MAX_PORTION as u32 + 15;
    let x = 40 + 70 * column;
    let y = 40 + 210 * row;
    Rect::new(x as i32, y as i32, width, height)
}

fn get_color(color: i32) -> Color {
    match color {
        1 => Color::RGB(192, 0, 0),
        2 => Color::RGB(0, 192, 0),
        3 => Color::RGB(0, 0, 192),
        4 => Color::RGB(192, 192, 0),
        5 => Color::RGB(0, 192, 192),
        6 => Color::RGB(192, 0, 192),
        7 => Color::RGB(192, 192, 192),
        8 => Color::RGB(255, 96, 0),
        _ => panic!(),
    }
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

fn render_font(
    canvas: &mut Canvas<Window>,
    font: &sdl2::ttf::Font,
    text: String,
    x: i32,
    y: i32,
    color: Color,
    center: bool,
) {
    let texture_creator = canvas.texture_creator();

    let surface = font.render(&text).blended(color).unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();
    let x: i32 = if center {
        x - texture.query().width as i32 / 2
    } else {
        x
    };
    canvas
        .copy(
            &texture,
            None,
            Rect::new(x, y, texture.query().width, texture.query().height),
        )
        .unwrap();
}

fn load_font<'a>(
    resources: &mut Resources<'a>,
    ttf_context: &'a Sdl2TtfContext,
    path_str: &str,
    point_size: u16,
    key: &str,
) {
    let font = ttf_context
        .load_font(path_str, point_size)
        .expect(&format!("cannot load font: {}", path_str));
    resources.fonts.insert(key.to_string(), font);
}
