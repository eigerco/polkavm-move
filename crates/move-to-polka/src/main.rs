// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::collections::HashSet;

use clap::Parser;
use move_to_polka::{initialize_logger, linker::create_blob};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Args {
    // path to Move source to compile
    pub source: String,
    #[arg(short, long, default_value = "output/output.polkavm")]
    // output file name
    pub output: String,
}

fn main() -> anyhow::Result<()> {
    initialize_logger();
    let options = Args::parse();
    let source = options.source.as_str();
    let output = options.output.as_str();

    create_blob(output, source, HashSet::new())?;
    Ok(())
}
