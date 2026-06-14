mod parser;
mod transcriber;

use clap::Parser;
use clap::Subcommand;
use std::path::PathBuf;

use crate::{parser::parse_sea, transcriber::analyze};

#[derive(Parser, Debug)]
#[command(name = "seac")] //TODO pick better command like cargo, javac, etc
#[command(about = "Transpiler for the Sea language")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Build { file: PathBuf },
    Compile { file: PathBuf },
    Run { file: PathBuf },
    Check { file: PathBuf },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build { file } => {
            let (sea_tree, source) = parse_sea(&file);
            let output = analyze(sea_tree, &source);
            let output_path = file.with_extension("c");
            std::fs::write(&output_path, output).unwrap();
            println!("Transpiled to {}", output_path.display());
        }
        Commands::Compile { file } => {
            // transpile first
            let (sea_tree, source) = parse_sea(&file);
            let output = analyze(sea_tree, &source);
            let c_path = file.with_extension("c");
            std::fs::write(&c_path, &output).unwrap();

            // then compile with gcc
            let bin_path = file.with_extension("");
            std::process::Command::new("gcc")
                .arg(&c_path)
                .arg("-o")
                .arg(&bin_path)
                .status()
                .expect("failed to run gcc");

            println!("Compiled to {}", bin_path.display());
        }
        Commands::Run { file } => {
            // transpile
            let (sea_tree, source) = parse_sea(&file);
            let output = analyze(sea_tree, &source);
            let c_path = file.with_extension("c");
            std::fs::write(&c_path, &output).unwrap();

            // compile
            let bin_path = file.with_extension("");
            std::process::Command::new("gcc")
                .arg(&c_path)
                .arg("-o")
                .arg(&bin_path)
                .status()
                .expect("failed to run gcc");

            // run
            std::process::Command::new(format!("./{}", bin_path.display()))
                .status()
                .expect("failed to run binary");
        }
        Commands::Check { file } => {
            // TODO — wire up Sea Checker
            println!("Checking {}...", file.display());
        }
    }
}
