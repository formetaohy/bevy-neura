# speech

A flappy bird whose only control is your voice: say `fly` (`飞` counts as well) and a local
`openai/whisper-tiny` model reads the microphone and flaps the bird inside the frame loop of a Bevy
app. The window, the pipes and the score are a Bevy game; the only control is your voice.

```sh
cargo run --release --example speech
```

## Controls

* say `fly` or `飞` for one flap, matched as words of whatever whisper reads, so `let me fly` counts
  too, where `butterfly` does not.

The window names the score and the best run, what the microphone is doing (`waiting`, `listening`,
`reading`, `transcribing`), the last transcript with whether it read a flap, and the state of a run:
`say fly to take off` or `game over - say fly to fly again`, where the next word opens a new run.

## The first run

The first run downloads the five files of `openai/whisper-tiny` (about 145 MB) into
`target/speech/whisper-tiny` and builds the kernels of its graph, both behind a loading screen with
a progress bar. Every run after that finds the files on disk and goes straight to the microphone.
The download needs `curl` on the `PATH` and access to `huggingface.co`.

## What it needs

* a GPU with Vulkan, DX12 or Metal;
* a recording device, opened when loading ends: a machine without one stops there with
  `the microphone does not open: no device of this host records`;
* speech loud enough to open a reading (an rms of 0.01): an utterance closes after 0.35 s of quiet
  and reads at most five seconds of speech.

A reading of one utterance stalls the frame for the length of the encoder alone, so the bird hangs
where it is instead of falling. Windows hands the microphone a signal with the speakers cancelled out
of it, so a reading of your own speakers is quieter than your own voice. Bevy embeds a font of ASCII
glyphs, so a transcript of another script reads as blanks in the window, and `飞` reaches the game
without ever reaching the window.
