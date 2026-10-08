#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Flap;

impl Flap {
    pub fn read(text: &str) -> Option<Self> {
        let words = format!(" {} ", words(text));
        KEYWORDS
            .iter()
            .any(|keyword| words.contains(&format!(" {keyword} ")))
            .then_some(Self)
    }
}

const KEYWORDS: [&str; 3] = ["fly", "飞", "飛"];

fn words(text: &str) -> String {
    let mut spaced = String::with_capacity(text.len() * 3);
    for letter in text.to_lowercase().chars() {
        if letter.is_ascii_alphanumeric() {
            spaced.push(letter);
        } else if is_han(letter) {
            spaced.push(' ');
            spaced.push(letter);
            spaced.push(' ');
        } else {
            spaced.push(' ');
        }
    }
    spaced.split_whitespace().collect::<Vec<&str>>().join(" ")
}

fn is_han(letter: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&letter)
}
