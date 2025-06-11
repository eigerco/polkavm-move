// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use clap::Parser;
use log::{debug, info};
use move_to_polka::{
    initialize_logger,
    linker::{create_colored_stdout, BuildOptions},
    run_to_polka,
};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Args {
    // path to Move source to compile
    pub source: String,
    #[arg(short, long, default_value = "output.polkavm")]
    // output file name
    pub output: String,
}

fn main() -> anyhow::Result<()> {
    initialize_logger();
    let options = Args::parse();
    let source = options.source.as_str();
    let output = options.output.as_str();
    info!("Compiling Move source: {source} to {output}");
    pub const MOVE_STDLIB_PATH: &str = env!("MOVE_STDLIB_PATH");
    debug!("Using Move standard library path: {MOVE_STDLIB_PATH}");

    let move_src = format!("{MOVE_STDLIB_PATH}/sources");
    let options = BuildOptions::new(output)
        .dependency(&move_src)
        .source(source)
        .address_mapping("std=0x1")
        .build();
    let mut color_writer = create_colored_stdout();
    run_to_polka(&mut color_writer, options)
}
