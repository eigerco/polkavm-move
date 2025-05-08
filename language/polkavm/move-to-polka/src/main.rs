// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use clap::Parser;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use move_to_polka::{initialize_logger, options::Options, run_to_polka};

fn main() -> anyhow::Result<()> {
    initialize_logger();
    let options = Options::parse();
    let color = if atty::is(atty::Stream::Stderr) && atty::is(atty::Stream::Stdout) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };
    let mut error_writer = StandardStream::stderr(color);
    run_to_polka(&mut error_writer, options)
}
