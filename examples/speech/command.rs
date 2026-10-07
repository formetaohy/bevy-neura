#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Command {
    Left,
    Right,
    Up,
    Down,
    Fire,
    Freeze,
    Restart,
    Unknown,
}

impl Command {
    pub fn read(text: &str) -> Self {
        let words = format!(" {} ", normal(text));
        for (command, keywords) in KEYWORDS {
            if keywords.iter().any(|keyword| match keyword.chars().next() {
                Some(letter) if letter.is_ascii() => words.contains(&format!(" {keyword} ")),
                _ => words.contains(keyword),
            }) {
                return command;
            }
        }
        Command::Unknown
    }

    pub fn name(self) -> &'static str {
        match self {
            Command::Left => "left",
            Command::Right => "right",
            Command::Up => "up",
            Command::Down => "down",
            Command::Fire => "fire",
            Command::Freeze => "freeze",
            Command::Restart => "restart",
            Command::Unknown => "nothing",
        }
    }
}

const KEYWORDS: [(Command, &[&str]); 7] = [
    (Command::Restart, &["restart", "again", "重来", "重新"]),
    (Command::Freeze, &["freeze", "stop", "停", "冻结"]),
    (Command::Fire, &["fire", "shoot", "开火", "射击"]),
    (Command::Left, &["left", "左"]),
    (Command::Right, &["right", "右"]),
    (Command::Up, &["up", "forward", "上", "前进"]),
    (Command::Down, &["down", "back", "下", "后退"]),
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
