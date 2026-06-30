use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize, Serialize)]
pub struct ProjectConfig {
    pub project: Project,
    #[serde(default)]
    pub build: Build,
    pub lighthouse: Lighthouse,
    pub dependencies: Option<Dependencies>,
}

#[derive(Deserialize, Serialize)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub authors: Option<Vec<String>>,
    pub src: String,
    pub main: String,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Build {
    pub out_dir: Option<String>,
    pub cc: Option<String>,
    pub cflags: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize)]
pub struct Lighthouse {
    pub enabled: bool,
    pub strict: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub struct Dependencies {}

/// Scaffolds a new sea.toml (and src/main.sea) in `dir`, using `name` as the
/// project name. Returns an error if a sea.toml already exists in `dir`.
pub fn init_project(name: &str, dir: &Path) -> std::io::Result<()> {
    let toml_path = dir.join("sea.toml");

    if toml_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "sea.toml already exists in this directory",
        ));
    }

    let config = ProjectConfig {
        project: Project {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            authors: None,
            src: "src".to_string(),
            main: "src/main.sea".to_string(),
        },
        build: Build::default(),
        lighthouse: Lighthouse {
            enabled: true,
            strict: Some(false),
        },
        dependencies: None,
    };

    let toml_str = toml::to_string_pretty(&config).expect("failed to serialize ProjectConfig");

    std::fs::write(&toml_path, toml_str)?;

    let src_dir = dir.join("src");
    std::fs::create_dir_all(&src_dir)?;
    std::fs::write(src_dir.join("main.sea"), "// entry point\n")?;

    write_gitignore(dir)?;

    Ok(())
}

/// Loads and parses a sea.toml from the given path.
pub fn load(toml_path: &Path) -> Result<ProjectConfig, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(toml_path)?;
    let config: ProjectConfig = toml::from_str(&contents)?;
    Ok(config)
}

fn write_gitignore(dir: &Path) -> std::io::Result<()> {
    use std::io::Write;

    let gitignore_path = dir.join(".gitignore");
    let entry = "/build\n";

    if gitignore_path.exists() {
        let contents = std::fs::read_to_string(&gitignore_path)?;
        if !contents.contains("/build") {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&gitignore_path)?;
            file.write_all(entry.as_bytes())?;
        }
    } else {
        std::fs::write(&gitignore_path, entry)?;
    }

    Ok(())
}
