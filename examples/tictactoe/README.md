# tictactoe

A Q-network learns tic-tac-toe by playing itself, then takes on the player.

```sh
cargo run --release --example tictactoe
```

## Controls

* wait for the bar to fill; click any empty square to play X, and the computer answers as O;
* the board clears itself a moment after every game.

## Notes

* training fills the bar in about half a minute;
* the run is seeded, so every run learns the same way;
* the device needs a Vulkan, DX12 or Metal adapter.
