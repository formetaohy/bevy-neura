use std::path::{Path, PathBuf};
use std::process::Command;

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

pub fn ensure(directory: &Path) {
    let missing = FILES
        .iter()
        .filter(|name| !directory.join(name).is_file())
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return;
    }
    println!("fetching {MODEL} into {}", directory.display());
    std::fs::create_dir_all(directory)
        .unwrap_or_else(|error| panic!("no directory of {}: {error}", directory.display()));
    for name in missing {
        let url = format!("{ENDPOINT}/{MODEL}/resolve/main/{name}");
        fetch(&url, &directory.join(name));
    }
}

fn fetch(url: &str, target: &Path) {
    println!("  {url}");
    let part = target.with_extension("part");
    let status = Command::new("curl")
        .args(["-fL", "--progress-bar", "--retry", "3", "-o"])
        .arg(&part)
        .arg(url)
        .status()
        .unwrap_or_else(|error| panic!("curl reads no {url}: {error}"));
    assert!(
        status.success(),
        "curl reads no response of {url}; fetch it by hand and write {}",
        target.display(),
    );
    std::fs::rename(&part, target)
        .unwrap_or_else(|error| panic!("no {}: {error}", target.display()));
}
