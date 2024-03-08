use rand::prelude::*;
use std::time;

pub const FPS: i32 = 30;
pub const TUBE_COUNT: usize = 10;
pub const MAX_PORTION: usize = 4;
pub const COLOR_COUNT: usize = TUBE_COUNT - 2;
pub const EMPTY: i32 = 0;
pub const TRANSFERING_WAIT: i32 = 15;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Command {
    None,
    Select(usize),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GameState {
    Playing,
    Transfering,
}

pub struct Game {
    pub rng: StdRng,
    pub frame: i32,
    pub is_clear: bool,
    pub requested_sounds: Vec<&'static str>,
    pub tubes: Vec<Vec<i32>>,
    pub from_tube: Option<usize>,
    pub to_tube: Option<usize>,
    pub state: GameState,
    pub transfering_wait: i32,
    pub transferred_count: i32,
    pub transfering_color: i32,
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

        let mut game = Game {
            rng: rng,
            frame: -1,
            is_clear: false,
            requested_sounds: Vec::new(),
            tubes: Vec::new(),
            from_tube: None,
            to_tube: None,
            state: GameState::Playing,
            transfering_wait: 0,
            transferred_count: 0,
            transfering_color: 0,
        };

        // portionをランダムに初期配置
        let mut portions: Vec<i32> = vec![EMPTY; MAX_PORTION * COLOR_COUNT];
        let mut index = 0;
        for i in 0..COLOR_COUNT {
            for _ in 0..MAX_PORTION {
                portions[index] = i as i32 + 1;
                index += 1;
            }
        }
        portions.shuffle(&mut game.rng);

        // portionを各tubeにセット
        for i in 0..COLOR_COUNT {
            let tube: Vec<i32> = portions[(i * MAX_PORTION)..((i + 1) * MAX_PORTION)].to_vec();
            game.tubes.push(tube);
        }
        // 空のtubeをセット
        for _ in 0..(TUBE_COUNT - COLOR_COUNT) {
            game.tubes.push(Vec::new());
        }

        game
    }

    pub fn update(&mut self, command: Command) {
        self.frame += 1;

        if self.state == GameState::Transfering {
            self.transfering_wait -= 1;
            if self.transfering_wait == 0 {
                self.state = GameState::Playing;
                self.from_tube = None;
                self.transferred_count = 0;
                self.check_clear();
            }
            return;
        }

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
        self.transfering_color = *self.tubes[from_tube].last().unwrap();
        self.transferred_count = 0;
        while self.tubes[from_tube].len() > 0
            && *self.tubes[from_tube].last().unwrap() == self.transfering_color
            && self.tubes[index].len() < MAX_PORTION
        {
            let portion = self.tubes[from_tube].pop().unwrap();
            self.tubes[index].push(portion);
            println!("transfer");
            self.transferred_count += 1;
        }
        self.requested_sounds.push("pour.wav");
        self.state = GameState::Transfering;
        self.transfering_wait = TRANSFERING_WAIT;
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
