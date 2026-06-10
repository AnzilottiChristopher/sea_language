use tree_sitter::Parser;
use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_sea() -> *const ();
}

pub static LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_sea) };

fn main() {
    let language: tree_sitter::Language = LANGUAGE.into();

    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .expect("Error loading Sea grammar");

    let source = std::fs::read_to_string("docs/test.sea").unwrap();
    let tree = parser.parse(&source, None).unwrap();

    println!("{}", tree.root_node().to_sexp());
}
