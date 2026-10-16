# lunar

Sixty-four landers and a proximal policy optimization trained inside the frame loop; a showcase lander
flies the greedy policy of the moment over a cratered moon.

```sh
cargo run --release --example lunar
```

## Controls

* `space` switches between eight and thirty-two landers a frame;
* `p` pauses the training and leaves the showcase flying;
* `r` replays the showcase episode.

## Notes

* the legs of the lander are rigid, so a landing that would flex them tips the craft instead;
* the device needs a Vulkan, DX12 or Metal adapter and 256 MB of heap;
* the randomness is seeded from a constant, so every run flies the same episodes.
