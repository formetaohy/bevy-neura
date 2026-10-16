# mnist

Draw the answer to the question in the box; a LeNet-5 trained in the frame loop reads the digit.

```sh
cargo run --release --example mnist
```

## Controls

* draw in the box with the mouse or a finger; the box is read 0.6 s after the pen rests;
* a right answer brings the next question, a wrong one keeps it until the pen comes down again.

## First run

The first run downloads MNIST (about 11 MB, needs `curl`) into `target/mnist`, trains for some 20 s,
and saves `target/mnist/lenet.neura`; later runs load that file. Delete it to train again.
