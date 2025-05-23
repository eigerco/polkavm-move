![Move-on-PolkaVM](assets/polkavm-move-logo.png)

# Move Language support in PolkaVM

Move is a statically-typed programming language designed for safe and flexible smart contract development, with a strong focus on digital asset management.
It uses a resource-oriented model that enforces ownership and prevents assets from being accidentally copied or lost, making it ideal for secure blockchain applications.
Move was originally developed at Facebook.

PolkaVM is a lightweight virtual machine designed to execute smart contracts within the Substrate-based Polkadot ecosystem.
It serves as the execution layer for runtime logic and smart contracts on parachains, enabling decentralized applications while maintaining interoperability, security, and upgradeability across the Polkadot network.

This project adds support to execute smart contracts written in Move on PolkaVM.

## Getting started

This project relies heavily on [LLVM](https://llvm.org/) and just requires installing the necessary developer tools.

```bash
# Ubuntu
sudo apt install libpolly-19-dev lld-19 libzstd-dev
```

```bash
# Fedora
dnf install llvm-devel
```

```bash
# MacOS
brew install llvm@19
```

Even though LLVM itself is written in C++, we use Rust, especially [llvm-sys](https://crates.io/crates/llvm-sys).

Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Build the `move-to-polka` tool:

```bash
cargo build --release
```

## Architecture

On a high level, we use a stackless version of Move byte-code and compile it down to RISC-V machine instructions.
Then, we use the PolkaVM linker to convert the ELF file into a PolkaVM file.
These steps all happen offline.

The PolkaVM file can then be loaded and executed inside a PolkaVM.

## Troubleshooting

If you get an error related to

```
error: No suitable version of LLVM was found system-wide or pointed
              to by LLVM_SYS_191_PREFIX
```

Try using

```bash
export LLVM_SYS_191_PREFIX="/usr/local/opt/llvm@19"
```

## Basic usage

The main crates for this repo are:

- `move-to-polka` crate, which is the actual Move to PolkaVM compiler
- `polkavm-wrapper` crate allows loading a compiled polkavm module and calls the provided "entry" function with the provided args for convenience.

### `move-to-polka` installation and usage

Install `move-to-polka` binary accessible from the terminal:

```bash
cargo install --path language/polkavm/move-to-polka
```

Compile the given move source file into a PolkaVM module (`output.polkavm` by default):

```bash
move-to-polka language/polkavm/examples/basic/sources/morebasic.move
```

### `polkavm-wrapper` installation and usage

Install `polkavm-wrapper` binary accessible from a terminal:

```bash
cargo install --path language/tools/polkavm-wrapper
```

Call the previously compiled module's entry function `sum` with the given args:

```bash
polkavm-wrapper -m output.polkavm -e sum -p 100 10
```

The expected output:

```bash
2025-05-06T06:55:13.223390Z  INFO polkavm_wrapper: Reading output.polkavm module
2025-05-06T06:55:13.223708Z  INFO polkavm_wrapper: 64bit module?: false
2025-05-06T06:55:13.223712Z  INFO polkavm_wrapper: Calling sum with args [100, 10]
2025-05-06T06:55:13.223791Z  INFO polkavm_wrapper: VM finished
2025-05-06T06:55:13.223793Z  INFO polkavm_wrapper: Result = 110
```

### Known limitations:

- No multi-module support, only SINGLE move module compilation is supported. An error will be generated if two modules in a single file are detected.
- Move project layout is not supported yet, only single Move file -> PolkaVM module compilation.
- No native function support yet (meaning that module compiles, but the polka linking phase will fail even with basic operations like division because it will emit abort native function call as part of the post-check).
- No `move-stdlib` yet (requires multi-module support).
- `polkavm-wrapper` assumes that all entry function args are `u64` (and therefore passed through two RISC-V 32-bit registers) and the return value is also `u64` (returned through two RISC-V 32-bit registers). This limitation can be lifted later when more complex data, such as args support, is added.

## History

This repository was forked from [anza-xyz/move](https://github.com/anza-xyz/move), which added Move support to Solana.

## About [Eiger](https://www.eiger.co)

We are engineers. We contribute to various ecosystems by building low-level implementations and core components. We believe in Move and in Polkadot and wanted to bring them together. Read more about this project on [our blog](https://www.eiger.co/blog/eiger-brings-move-to-polkadot).

Contact us at hello@eiger.co
Follow us on [X/Twitter](https://x.com/eiger_co)
