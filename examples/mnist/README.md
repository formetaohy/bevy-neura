# mnist

Write the answer to a question like `4 × 2 =` in a box on screen, and a LeNet-5 reads the digit you
wrote: the network learns the digits of MNIST inside the frame loop on the first run, then reads what
the pen drew, says whether it read the answer right, and keeps the score.

```sh
cargo run --release --example mnist
```

## Controls

* draw the answer in the box with the mouse, or with a finger on a touchscreen;
* the box is read once the pen rests for 0.6 s, so a digit of several strokes is read as one;
* a right answer brings the next question after a moment, and a wrong one keeps the question until
  the pen comes down again, which clears the box for another try.

## What the window says

* the hint at the top holds what to do;
* the question stands beside the box, and the box holds the ink of the pen;
* the line under the box names the digit the network read and how sure it is, in green for a right
  answer and in red for a wrong one;
* the last line counts the answers read right and the ones missed.

## What trains

Every question asks for one digit, so the `+`, the `-` and the `×` of the draws carry operands whose
answer lands between nothing and nine. The network is the LeNet-5 of two convolutions of six and
sixteen channels of five taps each, two pools of two, and three dense layers of one hundred and
twenty, eighty-four and ten numbers; the cross entropy of the ten classes trains it under AdamW, one
hundred and twenty-eight digits to a batch of a shuffled epoch, and six epochs of a rate of 1e-3 that
falls to 3e-4 and 1e-4. The ten thousand digits of the test set then say how well it reads what it
never saw, and the console carries the loss of every epoch beside the percent it read right.

## The first run

The first run downloads the four archives of MNIST (about 11 MB) into `target/mnist`, learns the
digits for about twenty seconds, and writes the learning to `target/mnist/lenet.neura`, both behind a
loading screen of a progress bar, a percent display and the step it reads. Every run after that finds
the learning on disk and goes straight to the box; delete the file to learn the digits again.

## What the reading reads

The box holds the strokes of the pen, and a reading takes their ink bounding box, scales it into a
box of twenty by twenty pixels of its own, shifts it so that its ink weighs the same about the centre
of a frame of twenty-eight by twenty-eight pixels, and hands that frame to the network — the
normalisation of MNIST itself. The reading draws the strokes again rather than the picture of them, so
it reads the shape a pen left instead of its width on screen.

## Notes

* the first run needs `curl` and a way to the archives at `storage.googleapis.com`, with
  `ossci-datasets.s3.amazonaws.com` behind it;
* the device needs a Vulkan, DX12 or Metal adapter and 256 MB of heap;
* the questions and the order of the batches come from constant seeds, so every run learns the same
  digits and asks the same questions.
