<p align="center">
  <img src="docs/assets/neura-readme-banner-animated.svg" width="784" alt="Neura">
</p>

<p align="center">
  <strong>Build AI-native games in Bevy</strong>
</p>

<p align="center">
  <a href="https://github.com/formetaohy/bevy-neura/actions/workflows/ci.yml"><img src="https://github.com/formetaohy/bevy-neura/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI status"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/language-Rust-orange?logo=rust" alt="Rust"></a>
  <a href="https://bevy.org/"><img src="https://img.shields.io/badge/dynamic/toml?url=https://raw.githubusercontent.com/formetaohy/bevy-neura/main/Cargo.toml&amp;query=$.dependencies.bevy.version&amp;label=bevy&amp;color=blue&amp;logo=bevy" alt="Bevy"></a>
  <a href="https://crates.io/crates/bevy-neura"><img src="https://img.shields.io/crates/v/bevy-neura?label=bevy-neura&amp;color=blueviolet" alt="bevy-neura"></a>
  <a href="https://crates.io/crates/neura"><img src="https://img.shields.io/badge/dynamic/toml?url=https://raw.githubusercontent.com/formetaohy/bevy-neura/main/Cargo.toml&amp;query=$.dependencies.neura&amp;label=neura&amp;color=blue" alt="neura"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green.svg" alt="MIT license"></a>
</p>

<table align="center">
  <tbody>
    <tr>
      <td align="center" valign="top">
        <a href="examples/mnist/README.md"><img src="docs/assets/mnist.gif" width="372" alt="mnist example"></a><br>
        <a href="examples/mnist/README.md"><strong>mnist</strong></a><br>
        Handwritten digit recognition
      </td>
      <td align="center" valign="top">
        <a href="examples/speech/README.md"><img src="docs/assets/speech.gif" width="298" alt="speech example"></a><br>
        <a href="examples/speech/README.md"><strong>speech</strong></a><br>
        Live speech transcription
      </td>
    </tr>
  </tbody>
  <tbody>
    <tr>
      <td align="center" valign="top">
        <a href="examples/tictactoe/README.md"><img src="docs/assets/tictactoe.gif" width="308" alt="tictactoe example"></a><br>
        <a href="examples/tictactoe/README.md"><strong>tictactoe</strong></a><br>
        Play against the trained AI
      </td>
    </tr>
  </tbody>
</table>

## What is bevy-neura?

**bevy-neura** is built on **[Neura](https://github.com/formetaohy/Neura)**, using **DX12, Metal, and Vulkan** for neural network training and inference, enabling you to build **AI-native games** in **[Bevy](https://bevy.org)**.

## Features

- **Game-loop Integration**: Run training and inference directly from your frame loop, without async/await.
- **Cross-platform Deployment**: Use your game's native graphics API directly — Neura runs right alongside your game.
- **Kernel Compiler**: Write custom kernels in Rust; Neura generates SPIR-V, HLSL, and MSL, so you do not maintain separate shader implementations.
- **High Performance**: End-to-end optimization for fast training and inference with a small memory footprint.

## Documents

- **[Examples](examples)**: See how to integrate AI into your game.

## Contact

Email: formetaohy@gmail.com

## License

[MIT](LICENSE)
