# speechbird

A flappy bird whose only control is your voice: say `flap` (`飞` counts as well) and a local
`openai/whisper-tiny` model reads the microphone and flaps the bird inside the frame loop of a Bevy
app. The window, the pipes and the score are a Bevy game; the only control is your voice.

```sh
cargo run --release --example speechbird
```

## Controls

* say `flap` or `飞` for one flap, matched as words of whatever whisper reads, so `let me flap` counts
  too, where `flapping` does not.

## What the picture says

The window carries no text; every state of the game is drawn.

* the score is a pair of digit displays at the top of the sky, where the best run of the session sits
  below it in a dimmer tone;
* the orb at the foot of the window is the microphone, growing with the level of what it hears: it
  turns cyan while an utterance records, amber while the encoder reads it and violet while the
  decoder names its words;
* the ring that pulses around the bird asks for a word, and runs while a run waits to take off and
  after a run ends, where the next word opens a new one;
* a flap pushes a puff of air below the bird, and the run lands in a burst of dust that shakes the
  window.

The console carries what the picture cannot: the size of the checkpoint, the seconds a reading takes
and the transcript of every utterance.

## The first run

The first run downloads the five files of `openai/whisper-tiny` (about 145 MB) into
`target/speech/whisper-tiny` and builds the kernels of its graph, both behind a loading screen of a
progress bar, a percent display and one pip per step. Every run after that finds the files on disk
and goes straight to the microphone.

## What it needs

* a GPU with Vulkan, DX12 or Metal;
* a recording device, opened when loading ends: a machine without one stops there with
  `the microphone does not open: no device of this host records`;
* speech loud enough to open a reading (an rms of 0.01): an utterance closes after 0.35 s of quiet
  and reads at most five seconds of speech.

A reading of one utterance stalls the frame for the length of the encoder alone, so the bird hangs
where it is instead of falling. Windows hands the microphone a signal with the speakers cancelled out
of it, so a reading of your own speakers is quieter than your own voice.
