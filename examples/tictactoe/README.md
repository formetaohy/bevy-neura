# tictactoe

A Q-network learns tic-tac-toe by playing itself while the panel shows the training progress, win/draw splits and recent games; it then takes on the player.

```sh
cargo run --release --example tictactoe
```

## Controls

* wait for the training panel to finish; click any empty square to play X, and the computer answers as O;
* the board clears itself a moment after every game.

## Notes

* training fills the panel in about half a minute;
* the run is seeded, so every run learns the same way;
* the device needs a Vulkan, DX12 or Metal adapter.
