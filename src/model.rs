use rand::prelude::*;
use std::{num, time};

pub const FPS: i32 = 30;
pub const TUBE_COUNT: usize = 10;
pub const MAX_PORTION: usize = 4;
pub const COLOR_COUNT: usize = TUBE_COUNT - 2;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Command {
    None,
}

pub struct Game {
    pub rng: StdRng,
    pub is_over: bool,
    pub requested_sounds: Vec<&'static str>,
    pub tubes: Vec<Vec<i32>>,
}

impl Game {
    pub fn new() -> Self {
        let now = time::SystemTime::now();
        let timestamp = now
            .duration_since(time::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();
        let rng = StdRng::seed_from_u64(timestamp);
        println!("random seed = {}", timestamp);
        // let rng = StdRng::seed_from_u64(0);

        let game = Game {
            rng: rng,
            is_over: false,
            requested_sounds: Vec::new(),
            tubes: Vec::new(),
        };

        game
    }

    pub fn init(&mut self) {
        let mut portions: Vec<i32> = vec![0; MAX_PORTION * COLOR_COUNT];
        let mut index = 0;
        for i in 0..COLOR_COUNT {
            for _ in 0..MAX_PORTION {
                portions[index] = i as i32 + 1;
                index += 1;
            }
        }
        portions.shuffle(&mut self.rng);

        self.tubes = Vec::new();

        for i in 0..COLOR_COUNT {
            let tube: Vec<i32> = portions[(i * MAX_PORTION)..((i + 1) * MAX_PORTION)].to_vec();
            self.tubes.push(tube);
        }
        for i in 0..(TUBE_COUNT - COLOR_COUNT) {
            self.tubes.push(Vec::new());
        }
        println!("tubes: {:?}", self.tubes);
    }

    pub fn update(&mut self, command: Command) {
        if self.is_over {
            return;
        }

        match command {
            Command::None => {}
        }
    }
}
