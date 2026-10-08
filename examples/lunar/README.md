# lunar

The lunar lander of OpenAI Gym, rebuilt in Bevy, where a proximal policy optimization learns to land
it from inside the frame loop: the policy, the critic, the clipped surrogate and the AdamW update are
one Neura graph, sixty-four landers fly at once, and the scene is a cratered moon with a pad, flags,
beacons and engine exhaust.

```sh
cargo run --release --example lunar
```

## Controls

* `space` switches between eight and thirty-two landers a frame, which is the speed of the training;
* `p` pauses the training and leaves the showcase flying;
* `r` replays the showcase episode.

## What learns what

Every frame the training flies one step of sixty-four landers at a time. The policy samples one of
four engines per lander on the device (`categorical`), the landers step, and each transition lands in
a rollout of sixty-four steps. One rollout of four thousand and ninety-six transitions then becomes
four epochs of eight minibatches of five hundred and twelve transitions: the clipped surrogate of the
policy, the squared error of the critic, an entropy bonus of 0.01, and one AdamW step of 3e-4 follow
the advantages of a generalized advantage estimation of 0.95 under a discount of 0.99.

The learner and the showcase are two programs of one weight store, so the showcase always answers
what the learner has just learned: it flies one step a frame with the greedy action and paints its
score beside the history of the training.

## What the window shows

The upper left names the device, the samples and the updates of the training, its speed in
transitions a second, the episodes it has flown, the mean and the best of the last fifty episodes,
the losses of the last update, and the explained variance of the critic. The upper right names the
showcase: the return so far, the worth of its state, and the shape of its policy over the four
engines. The bottom left charts the return of the last ninety-six episodes over the zero line of the
reward and the line of the two hundred points that the original game counts as solved, and a landing
or a crash flashes the screen and names itself.

## Fidelity

The terrain, the pad, the four engines, the observations, the reward, the terminations and the sleep
of a landed craft follow `lunar_lander.py` of OpenAI Gym, down to its quirks: the pad is smoothed to
3.3 where its flags stand at 3.33, the side engines push from fourteen pixels above the centre, and a
landed craft earns its hundred points only after half a second of rest. The engines, the terrain and
the pad are drawn from the shapes that collide, so the window shows the simulation.

One difference remains: the legs of the original hang on a pair of revolutions with a motor of forty
newton metres and a limit, a spring that a sequential impulse solver of this size does not hold, so
here the legs are rigid. The wind of the original stays off, as it is by default.

The randomness is a PCG64 of numpy seeded from a constant, so every run flies the same episodes and
the curve of the chart is the same curve.
