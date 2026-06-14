mod parser;
mod transcriber;

use clap::Parser;
use std::io::Write;
use std::{fs::File, path::PathBuf};

use crate::{parser::parse_sea, transcriber::analyze};

#[derive(Parser, Debug)]
#[command(name = "seac")] //TODO pick better command like cargo, javac, etc
#[command(about = "Traspiler for the Sea language")]

//TODO Make more commands that run the c code too
struct Cli {
    file: PathBuf,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let (sea_tree, source) = parse_sea(&cli.file);

    let output = analyze(sea_tree, &source);
    //TODO make the file path reflect the actual class
    let mut file = File::create("test.c")?;

    write!(file, "{}", output)
}
