use std::path::PathBuf;
use tree_sitter::Parser;
use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_sea() -> *const ();
}

pub static LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_sea) };

pub fn parse_sea(path: &PathBuf) -> (tree_sitter::Tree, String) {
    let language: tree_sitter::Language = LANGUAGE.into();
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .expect("Error loading Sea grammar");
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|e| {
            eprintln!("Error reading {:?}: {}", path, e);
            std::process::exit(1);
        })
        .replace("\r\n", "\n");
    let tree = parser.parse(&source, None).unwrap();
    (tree, source)
}
