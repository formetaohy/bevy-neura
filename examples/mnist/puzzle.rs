use bevy_neura_example::random::Random;

pub const DIGITS: u32 = 10;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Puzzle {
    pub left: u32,
    pub operation: Operation,
    pub right: u32,
}

impl Operation {
    pub const ALL: [Self; 3] = [Self::Add, Self::Subtract, Self::Multiply];

    pub fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "×",
        }
    }

    pub fn terms(self, answer: u32) -> Vec<(u32, u32)> {
        let top = DIGITS - 1;
        match self {
            Self::Add => (0..=answer).map(|left| (left, answer - left)).collect(),
            Self::Subtract => (0..=top - answer)
                .map(|right| (right + answer, right))
                .collect(),
            Self::Multiply => (2..=top)
                .filter(|left| answer.is_multiple_of(*left))
                .map(|left| (left, answer / left))
                .filter(|(_, right)| (2..=top).contains(right))
                .collect(),
        }
    }
}

impl Puzzle {
    pub fn draw(random: &mut Random) -> Self {
        let answer = random.below(DIGITS as usize) as u32;
        let operations = Operation::ALL
            .into_iter()
            .filter(|operation| !operation.terms(answer).is_empty())
            .collect::<Vec<Operation>>();
        let operation = operations[random.below(operations.len())];
        let terms = operation.terms(answer);
        let (left, right) = terms[random.below(terms.len())];
        Self {
            left,
            operation,
            right,
        }
    }

    pub fn answer(self) -> u32 {
        match self.operation {
            Operation::Add => self.left + self.right,
            Operation::Subtract => self.left - self.right,
            Operation::Multiply => self.left * self.right,
        }
    }

    pub fn text(self) -> String {
        format!("{} {} {} =", self.left, self.operation.symbol(), self.right)
    }
}
