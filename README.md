<h1 align="center">
  <strong>bevy-neura</strong>
</h1>

<p align="center">
  <strong>Build AI-native games in Bevy</strong>
</p>

<p align="center">
  <a href="https://github.com/formetaohy/bevy-neura/actions/workflows/ci.yml"><img src="https://github.com/formetaohy/bevy-neura/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI status"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/language-Rust-orange?logo=rust" alt="Rust"></a>
  <a href="https://bevy.org/"><img src="https://img.shields.io/badge/bevy-0.19-blue?logo=bevy" alt="Bevy"></a>
  <a href="https://crates.io/crates/bevy-neura"><img src="https://img.shields.io/badge/bevy--neura-0.5.0-blueviolet" alt="bevy-neura"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green.svg" alt="MIT license"></a>
</p>

## What is bevy-neura?

**bevy-neura** is built on **[Neura](https://github.com/formetaohy/Neura)**, using **DX12, Metal, and Vulkan** for neural network training and inference, enabling you to build **AI-native games** in **[Bevy](https://bevy.org)**.

## Features

- **Game-loop Integration**: Run training and inference directly from your frame loop, without async/await.
- **Cross-platform Deployment**: Use your game's native graphics API directly — Neura runs right alongside your game.
- **Kernel Compiler**: Write custom kernels in Rust; Neura generates SPIR-V, HLSL, and MSL, so you do not maintain separate shader implementations.
- **High Performance**: End-to-end optimization for fast training and inference with a small memory footprint.

## Contact

Email: formetaohy@gmail.com

## License

[MIT](LICENSE)
