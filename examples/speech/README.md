# speech

A complete little game whose only control is your voice: hold the button (or the space bar), say
`left`, `right`, `up`, `down`, `fire`, `freeze` or `restart`, and a local **Whisper** model reads
what you said right inside the frame loop of a Bevy app.

```sh
cargo run --release --example speech
```

The first run fetches `openai/whisper-tiny` (154 MB of config, vocabulary and weights) into
`target/speech`, and every run after that reads it from there. The window opens with a loading
notice, the model compiles, the microphone opens, and the arena appears.

| Environment | Meaning |
| --- | --- |
| `SPEECH_MODEL` | the model to read, `openai/whisper-tiny` by default; `openai/whisper-base` and larger work too |
| `SPEECH_MODEL_DIR` | a directory that already holds `config.json`, `preprocessor_config.json`, `vocab.json`, `added_tokens.json` and `model.safetensors` |
| `SPEECH_MODEL_ENDPOINT` | what a fetch reads, `https://huggingface.co` by default; set `https://hf-mirror.com` where hugging face is out of reach |
| `SPEECH_HEAP` | the megabytes of device heap the graph holds, 1024 by default; a model of 74 M numbers or more reads a larger figure |

## The game

A green square walks, red squares march at it, and the only weapon is your voice. `fire` burns the
closest pursuer within reach, `freeze` stops them for a moment, and each burnt pursuer is ten
points. Three pursuers reach the square and the run is over: say `restart` or press `R`. Arrow
keys and `WASD` steer as well, which is how the game is checked without a microphone.

## How a reading runs

The microphone streams 48 kHz frames into a bounded ring, the press of the button drains that ring
into an utterance, and the release reads it:

1. a resampler of windowed sinc taps carries the utterance to the 16 kHz Whisper reads (32 taps,
   a Blackman window, one phase table per 44.1 kHz or 48 kHz device);
2. a spectrogram of 3000 frames of 400 samples (hop 160, a periodic Hann window, reflection at both
   ends) reads the mel filterbank of the model, logarithm and all;
3. the encoder program answers with 1500 states and the keys and values every decoder layer reads;
4. the host carries those keys and values once into the decoder program, which keeps them for the
   whole utterance;
5. one `run` per frame names the next token on the device with `argmax`, so the game keeps its
   frames while a reading proceeds;
6. the language of the reading is the language token its first step weighs highest, and the tokens
   the reading names afterwards read back through the vocabulary of the model.

The graph is built once through `ModelPlan` and `Roles`, attached once to the runtime the plugin
holds, and poured with the checkpoint once. Every parameter of the graph carries the name the
checkpoint holds, and the loader lays each one out the way the graph reads it:

| checkpoint | graph |
| --- | --- |
| `*.self_attn.{q,k,v}_proj.weight` | `[heads, 1, d_model, width]`, the head of an output lane |
| `*.out_proj.weight` | `[heads, 1, width, d_model]`, merged by a sum over the head axis |
| `*.fc1.weight`, `*.fc2.weight` | `[d_model, ffn]` and `[ffn, d_model]` |
| `model.decoder.embed_tokens.weight` | the table itself and `proj_out.weight`, its transpose |
| `*.conv{1,2}.weight` | `[out, in, 1, taps]`, one row of a window |
| `*layer_norm.weight`, `*layer_norm.bias` | the scale and the shift of the layer |
| `*.embed_positions.weight` | the table every token reads its place from |

Nothing about Whisper lives in the plugin: the example reads the checkpoint, lays it out, and
drives `Model`, `Inputs` and `Runtime` the way any game would.

## What was checked

`cargo test --test speech` checks the example against a torch reference without a network:

* `a_spectrogram_matches_the_reference` reads a synthetic utterance through the mel front end and
  the filterbank of the model, against torch's own spectrogram;
* `a_vocabulary_names_the_pieces_of_its_model` reads bytes of a byte level vocabulary, raw unicode
  and all;
* `a_small_model_answers_the_reference` runs a graph of the very same shape (two layers of eight
  numbers, two heads, sixty four tokens) against keys, values, logits, tokens and the transcript
  torch answers for it;
* `a_transcript_names_the_command_it_holds` and
  `a_resampler_carries_the_tones_it_holds_and_leaves_the_tones_it_cannot` hold the two pure
  functions of the example to the tones and the phrasings they must read.

With a downloaded model and a wave of your own, the same check reads the whole of whisper-tiny:

```sh
python examples/speech/tools/reference.py --model-dir target/speech/whisper-tiny --audio speech.wav --out target/speech/reference
SPEECH_REFERENCE_DIR=target/speech/reference cargo test --release --test speech a_downloaded
```

On an RTX 2060 the encoder of whisper-tiny answers in about 0.23 s, a reading of eleven tokens
takes about 0.15 s after that, and the same audio reads the same transcript in torch and in the
graph to `6e-4` of a key, `1e-5` of a mel frame and exactly the same tokens. The reference reads
gelu the tanh way, which is what the graph computes and what whisper.cpp computes as well: the
exact erf gelu of torch names the very same transcript for the audio of that check.

## The reference of the checks

`tools/reference.py` writes every fixture of `tests/data/speech` and every golden the downloaded
model is checked against:

```sh
python examples/speech/tools/reference.py --fixture --out tests/data/speech
python examples/speech/tools/reference.py --mel-golden --out tests/data/speech
python examples/speech/tools/reference.py --vocabulary --out tests/data/speech
```

It reads torch and transformers, and the model of `target/speech/whisper-tiny`. `--medium` writes a
model of the frames of a real one with the numbers of a small one, which is how a difference of the
long attention of 1500 keys is told from a difference of the numbers; `--synthetic` reads a
checkpoint of its own instead of a wave, and every mode hands the mel, the keys, the values, the
logits and the tokens of a reading a checkpoint of the graph must answer.

## Notes

* Windows hands the microphone a signal with the speakers cancelled out of it, so a reading of your
  own speakers is quieter than your own voice.
* A reading quieter than an rms of 0.004 is dropped, which is what keeps the whispers of silence
  out of the game.
* Everything is on the thread of the frame loop: the microphone streams on the thread of the host
  its device hands it, and a reading of one utterance stalls its frame for the length of the encoder
  alone.
