use crate::archive::Archive;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

const ENDPOINTS: [&str; 2] = [
    "https://storage.googleapis.com/cvdf-datasets/mnist",
    "https://ossci-datasets.s3.amazonaws.com/mnist",
];

#[derive(Clone, Debug)]
pub enum Report {
    Measuring,
    Fetching { done: u64, total: u64 },
    Done,
    Failed(String),
}

enum State {
    Measuring,
    Fetching { total: u64 },
    Done,
    Failed(String),
}

struct Shared {
    state: State,
    part: PathBuf,
    done: u64,
}

pub struct Transfer {
    shared: Arc<Mutex<Shared>>,
}

impl Transfer {
    pub fn start(directory: &Path) -> Self {
        let missing = Archive::ALL
            .into_iter()
            .filter(|archive| !directory.join(archive.name()).is_file())
            .collect::<Vec<Archive>>();
        assert!(
            !missing.is_empty(),
            "a transfer of the digits carries archives no host holds",
        );
        std::fs::create_dir_all(directory)
            .unwrap_or_else(|error| panic!("no directory of {}: {error}", directory.display()));
        let shared = Arc::new(Mutex::new(Shared {
            state: State::Measuring,
            part: PathBuf::new(),
            done: 0,
        }));
        let worker = Arc::clone(&shared);
        let directory = directory.to_path_buf();
        std::thread::spawn(move || transfer(&directory, missing, &worker));
        Self { shared }
    }

    pub fn report(&self) -> Report {
        let shared = self
            .shared
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        match &shared.state {
            State::Measuring => Report::Measuring,
            State::Fetching { total } => Report::Fetching {
                done: shared.done + size(&shared.part),
                total: *total,
            },
            State::Done => Report::Done,
            State::Failed(error) => Report::Failed(error.clone()),
        }
    }
}

fn transfer(directory: &Path, missing: Vec<Archive>, shared: &Mutex<Shared>) {
    let total = missing
        .iter()
        .map(|archive| expected(*archive))
        .sum::<u64>();
    set(shared, State::Fetching { total });
    let mut done = 0u64;
    for archive in &missing {
        let target = directory.join(archive.name());
        let part = target.with_extension("part");
        note(shared, done, part.clone());
        let length = match fetch(*archive, &part) {
            Ok(length) => length,
            Err(error) => {
                set(shared, State::Failed(error));
                return;
            }
        };
        if let Err(error) = std::fs::rename(&part, &target) {
            set(
                shared,
                State::Failed(format!("no {}: {error}", target.display())),
            );
            return;
        }
        done += length;
        note(shared, done, PathBuf::new());
    }
    set(shared, State::Done);
}

fn expected(archive: Archive) -> u64 {
    ENDPOINTS
        .iter()
        .filter_map(|endpoint| length(&url(endpoint, archive)))
        .max()
        .unwrap_or(0)
}

fn fetch(archive: Archive, part: &Path) -> Result<u64, String> {
    let mut refused = Vec::new();
    for endpoint in ENDPOINTS {
        let url = url(endpoint, archive);
        match download(&url, part) {
            Ok(()) => return Ok(size(part)),
            Err(error) => refused.push(error),
        }
    }
    Err(format!("{}: {}", archive.name(), refused.join("; ")))
}

fn note(shared: &Mutex<Shared>, done: u64, part: PathBuf) {
    let mut shared = shared.lock().unwrap_or_else(|error| error.into_inner());
    shared.done = done;
    shared.part = part;
}

fn set(shared: &Mutex<Shared>, state: State) {
    shared
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .state = state;
}

fn url(endpoint: &str, archive: Archive) -> String {
    format!("{endpoint}/{}", archive.name())
}

fn download(url: &str, part: &Path) -> Result<(), String> {
    let status = curl()
        .args(["-fL", "-o"])
        .arg(part)
        .arg(url)
        .status()
        .map_err(|error| format!("curl does not run: {error}"))?;
    if !status.success() {
        return Err(format!("curl answers {status} for {url}"));
    }
    Ok(())
}

fn length(url: &str) -> Option<u64> {
    let output = curl().args(["-I", "-L"]).arg(url).output().ok()?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .filter_map(|(_, value)| value.trim().parse().ok())
        .next_back()
}

fn curl() -> Command {
    let mut command = Command::new("curl");
    command.args(["-sS", "--retry", "3"]);
    #[cfg(windows)]
    command.arg("--ssl-no-revoke");
    command
}

fn size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|data| data.len()).unwrap_or(0)
}
