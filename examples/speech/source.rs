use std::path::{Path, PathBuf};
use std::process::Command;

pub const FILES: [&str; 5] = [
    "config.json",
    "preprocessor_config.json",
    "vocab.json",
    "added_tokens.json",
    "model.safetensors",
];

pub fn model() -> String {
    std::env::var("SPEECH_MODEL").unwrap_or_else(|_| "openai/whisper-tiny".to_string())
}

pub fn endpoint() -> String {
    std::env::var("SPEECH_MODEL_ENDPOINT")
        .unwrap_or_else(|_| "https://huggingface.co".to_string())
        .trim_end_matches('/')
        .to_string()
}

pub fn directory() -> PathBuf {
    match std::env::var_os("SPEECH_MODEL_DIR") {
        Some(directory) => PathBuf::from(directory),
        None => {
            let name = model().replace('/', "--");
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("speech")
                .join(name)
        }
    }
}

pub fn ensure(directory: &Path) {
    let missing = FILES
        .iter()
        .filter(|name| !directory.join(name).is_file())
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return;
    }
    println!("fetching {} into {}", model(), directory.display());
    std::fs::create_dir_all(directory)
        .unwrap_or_else(|error| panic!("no directory of {}: {error}", directory.display()));
    let endpoint = endpoint();
    for name in missing {
        let url = format!("{endpoint}/{}/resolve/main/{name}", model());
        fetch(&url, &directory.join(name));
    }
}

fn fetch(url: &str, target: &Path) {
    println!("  {url}");
    let attempts: [(&str, Vec<String>); 3] = [
        (
            "curl",
            vec![
                "-fL".to_string(),
                "--progress-bar".to_string(),
                "-o".to_string(),
                target.display().to_string(),
                url.to_string(),
            ],
        ),
        (
            "wget",
            vec![
                "-q".to_string(),
                "-O".to_string(),
                target.display().to_string(),
                url.to_string(),
            ],
        ),
        (
            "powershell",
            vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                format!(
                    "Invoke-WebRequest -UseBasicParsing -Uri '{url}' -OutFile '{}'",
                    target.display()
                ),
            ],
        ),
    ];
    for (program, arguments) in attempts {
        let Ok(status) = Command::new(program).args(&arguments).status() else {
            continue;
        };
        assert!(
            status.success(),
            "{program} reads no response of {url}; check the network and SPEECH_MODEL_ENDPOINT",
        );
        return;
    }
    panic!("none of curl, wget and powershell answers {url}");
}
