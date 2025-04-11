![Move-on-PolkaVM](assets/polkavm-move-logo.png)

# Move Language support in PolkaVM

Move is a statically-typed programming language designed for safe and flexible smart contract development, with a strong focus on digital asset management.
It uses a resource-oriented model that enforces ownership and prevents assets from being accidentally copied or lost, making it ideal for secure blockchain applications.
Move was originally developed at Facebook.

PolkaVM is a lightweight virtual machine designed to execute smart contracts within the Substrate-based Polkadot ecosystem.
It serves as the execution layer for runtime logic and smart contracts on parachains, enabling decentralized applications while maintaining interoperability, security, and upgradeability across the Polkadot network.

This project adds support to execute smart contracts that are written in Move on PolkaVM.

## Getting started

This project relies heavly on [LLVM](https://llvm.org/) and just must install the necessary developer tools.

```
# MacOS
brew install llvm
```

```
# Fedora
dnf install llvm-devel
```

Even if llvm itself is written in C++, we use Rust and especially [llvm-sys](https://crates.io/crates/llvm-sys).

Install Rust
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

And build the move-to-polkavm tool

```
cargo build --release
```

## Architecture

On a high level, we use a stackless version of Move byte-code and compiles it down to Risc-V machine instructions.
Then, we use the polkavm linker to covert the elf file into a polkavm file.
These steps all happens offline.

The polkavm file can then be loaded an run inside a PolkaVM.

## History

This repository was forked from [anza-xyz/move](https://github.com/anza-xyz/move) that added Move support to Solana.
