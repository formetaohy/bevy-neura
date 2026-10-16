# speech

Hold a button, speak, and a local `openai/whisper-tiny` model reads the microphone inside the frame
loop of a Bevy app and writes what it heard on screen.

```sh
cargo run --release --example speech
```

## Controls

* hold the button with the mouse for as long as you speak and let go when you are done;
* a hold shorter than 0.3 s is dropped, a hold past 28 s is cut short, and a hold the model hears no
  speech in is dropped as well.

## First run

The first run downloads the five files of `openai/whisper-tiny` (about 145 MB) into
`target/speech/whisper-tiny` and builds the kernels of its graph, both behind a loading screen. Every
run after that finds the files on disk and goes straight to the microphone.

## What it needs

* a GPU with Vulkan, DX12 or Metal and a recording device;
* a font for the script you speak: the transcript is drawn with the system fonts of the host, so text
  of a script the host holds no font for stays blank.

The encoder of one utterance runs inside one frame, so the window stands still for the fraction of a
second a reading takes.
