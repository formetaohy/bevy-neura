use crate::digit;

pub struct Archives {
    pub train_digits: Vec<u8>,
    pub train_answers: Vec<u8>,
    pub test_digits: Vec<u8>,
    pub test_answers: Vec<u8>,
}

pub struct Dataset {
    train_digits: Vec<u8>,
    train_answers: Vec<u8>,
    test_digits: Vec<u8>,
    test_answers: Vec<u8>,
}

impl Dataset {
    pub fn of(archives: Archives) -> Self {
        let (train_digits, train_count) = images(&archives.train_digits);
        let (train_answers, train_held) = answers(&archives.train_answers);
        let (test_digits, test_count) = images(&archives.test_digits);
        let (test_answers, test_held) = answers(&archives.test_answers);
        assert_eq!(
            [train_count, test_count],
            [train_held, test_held],
            "the archives hold {train_count} and {test_count} digits beside {train_held} and {test_held} answers",
        );
        Self {
            train_digits,
            train_answers,
            test_digits,
            test_answers,
        }
    }

    pub fn train_size(&self) -> u32 {
        self.train_answers.len() as u32
    }

    pub fn test_size(&self) -> u32 {
        self.test_answers.len() as u32
    }

    pub fn train(&self, index: u32) -> (&[u8], u8) {
        (
            image(&self.train_digits, index),
            self.train_answers[index as usize],
        )
    }

    pub fn test(&self, index: u32) -> (&[u8], u8) {
        (
            image(&self.test_digits, index),
            self.test_answers[index as usize],
        )
    }
}

fn image(digits: &[u8], index: u32) -> &[u8] {
    let side = digit::SIDE;
    assert!(
        (index as usize) < digits.len() / (side * side),
        "the digit {index} lies beyond {} digits",
        digits.len() / (side * side),
    );
    let start = index as usize * side * side;
    &digits[start..start + side * side]
}

fn images(bytes: &[u8]) -> (Vec<u8>, u32) {
    let mut dims = [0u32; 3];
    let magic = header(bytes, &mut dims);
    assert_eq!(
        magic, 0x0000_0803,
        "an archive of images opens {magic:#010x}"
    );
    let [count, rows, columns] = dims;
    assert_eq!(
        [rows, columns],
        [digit::SIDE as u32, digit::SIDE as u32],
        "an archive of images holds {rows} by {columns} pixels",
    );
    let side = digit::SIDE * digit::SIDE;
    assert_eq!(
        bytes.len(),
        16 + count as usize * side,
        "an archive of {count} digits of {side} pixels holds {} bytes",
        bytes.len(),
    );
    (bytes[16..].to_vec(), count)
}

fn answers(bytes: &[u8]) -> (Vec<u8>, u32) {
    let mut dims = [0u32; 1];
    let magic = header(bytes, &mut dims);
    assert_eq!(
        magic, 0x0000_0801,
        "an archive of answers opens {magic:#010x}"
    );
    let [count] = dims;
    assert_eq!(
        bytes.len(),
        8 + count as usize,
        "an archive of {count} answers holds {} bytes",
        bytes.len(),
    );
    (bytes[8..].to_vec(), count)
}

fn header(bytes: &[u8], dims: &mut [u32]) -> u32 {
    let words = dims.len() + 1;
    assert!(
        bytes.len() >= 4 * words,
        "an archive of {words} words opens {} bytes",
        bytes.len(),
    );
    for (at, dim) in dims.iter_mut().enumerate() {
        let word = 4 * at + 4;
        *dim = u32::from_be_bytes(bytes[word..word + 4].try_into().expect("a word"));
    }
    u32::from_be_bytes(bytes[..4].try_into().expect("a word"))
}
