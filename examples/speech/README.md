# speech

A pac-man whose only control is your voice: say `left`, `right`, `up` or `down`, and a local
`openai/whisper-tiny` model reads the microphone and turns the pac inside the frame loop of a Bevy
app. Arrow keys steer as well, so the game plays without a voice.

```sh
cargo run --release --example speech
```

## Controls

* say `left`, `right`, `up` or `down`, matched as words of whatever whisper reads, so `move left`
  counts too;
* press the arrow keys for the same four directions.

The window names the score, level and lives, what the microphone is doing (`waiting`, `listening`,
`reading`, `transcribing`), the last transcript with the direction it read, and the state of a run:
`get ready`, `caught!`, `level cleared` or `game over - say a direction to play again`, where the
next word opens a new run.

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

A reading of one utterance stalls the frame for the length of the encoder alone, so the maze pauses
instead of skipping. Windows hands the microphone a signal with the speakers cancelled out of it, so
a reading of your own speakers is quieter than your own voice. Bevy embeds a font of ASCII glyphs,
so a transcript of another script reads as blanks in the window.
