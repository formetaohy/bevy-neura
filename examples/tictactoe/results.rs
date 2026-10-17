use crate::board::Ending;
use std::collections::VecDeque;

pub const CHUNK: u32 = 1_000;
pub const RECENT: usize = 128;
const SHORTEST: u32 = 5;
const LONGEST: u32 = 9;

#[derive(Clone, Copy, Default)]
pub struct Chunk {
    pub x_wins: u32,
    pub o_wins: u32,
    pub draws: u32,
}

impl Chunk {
    pub fn count(&self, ending: Ending) -> u32 {
        match ending {
            Ending::XWins => self.x_wins,
            Ending::OWins => self.o_wins,
            Ending::Draw => self.draws,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Record {
    pub ending: Ending,
    pub moves: u32,
}

pub struct Results {
    total: u32,
    completed: u32,
    x_wins: u32,
    o_wins: u32,
    draws: u32,
    chunks: Vec<Chunk>,
    recent: VecDeque<Record>,
    moves: u64,
}

impl Results {
    pub fn new(total: u32) -> Self {
        assert!(total > 0, "a training of {total} games holds none");
        Self {
            total,
            completed: 0,
            x_wins: 0,
            o_wins: 0,
            draws: 0,
            chunks: vec![Chunk::default(); total.div_ceil(CHUNK) as usize],
            recent: VecDeque::with_capacity(RECENT),
            moves: 0,
        }
    }

    pub fn record(&mut self, ending: Ending, moves: u32) {
        assert!(
            self.completed < self.total,
            "a training of {} games records no more",
            self.total,
        );
        assert!(
            (SHORTEST..=LONGEST).contains(&moves),
            "a tic-tac-toe game ends in {SHORTEST} to {LONGEST} moves, not {moves}",
        );
        let chunk = &mut self.chunks[(self.completed / CHUNK) as usize];
        match ending {
            Ending::XWins => {
                self.x_wins += 1;
                chunk.x_wins += 1;
            }
            Ending::OWins => {
                self.o_wins += 1;
                chunk.o_wins += 1;
            }
            Ending::Draw => {
                self.draws += 1;
                chunk.draws += 1;
            }
        }
        self.moves += u64::from(moves);
        self.completed += 1;
        if self.recent.len() == RECENT {
            self.recent.pop_front();
        }
        self.recent.push_back(Record { ending, moves });
    }

    pub fn completed(&self) -> u32 {
        self.completed
    }

    pub fn total(&self) -> u32 {
        self.total
    }

    pub fn x_wins(&self) -> u32 {
        self.x_wins
    }

    pub fn o_wins(&self) -> u32 {
        self.o_wins
    }

    pub fn draws(&self) -> u32 {
        self.draws
    }

    pub fn chunks(&self) -> &[Chunk] {
        &self.chunks
    }

    pub fn recent(&self) -> &VecDeque<Record> {
        &self.recent
    }

    pub fn average_moves(&self) -> f32 {
        if self.completed == 0 {
            return 0.0;
        }
        self.moves as f32 / self.completed as f32
    }
}
