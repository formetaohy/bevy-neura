use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub enum Report {
    Measuring,
    Fetching { done: u64, total: u64 },
    Done,
    Failed(String),
}

pub struct Artifact {
    name: String,
    urls: Vec<String>,
}

impl Artifact {
    pub fn of(name: &str, urls: impl IntoIterator<Item = String>) -> Self {
        let urls = urls.into_iter().collect::<Vec<String>>();
        assert!(!urls.is_empty(), "the artifact {name} names no host");
        Self {
            name: name.to_string(),
            urls,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
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
    pub fn start(directory: &Path, artifacts: Vec<Artifact>) -> Option<Self> {
        assert!(
            !artifacts.is_empty(),
            "a transfer carries no artifact of {}",
            directory.display(),
        );
        let missing = artifacts
            .into_iter()
            .filter(|artifact| !directory.join(artifact.name()).is_file())
            .collect::<Vec<Artifact>>();
        if missing.is_empty() {
            return None;
        }
        std::fs::create_dir_all(directory)
            .unwrap_or_else(|error| panic!("no directory of {}: {error}", directory.display()));
        let shared = Arc::new(Mutex::new(Shared {
            state: State::Measuring,
            part: PathBuf::new(),
            done: 0,
        }));
        let worker = Arc::clone(&shared);
        let directory = directory.to_path_buf();
        std::thread::spawn(move || transfer(&directory, &missing, &worker));
        Some(Self { shared })
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

fn transfer(directory: &Path, missing: &[Artifact], shared: &Mutex<Shared>) {
    let sizes = missing.iter().map(expected).collect::<Vec<u64>>();
    let total = sizes.iter().sum::<u64>();
    set(shared, State::Fetching { total });
    let mut done = 0u64;
    for (artifact, known) in missing.iter().zip(sizes) {
        let target = directory.join(artifact.name());
        let part = target.with_extension("part");
        note(shared, done, part.clone());
        if let Err(error) = fetch(artifact, &part) {
            set(shared, State::Failed(error));
            return;
        }
        if let Err(error) = std::fs::rename(&part, &target) {
            set(
                shared,
                State::Failed(format!("no {}: {error}", target.display())),
            );
            return;
        }
        done += if known > 0 { known } else { size(&target) };
        note(shared, done, PathBuf::new());
    }
    set(shared, State::Done);
}

fn expected(artifact: &Artifact) -> u64 {
    artifact
        .urls
        .iter()
        .filter_map(|url| length(url))
        .max()
        .unwrap_or(0)
}

fn fetch(artifact: &Artifact, part: &Path) -> Result<(), String> {
    let mut refused = Vec::new();
    for url in &artifact.urls {
        match download(url, part) {
            Ok(()) => return Ok(()),
            Err(error) => refused.push(error),
        }
    }
    Err(format!("{}: {}", artifact.name(), refused.join("; ")))
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
