# speech

A little pac-man whose only control is your voice: say `left`, `right`, `up` or `down`, and a local
**Whisper** model reads what you said right inside the frame loop of a Bevy app.

```sh
cargo run --release --example speech
```

## The game

A maze of 170 dots and four power pellets holds a pac, a house of four ghosts and three lives. The
pac is the round yellow mouth of the arcade, and it points that mouth at the lane of the last word
and chews while it walks, so a word that turned it shows itself even where a wall of the maze stops
it. It walks nothing but the direction of that word: it keeps that direction at a constant speed
through every tile until a wall stops it, a word for the lane it came from turns it around where it
stands, and a word the tile below cannot walk leaves it waiting on the spot instead of walking on.
The ghosts leave their house one by one and chase it, a power pellet turns them blue and edible for
seven seconds, and eating every dot clears the level and speeds the next one up. A word spoken
while the game is over starts a new run, so the maze listens for four words and nothing else. Arrow
keys steer as well, which is how the game is checked without a microphone.

Three lines and a notice carry the whole of the interface:

```
score 120   level 1   lives 2
[listening] heard "move left" -> left
say left, right, up or down, or press the arrow keys
```

## The loading screen

The first run fetches `openai/whisper-tiny` (145 MB of config, vocabulary and weights) into
`target/speech/whisper-tiny`, and the loading screen names every step of the way with the bytes of
the download it has already read:

```
voice pac-man
whisper reads the microphone in the frame loop

[x] openai/whisper-tiny is here
[x] the checkpoint, vocabulary and mel banks
[>] the encoder behind the microphone
[ ] the decoder that names the words
[ ] the microphone that feeds the maze
[#####################-------]
```

The download runs on a thread of its own and the frame loop reads the size of the part file it
writes, so the bar follows the transfer instead of guessing at it; every run after the first finds
the five files on disk and starts at the checkpoint. The first run of a machine also builds the
kernels of the graph once and neura keeps them in its artifact cache, which is what the encoder and
the decoder of that run spend their seconds on. Nothing is configured: the model, its directory,
the words and the heap the device holds are constants of the example.

## How a reading runs

The microphone streams frames into a bounded ring, and every frame of the game drains that ring
into an utterance:

1. an rms of 0.01 opens an utterance, and 0.35 s of quiet under an rms of 0.006 (or five seconds of
   speech) closes it, where a device that hands no samples at all counts as quiet too;
2. a resampler of windowed sinc taps carries the utterance to the 16 kHz Whisper reads (32 taps,
   a Blackman window, one phase table per 44.1 kHz or 48 kHz device);
3. a spectrogram of 3000 frames of 400 samples (hop 160, a periodic Hann window, reflection at both
   ends) reads the mel filterbank of the model, logarithm and all;
4. the encoder program answers with 1500 states and the keys and values every decoder layer reads;
5. the host carries those keys and values once into the decoder program, which keeps them for the
   whole utterance;
6. one `run` per frame names the next token on the device with `argmax`, so the game keeps its
   frames while a reading proceeds;
7. the language of the reading is the language token its first step weighs highest, and the tokens
   the reading names afterwards read back through the vocabulary of the model, where the four words
   of the game are looked up.

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

`cargo test --test game` holds the maze and the game loop of the example to their own rules,
without a window, a model or a microphone:

* `the_pellets_of_a_maze_wait_behind_corridors_a_pac_can_walk` walks the maze of the layout and
  finds every pellet, and no way into the house;
* `a_ghost_leaves_the_house_and_hunts_through_the_door_it_keeps` walks it as a ghost does, out of
  the house through the door and never back in;
* `the_hud_names_the_direction_a_word_asks_for` reads a word of a transcript into the two names
  the game names it with;
* `a_spoken_word_turns_the_pac_and_the_dots_it_crosses_count` runs the two systems of the game in a
  headless app: a written utterance walks the pac into the next tile, and the dot it crosses scores;
* `a_word_of_nothing_leaves_the_pac_where_it_stands` holds a transcript without a word of the game
  to a pac that stays where it is;
* `a_word_for_the_lane_a_pac_came_from_turns_it_around` walks a pac into a corridor and reads the
  opposite word as a pac that turns where it stands, and not one tile later;
* `the_mouth_of_the_pac_points_where_the_word_walks_it` reads all four words and holds the mouth of
  the sector mesh the pac draws itself with to the lane of each one;
* `a_word_the_maze_cannot_walk_leaves_the_pac_waiting` reads a word for a lane a wall of the maze
  shuts, and finds a pac waiting on the centre of its tile instead of walking on;
* `a_wall_stops_the_pac_and_the_next_word_turns_it` reads a corridor to the wall that ends it, and
  the next word out of it again;
* `a_word_after_the_last_life_opens_a_new_maze` lets a pac that never moves lose three lives and
  reads the next word as a new run: score 0, three lives and a pac at its start;
* `a_ghost_that_catches_a_pac_that_stands_still_takes_a_life` lets forty seconds of chase pass and
  finds a pac back at its start with one life less.

`cargo test --test speech` checks the reading against a torch reference without a network:

* `a_spectrogram_matches_the_reference` reads a synthetic utterance through the mel front end and
  the filterbank of the model, against torch's own spectrogram;
* `a_vocabulary_names_the_pieces_of_its_model` reads bytes of a byte level vocabulary, raw unicode
  and all;
* `a_small_model_answers_the_reference` runs a graph of the very same shape (two layers of eight
  numbers, two heads, sixty four tokens) against keys, values, logits, tokens and the transcript
  torch answers for it;
* `a_transcript_names_the_direction_it_holds` and
  `a_resampler_carries_the_tones_it_holds_and_leaves_the_tones_it_cannot` hold the two pure
  functions of the example to the words and the tones they must read.

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

* A machine without a recording device stops at the last step of the loading screen with
  `the microphone does not open: no device of this host records`, so the arrow keys are the game
  of a machine that cannot listen.
* Windows hands the microphone a signal with the speakers cancelled out of it, so a reading of your
  own speakers is quieter than your own voice.
* A reading quieter than an rms of 0.01 is never opened, which is what keeps the whispers of silence
  and the fans of a desktop out of the game.
* The download passes `--ssl-no-revoke` on Windows: a machine whose revocation servers are
  unreachable otherwise refuses the connection of every checkpoint file.
* Everything is on the thread of the frame loop: the microphone streams on the thread of the host
  its device hands it, and a reading of one utterance stalls its frame for the length of the encoder
  alone. The maze keeps the time it is given, so a stall pauses the ghosts instead of teleporting
  them.
* Bevy embeds a font of ASCII glyphs, so the labels of the window stay in ASCII; text of another
  script reads as blanks unless a font of its own is loaded.
