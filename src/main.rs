mod parser;
mod sea_toml_parser;
mod transcriber;
use crate::{parser::parse_sea, transcriber::analyze};
use clap::Parser;
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "sea")]
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
    Init,
    New { name: String },
}

fn run_lighthouse(file: &PathBuf) -> bool {
    let result = std::process::Command::new("lighthouse").arg(file).status();
    match result {
        Ok(status) if !status.success() => {
            eprintln!("Lighthouse found issues — fix them before compiling");
            false
        }
        Err(_) => {
            eprintln!("Warning: lighthouse not found — skipping checks");
            true // don't block if lighthouse not installed
        }
        _ => true,
    }
}

fn transpile(file: &PathBuf) -> (PathBuf, Vec<PathBuf>) {
    let (sea_tree, source) = parse_sea(file);
    let mut imported_c_files: Vec<PathBuf> = Vec::new();
    let output = analyze(sea_tree, &source, file, &mut imported_c_files);
    let c_path = file.with_extension("c");
    std::fs::write(&c_path, &output).unwrap();
    (c_path, imported_c_files)
}

fn compile(c_path: &PathBuf, imported_c_files: &Vec<PathBuf>) -> PathBuf {
    let bin_path = c_path.with_extension("");
    let mut cmd = std::process::Command::new("gcc");
    cmd.arg(c_path);
    for c_file in imported_c_files {
        cmd.arg(c_file);
    }
    cmd.arg("-o").arg(&bin_path);
    cmd.status().expect("failed to run gcc");
    bin_path
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build { file } => {
            let (c_path, _) = transpile(&file);
            println!("Transpiled to {}", c_path.display());
        }
        Commands::Compile { file } => {
            if !run_lighthouse(&file) {
                std::process::exit(1);
            }
            let (c_path, imported_c_files) = transpile(&file);
            let bin_path = compile(&c_path, &imported_c_files);
            println!("Compiled to {}", bin_path.display());
        }
        Commands::Run { file } => {
            if !run_lighthouse(&file) {
                std::process::exit(1);
            }
            let (c_path, imported_c_files) = transpile(&file);
            let bin_path = compile(&c_path, &imported_c_files);
            std::process::Command::new(format!("./{}", bin_path.display()))
                .status()
                .expect("failed to run binary");
        }
        Commands::Check { file } => {
            run_lighthouse(&file);
        }
        Commands::Init => {
            let cwd = std::env::current_dir().expect("failed to get current directory");
            let name = cwd
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("sea_project")
                .to_string();

            match sea_toml_parser::init_project(&name, &cwd) {
                Ok(_) => println!("Initialized sea.toml in current directory"),
                Err(e) => {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::New { name } => {
            let dir = PathBuf::from(&name);
            if dir.exists() {
                eprintln!("error: directory '{}' already exists", name);
                std::process::exit(1);
            }
            std::fs::create_dir_all(&dir).expect("failed to create project directory");

            match sea_toml_parser::init_project(&name, &dir) {
                Ok(_) => println!("Created new project '{}'", name),
                Err(e) => {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
