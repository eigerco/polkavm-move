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
sudo apt install libpolly-19-dev lld-19 zstd libzstd-dev llvm-19 llvm-19-dev clang-19
```

```bash
# Fedora
dnf install llvm-devel
```

```bash
# MacOS
brew install llvm lld
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

### Details

We first fetch all git dependencies of the Move project described in the Move.toml file.
Then, we compile the sources of your Move project with the specified dependencies, first to move bytecode, then
to move stackless bytecode, then to LLVM IR, then use the LLVM backend to emit RISC-V object files. These are
then combined and then linked to a .polkavm file using the `polkatool` linker.

We assume the Move project will have one module with at least one `entry` function, and that no other module
contains `entry` functions (to avoid duplicate symbols in the executable). We further assume this `entry` function
takes a single argument, namely a Move Signer.
Pallet-revive expects the .polkavm files to have 2 exports: `deploy`
and `call`. These are generated during translation. The `call` function calls a `call_selector` function that
will contain a switch to call any `entry` function of the module, based on the keccak hash of the function name.
The owner account of the smart contract (the user that uploaded the code) is found using the `origin()` syscall.
This returns a H160, and we transform it into the 32 byte AccountId. This is passed to the chosen `entry` function
as signer argument (thus mapping the Polkadot AccountId one to one with a Move signer address).

### Pallet-revive integration

We have implemented the following syscalls in pallet-revive:

```rust
// Move syscalls
fn debug_print(ptr_to_type: u32, address_ptr: u32);
fn exists(address_ptr: u32, ptr_to_tag: u32) -> u32;
fn move_to(ptr_to_signer: u32, ptr_to_struct: u32, ptr_to_tag: u32) -> u32;
fn move_from(address_ptr: u32, remove: u32, ptr_to_tag: u32, is_mut: u32) -> u32;
fn release(ptr_to_signer: u32, ptr_to_struct: u32, ptr_to_tag: u32);
fn hash_hash2_256(ptr_to_buf: u32) -> u32;
fn hash_hash3_256(ptr_to_buf: u32) -> u32;
```

Furthermore, we hooked up the Move `abort` syscall to the pallet-revive `terminate` syscall.

### Global Storage

Move global storage is implemented as pallet storage. See `polkadot-sdk/substrate/frame/revive/src/move_storage.rs`.

## Basic usage

The main crates for this repo are:

- `move-to-polka` crate, which is the actual Move to PolkaVM compiler

### `move-to-polka` installation and usage

Install `move-to-polka` binary:

```bash
cargo install --path crates/move-to-polka
```

Compile the given move project (should contain Move.toml) into a PolkaVM module (`output/output.polkavm` by default):

```bash
move-to-polka examples/storage
```

## Running on pallet-revive

