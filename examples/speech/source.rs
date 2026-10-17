use bevy_neura_example::transfer::Artifact;
use std::path::{Path, PathBuf};

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

pub fn artifacts() -> Vec<Artifact> {
    FILES
        .iter()
        .map(|name| Artifact::of(name, [format!("{ENDPOINT}/{MODEL}/resolve/main/{name}")]))
        .collect()
}
