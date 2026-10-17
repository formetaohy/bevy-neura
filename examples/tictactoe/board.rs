pub const CELLS: usize = 9;

pub const SYMMETRIES: [[usize; CELLS]; 8] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8],
    [2, 5, 8, 1, 4, 7, 0, 3, 6],
    [8, 7, 6, 5, 4, 3, 2, 1, 0],
    [6, 3, 0, 7, 4, 1, 8, 5, 2],
    [6, 7, 8, 3, 4, 5, 0, 1, 2],
    [2, 1, 0, 5, 4, 3, 8, 7, 6],
    [0, 3, 6, 1, 4, 7, 2, 5, 8],
    [8, 5, 2, 7, 4, 1, 6, 3, 0],
];

const LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Ongoing,
    Won,
    Drawn,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Board {
    cells: [i8; CELLS],
    turn: i8,
}

impl Board {
    pub fn start() -> Self {
        Self {
            cells: [0; CELLS],
            turn: 1,
        }
    }

    pub fn cells(&self) -> &[i8; CELLS] {
        &self.cells
    }

    pub fn perspective(&self) -> [f32; CELLS] {
        self.cells.map(|mark| f32::from(mark * self.turn))
    }

    pub fn legal(&self) -> impl Iterator<Item = usize> + '_ {
        (0..CELLS).filter(|at| self.cells[*at] == 0)
    }

    pub fn play(&mut self, at: usize) -> Outcome {
        assert!(
            at < CELLS && self.cells[at] == 0,
            "cell {at} holds {} and takes no mark",
            self.cells[at],
        );
        self.cells[at] = self.turn;
        let outcome = if self.wins(self.turn) {
            Outcome::Won
        } else if self.cells.iter().all(|mark| *mark != 0) {
            Outcome::Drawn
        } else {
            Outcome::Ongoing
        };
        self.turn = -self.turn;
        outcome
    }

    fn wins(&self, mark: i8) -> bool {
        LINES
            .iter()
            .any(|line| line.iter().all(|at| self.cells[*at] == mark))
    }
}