- First run all the tests in [polkavm-move](https://github.com/eigerco/polkavm-move) repo (this generates all the .polkavm files).
- Clone our fork of [polkadot-sdk](https://github.com/eigerco/polkadot-sdk).
- Run the node from within the clone: `RUST_LOG="error,sc_rpc_server=info,runtime::revive=debug" cargo run --release --bin substrate-node -- --dev`
- Log in to the Web GUI at https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer
- Go to the Extrinsics, choose 'revive'.
- Choose instantiateWithCode, with following settings
  - value: 12345
  - gasLimit refTime: 1000000000000
  - gasLimit proofSize: 500000
  - storageDepositLimit: 12345678901234567890
  - code (choose the crates/move-to-polka/output/storage/storage.polkavm file)
  - data: 0xfa1e1f30 (see [How to find the call selector](#how_to_find_the_call_selector))
- Check the logs for the H160 of the uploaded contract
- Choose 'call', fill in the H160 address of the contract, use same settings for the rest
- Observe the logs, see that the code is called. Output should look like this:

```
2025-07-31 11:40:21.022 DEBUG tokio-runtime-worker runtime::revive: move_byte_vec: MoveByteVector { ptr: 0x30558, capacity: 20, length: 18 }
2025-07-31 11:40:21.022 DEBUG tokio-runtime-worker runtime::revive: move_to called with address ptr: 0xFFFCFEA8, value ptr: 0xFFFCFB40, address: @7DA26DA5E784569AE3CD4C8558852C82D69FA904BD1A14611CD3FD15C79335D4, value: [2a, 0, 0, 0, 0, 0, 0, 0, 45, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, ca, fe, ba, be]
2025-07-31 11:40:21.023 DEBUG tokio-runtime-worker runtime::revive: exists: tag: [8c, af, 68, 33, 5d, 67, b0, 3b, e9, e9, 3e, 4b, 92, 6d, 56, 74, 9c, 8a, c5, ff, 13, d9, 40, 30, b5, 3f, ab, 61, b5, ea, 9d, fa] signer: @7DA26DA5E784569AE3CD4C8558852C82D69FA904BD1A14611CD3FD15C79335D4
2025-07-31 11:40:21.023 DEBUG tokio-runtime-worker runtime::revive: entry: Some(GlobalResourceEntry { data: BoundedVec([2a, 0, 0, 0, 0, 0, 0, 0, 45, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, ca, fe, ba, be], 800), borrow_count: 1, borrow_mut: false })
2025-07-31 11:40:21.023 DEBUG tokio-runtime-worker runtime::revive: Data copied to guest memory at address: 0xFFFE0000, length: 24
2025-07-31 11:40:21.023 DEBUG tokio-runtime-worker runtime::revive: move_byte_vec: MoveByteVector { ptr: 0xfffe0000, capacity: 18, length: 18 }
2025-07-31 11:40:21.023 DEBUG tokio-runtime-worker runtime::revive: move_from called with address ptr: 0xFFFCFF70, address: FFFE0018, value: [2a, 0, 0, 0, 0, 0, 0, 0, 45, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, ca, fe, ba, be], remove: 0, is_mut: 0
2025-07-31 11:40:21.024 DEBUG tokio-runtime-worker runtime::revive: move_byte_vec: MoveByteVector { ptr: 0x305c4, capacity: 20, length: 18 }
2025-07-31 11:40:21.024 DEBUG tokio-runtime-worker runtime::revive: release called with address ptr: 0xFFFCFF70, value ptr: 0xFFFCFB40, address: @7DA26DA5E784569AE3CD4C8558852C82D69FA904BD1A14611CD3FD15C79335D4, value: [2a, 0, 0, 0, 0, 0, 0, 0, 45, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, ca, fe, ba, be]
2025-07-31 11:40:21.024 DEBUG tokio-runtime-worker runtime::revive: Decremented borrow count for global at [d4, 35, 93, c7, 15, fd, d3, 1c, 61, 14, 1a, bd, 4, a9, 9f, d6, 82, 2c, 85, 58, 85, 4c, cd, e3, 9a, 56, 84, e7, a5, 6d, a2, 7d] with type StructTagHash([8c, af, 68, 33, 5d, 67, b0, 3b, e9, e9, 3e, 4b, 92, 6d, 56, 74, 9c, 8a, c5, ff, 13, d9, 40, 30, b5, 3f, ab, 61, b5, ea, 9d, fa])
2025-07-31 11:40:21.024 DEBUG tokio-runtime-worker runtime::revive: entry: GlobalResourceEntry { data: BoundedVec([2a, 0, 0, 0, 0, 0, 0, 0, 45, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, ca, fe, ba, be], 800), borrow_count: 0, borrow_mut: false }
```

### How to find the call selector

To call module::function (in the example below storage::store_then_borrow), take the first 4 bytes (8 hex chars) of the keccak 256 hash
of the module::function name.

```bash
echo -n 'storage::store_then_borrow' | keccak-256sum | cut -c -8
fa1e1f30
```

## Known limitations:

- Compiled Move code can not call external modules (as agreed), any dependencies need to be compiled in.

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

Depending on your distribution, you may need to set the following kernel parameters:

```
sudo sysctl -w kernel.apparmor_restrict_unprivileged_userns=0
sudo sysctl -w vm.unprivileged_userfaultfd=1
```

## History

This repository was forked from [anza-xyz/move](https://github.com/anza-xyz/move), which added Move support to Solana.

## About [Eiger](https://www.eiger.co)

We are engineers. We contribute to various ecosystems by building low-level implementations and core components. We believe in Move and in Polkadot and wanted to bring them together. Read more about this project on [our blog](https://www.eiger.co/blog/eiger-brings-move-to-polkadot).

Contact us at hello@eiger.co
Follow us on [X/Twitter](https://x.com/eiger_co)
