# Move bytecode to llvm bitcode compiler

Takes move bytecode in text or binary format

See: `move-to-polka --help` for cli.

Compile move-bytecode to llvm bitcode

## Overview

The move compiler uses llvm-sys to interface with llvm. It translates stackless bytecode representation of move to llvm-ir.
Read more about bytecode translations from [here](https://github.com/move-language/move/issues/817) and [here](https://brson.github.io/2023/03/12/move-on-llvm#challenges-of-porting-move)

The [docs](./docs) directory contains the documentation. Interesting links:

- [Move to llvm compiler](./docs/MoveToLLVM.md)
- [Project stages](./docs/MoveToLLVM.md#project-stages)

Developer documentation

- [Dependencies](./docs/Development.md#Dependencies)
- [Setup](./docs/Development.md#Setup)
- [Testing](./docs/Development.md#Testing)
- [Debugging](./docs/Development.md#Debugging)

### TODO

Create issues instead of having TODOs here.

## ACKNOWLEDGEMENTS

- The [ziglang codebase](https://git.sr.ht/~andrewrk/ziglang/tree/master) has examples of LLVM C API that has been helpful.
- Parts of [Aptos move](https://github.com/aptos-labs/aptos-core) and [Sui move](https://github.com/MystenLabs/sui) were very helpful.
