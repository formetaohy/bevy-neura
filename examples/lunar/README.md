# lunar

Sixty-four landers and a proximal policy optimization that learns to land them: the policy, the
critic, the clipped surrogate and the AdamW update are one Neura graph that trains inside the frame
loop, while a showcase lander flies the greedy policy of the moment over a cratered moon with a pad,
flags and engine exhaust.

```sh
cargo run --release --example lunar
```

## Controls

* `space` switches between eight and thirty-two landers a frame, the speed of the training;
* `p` pauses the training and leaves the showcase flying;
* `r` replays the showcase episode.

## What trains

Every frame the training flies one step of sixty-four landers: the policy samples one of four engines
per lander on the device, and each transition lands in a rollout of sixty-four steps. A rollout of
four thousand and ninety-six transitions then becomes four epochs of eight minibatches of five hundred
and twelve transitions — the clipped surrogate of the policy, the squared error of the critic and an
entropy bonus of 0.01, under one AdamW step of 3e-4 and the advantages of a generalized advantage
estimation of 0.95 with a discount of 0.99.

The showcase answers the same weight store, so it always flies what the learner has just learned. The
window names the device, the samples, the speed and the losses of the training, charts the return of
the last ninety-six episodes over the zero line and the two hundred points of a solved landing, and
flashes a landing or a crash.

## Notes

* the legs of the lander are rigid, so a landing that would flex them tips the craft instead;
* the engines, the terrain and the pad are drawn from the shapes that collide, so the window shows the
  simulation;
* the randomness is seeded from a constant, so every run flies the same episodes;
* the device needs a Vulkan, DX12 or Metal adapter and 256 MB of heap.
