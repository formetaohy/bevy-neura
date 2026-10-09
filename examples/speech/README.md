# speech

Hold a button, speak, and a local `openai/whisper-tiny` model reads the microphone inside the frame
loop of a Bevy app and writes what it heard on screen. The window is the whole app: one button that
records while you hold it and the words that land.

```sh
cargo run --release --example speech
```

## Controls

* hold the button with the mouse for as long as you speak and let go when you are done;
* a hold shorter than 0.3 s is dropped, and a hold past 28 s is cut short and read at that cap;
* a hold the model hears no speech in is dropped as well, so the page keeps the words you said and
  nothing else.

## What the picture says

The window carries the state of the app as text and colour:

* the words land above the button, the newest line in the largest and brightest type, the older
  lines fading up the screen, and the last six lines are kept;
* the circle is the microphone: a small dim core while nothing records, red and growing with the
  level of what it hears while it records, amber and pulsing while the model reads;
* the button sits alone at the bottom of the window and greys out while a reading runs, taking the
  next hold when the reading is over.

The console carries what the picture cannot: the seconds of sound, the language, the tokens and the
seconds the encoder took for every reading.

## The first run

The first run downloads the five files of `openai/whisper-tiny` (about 145 MB) into
`target/speech/whisper-tiny` and builds the kernels of its graph, both behind a loading screen of a
progress bar, a percent display and the step it reads. Every run after that finds the files on disk
and goes straight to the microphone.

## What it needs

* a GPU with Vulkan, DX12 or Metal;
* a recording device, opened when loading ends: a machine without one stops there with
  `the microphone does not open: no device of this host records`;
* a font for the script you speak: the transcript is drawn with the system fonts of the host, so
  text of a script the host holds no font for stays blank.

The encoder of one utterance runs inside one frame, so the window stands still for the fraction of a
second a reading takes. Windows hands the microphone a signal with the speakers cancelled out of it,
so a reading of your own speakers is quieter than your own voice.
