use std::collections::HashMap;
use std::path::Path;

pub const SOT: u32 = 50258;
pub const EOT: u32 = 50257;
pub const TRANSCRIBE: u32 = 50359;
pub const NO_TIMESTAMPS: u32 = 50363;

pub struct Vocabulary {
    pieces: Vec<Vec<u8>>,
    languages: Vec<(u32, String)>,
}

impl Vocabulary {
    pub fn of(directory: &Path) -> Self {
        let tokens: HashMap<String, u32> = read(directory, "vocab.json");
        let mut pieces = vec![Vec::new(); EOT as usize];
        for (piece, token) in tokens {
            if (token as usize) < pieces.len() {
                pieces[token as usize] = bytes_of(&piece);
            }
        }
        let added: HashMap<String, u32> = read(directory, "added_tokens.json");
        let mut languages = added
            .into_iter()
            .filter_map(|(name, token)| {
                let code = name.strip_prefix("<|")?.strip_suffix("|>")?;
                (code.len() == 2 && code.chars().all(|letter| letter.is_ascii_lowercase()))
                    .then(|| (token, code.to_string()))
            })
            .collect::<Vec<(u32, String)>>();
        languages.sort_unstable();
        assert!(
            !languages.is_empty(),
            "the model of {} holds no language token",
            directory.display(),
        );
        Self { pieces, languages }
    }

    pub fn text(&self, tokens: &[u32]) -> String {
        let mut raw = Vec::new();
        for token in tokens {
            if *token >= EOT {
                continue;
            }
            raw.extend_from_slice(&self.pieces[*token as usize]);
        }
        String::from_utf8_lossy(&raw).into_owned()
    }

    pub fn languages(&self) -> &[(u32, String)] {
        &self.languages
    }
}

fn read(directory: &Path, name: &str) -> HashMap<String, u32> {
    let path = directory.join(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("the model holds no {}: {error}", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("{} holds no json: {error}", path.display()))
}

fn bytes_of(piece: &str) -> Vec<u8> {
    let encoder = byte_encoder();
    let mut bytes = Vec::with_capacity(piece.len());
    for character in piece.chars() {
        match encoder.get(&character) {
            Some(byte) => bytes.push(*byte),
            None => {
                let mut raw = [0u8; 4];
                bytes.extend_from_slice(character.encode_utf8(&mut raw).as_bytes());
            }
        }
    }
    bytes
}

fn byte_encoder() -> HashMap<char, u8> {
    let mut printable = (b'!'..=b'~')
        .chain(0xa1..=0xac)
        .chain(0xae..=0xff)
        .map(u32::from)
        .collect::<Vec<u32>>();
    let mut mapped = printable.clone();
    let mut escaped = 256u32;
    for byte in 0..256u32 {
        if !printable.contains(&byte) {
            printable.push(byte);
            mapped.push(escaped);
            escaped += 1;
        }
    }
    printable
        .into_iter()
        .zip(mapped)
        .map(|(byte, code)| {
            (
                char::from_u32(code).expect("a byte names a character"),
                byte as u8,
            )
        })
        .collect()
}
