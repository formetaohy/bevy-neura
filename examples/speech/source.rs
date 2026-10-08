use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

pub const MODEL: &str = "openai/whisper-tiny";

const ENDPOINT: &str = "https://huggingface.co";
const FILES: [&str; 5] = [
    "config.json",
    "preprocessor_config.json",
    "vocab.json",
    "added_tokens.json",
    "model.safetensors",
];

pub fn directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("speech")
        .join("whisper-tiny")
}

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
    pub fn start(directory: &Path) -> Option<Self> {
        let missing = FILES
            .iter()
            .copied()
            .filter(|name| !directory.join(name).is_file())
            .collect::<Vec<&str>>();
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
        std::thread::spawn(move || transfer(&directory, missing, &worker));
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

fn transfer(directory: &Path, missing: Vec<&str>, shared: &Mutex<Shared>) {
    let mut sizes = Vec::with_capacity(missing.len());
    let mut total = 0u64;
    for name in &missing {
        let reported = length(&url(name)).unwrap_or(0);
        total += reported;
        sizes.push(reported);
    }
    set(shared, State::Fetching { total });
    let mut done = 0u64;
    for (name, known) in missing.iter().zip(&sizes) {
        let target = directory.join(name);
        let part = target.with_extension("part");
        note(shared, done, part.clone());
        if let Err(error) = download(&url(name), &part) {
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
        done += if *known > 0 { *known } else { size(&target) };
        note(shared, done, PathBuf::new());
    }
    set(shared, State::Done);
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

fn url(name: &str) -> String {
    format!("{ENDPOINT}/{MODEL}/resolve/main/{name}")
}

fn download(url: &str, part: &Path) -> Result<(), String> {
    let status = curl()
        .args(["-fL", "-o"])
        .arg(part)
        .arg(url)
        .status()
        .map_err(|error| format!("curl does not run: {error}"))?;
    if !status.success() {
        return Err(format!(
            "curl answers {status} for {url}; fetch it by hand and write {}",
            part.display(),
        ));
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
