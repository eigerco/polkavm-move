#!/usr/bin/env bash

set -euo pipefail


function build() {
    output_path="output/$1.a"

    echo "> Building: '$1' (-> $output_path)"

    #RUSTFLAGS="--remap-path-prefix=$(pwd)= --remap-path-prefix=$HOME=~" \
    cargo rustc \
        --crate-type=staticlib \
        -Z build-std=core,alloc \
        --target riscv32emac-unknown-none-polkavm.json \
        --release \
        -- -C codegen-units=1 -C opt-level=s
}

build "polka-move-native"