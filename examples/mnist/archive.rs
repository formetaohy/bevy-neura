use crate::dataset::Archives;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Archive {
    TrainDigits,
    TrainAnswers,
    TestDigits,
    TestAnswers,
}

impl Archive {
    pub const ALL: [Self; 4] = [
        Self::TrainDigits,
        Self::TrainAnswers,
        Self::TestDigits,
        Self::TestAnswers,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::TrainDigits => "train-images-idx3-ubyte.gz",
            Self::TrainAnswers => "train-labels-idx1-ubyte.gz",
            Self::TestDigits => "t10k-images-idx3-ubyte.gz",
            Self::TestAnswers => "t10k-labels-idx1-ubyte.gz",
        }
    }
}

pub fn directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("mnist")
}

pub fn checkpoint() -> PathBuf {
    directory().join("lenet.neura")
}

pub fn held(directory: &Path) -> bool {
    Archive::ALL
        .into_iter()
        .all(|archive| directory.join(archive.name()).is_file())
}

pub fn unpack(directory: &Path) -> Archives {
    Archives {
        train_digits: inflate(directory, Archive::TrainDigits),
        train_answers: inflate(directory, Archive::TrainAnswers),
        test_digits: inflate(directory, Archive::TestDigits),
        test_answers: inflate(directory, Archive::TestAnswers),
    }
}

fn inflate(directory: &Path, archive: Archive) -> Vec<u8> {
    let path = directory.join(archive.name());
    let compressed =
        std::fs::read(&path).unwrap_or_else(|error| panic!("no {}: {error}", path.display()));
    let mut bytes = Vec::new();
    flate2::read::GzDecoder::new(compressed.as_slice())
        .read_to_end(&mut bytes)
        .unwrap_or_else(|error| panic!("{} does not open: {error}", path.display()));
    bytes
}
