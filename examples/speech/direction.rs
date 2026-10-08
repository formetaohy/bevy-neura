#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn read(text: &str) -> Option<Self> {
        let words = format!(" {} ", normal(text));
        KEYWORDS
            .iter()
            .find(|(_, keywords)| {
                keywords.iter().any(|keyword| match keyword.chars().next() {
                    Some(letter) if letter.is_ascii() => words.contains(&format!(" {keyword} ")),
                    _ => words.contains(keyword),
                })
            })
            .map(|(direction, _)| *direction)
    }

    pub fn name(self) -> &'static str {
        match self {
            Direction::Left => "left",
            Direction::Right => "right",
            Direction::Up => "up",
            Direction::Down => "down",
        }
    }
}

const KEYWORDS: [(Direction, &[&str]); 4] = [
    (Direction::Left, &["left", "左"]),
    (Direction::Right, &["right", "右"]),
    (Direction::Up, &["up", "上"]),
    (Direction::Down, &["down", "下"]),
];

fn normal(text: &str) -> String {
    let lowered = text.to_lowercase();
    let mut out = String::with_capacity(lowered.len());
    for letter in lowered.chars() {
        if letter.is_ascii_alphanumeric() || is_han(letter) {
            out.push(letter);
        } else {
            out.push(' ');
        }
    }
    out.split_whitespace().collect::<Vec<&str>>().join(" ")
}

fn is_han(letter: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&letter)
}
