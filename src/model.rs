use rand::prelude::*;
use std::{num, time};

pub const FPS: i32 = 30;
pub const TUBE_COUNT: usize = 10;
pub const MAX_PORTION: usize = 4;
pub const COLOR_COUNT: usize = TUBE_COUNT - 2;
pub const EMPTY: i32 = 0;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Command {
    None,
    Select(usize),
}

pub struct Game {
    pub rng: StdRng,
    pub frame: i32,
    pub is_clear: bool,
    pub requested_sounds: Vec<&'static str>,
    pub tubes: Vec<Vec<i32>>,
    pub from_tube: Option<usize>,
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
            frame: -1,
            is_clear: false,
            requested_sounds: Vec::new(),
            tubes: Vec::new(),
            from_tube: None,
        };

        game
    }

    pub fn init(&mut self) {
        let mut portions: Vec<i32> = vec![EMPTY; MAX_PORTION * COLOR_COUNT];
        let mut index = 0;
        for i in 0..COLOR_COUNT {
            for _ in 0..MAX_PORTION {
                portions[index] = i as i32 + 1;
                index += 1;
            }
        }
        portions.shuffle(&mut self.rng);

        self.frame = -1;
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
        self.frame += 1;

        if self.is_clear {
            return;
        }

        match command {
            Command::None => {}
            Command::Select(index) => {
                if self.from_tube == None {
                    if self.transferrable_from(index) {
                        self.from_tube = Some(index);
                        println!("from: {index}");
                    }
                } else {
                    if self.from_tube == Some(index) {
                        self.from_tube = None;
                    } else if self.transferrable_to(index) {
                        self.transfer(index);
                        self.check_clear();
                    }
                }
            }
        }
    }

    pub fn transferrable_from(&self, index: usize) -> bool {
        self.tubes[index].len() != 0
    }

    pub fn transferrable_to(&self, index: usize) -> bool {
        self.tubes[index].len() == 0
            || (self.tubes[index].len() != MAX_PORTION
                && self.tubes[index].last() == self.tubes[self.from_tube.unwrap()].last())
    }

    pub fn transfer(&mut self, index: usize) {
        println!("transfer {} -> {}", self.from_tube.unwrap(), index);
        let from_tube = self.from_tube.unwrap();
        let move_color = *self.tubes[from_tube].last().unwrap();
        while self.tubes[from_tube].len() > 0
            && *self.tubes[from_tube].last().unwrap() == move_color
            && self.tubes[index].len() < MAX_PORTION
        {
            let portion = self.tubes[from_tube].pop().unwrap();
            self.tubes[index].push(portion);
            println!("transfer");
        }
        self.from_tube = None;
        self.requested_sounds.push("pour.wav");
    }

    pub fn check_clear(&mut self) {
        if self.tubes.iter().all(|tube| {
            tube.iter().all(|portion| *portion == EMPTY)
                || tube.len() == MAX_PORTION && tube.iter().all(|portion| *portion == tube[0])
        }) {
            self.is_clear = true;
            self.requested_sounds.push("bravo.wav");
        }
    }
}
