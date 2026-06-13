use tree_sitter::{Node, Tree};

pub fn analyze(tree: Tree, source: &String) -> String {
    let root = tree.root_node();
    let mut output = String::new();

    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        match child.kind() {
            "class_declaration" => {
                output.push_str(&transpile_class(&child, source));
            }
            _ => {}
        }
    }

    output
}

fn transpile_class(node: &Node, source: &String) -> String {
    let mut output = String::new();
    let mut methods = String::new();

    let name_node = node.child_by_field_name("name").unwrap();
    let name = &source[name_node.start_byte()..name_node.end_byte()];

    output.push_str(&format!("typedef struct {name} {name};\n"));
    output.push_str(&format!("struct {name} {{\n"));

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "field_declaration" => {
                output.push_str(&transpile_field(&child, source, 1));
            }
            "constructor_declaration" => {
                methods.push_str(&transpile_constructor(&child, source, 0, name));
            }
            "method_declaration" => {
                //TODO Create function
                methods.push_str(&transpile_methods(&child, source, 0));
            }
            _ => {
                //TODO figure out why I am ignnoring everything else
            }
        }
    }

    output.push_str("};\n");
    output
}

fn transpile_field(node: &Node, source: &String, level: usize) -> String {
    let type_node = node.child_by_field_name("type").unwrap();
    let name_node = node.child_by_field_name("name").unwrap();

    let type_text = &source[type_node.start_byte()..type_node.end_byte()];
    let name_text = &source[name_node.start_byte()..name_node.end_byte()];

    format!("{}{type_text} {name_text};\n", indent(level))
}
fn transpile_constructor(node: &Node, source: &String, level: usize, class_name: &str) -> String {
    let params_str = match node.child_by_field_name("parameters") {
        Some(params_node) => transpile_params(&params_node, source),
        None => String::new(),
    };
    format!(
        "{}void {class_name}_init({class_name} *self, {params_str}) {{\n}}\n",
        indent(level)
    )
}
fn transpile_methods(node: &Node, source: &String, level: usize) -> String {
    format!("TODO")
}

fn transpile_params(node: &Node, source: &String) -> String {
    let mut params: Vec<String> = Vec::new();

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "sea_parameter" {
            let type_node = child.child_by_field_name("type").unwrap();
            let name_node = child.child_by_field_name("name").unwrap();

            let type_text = &source[type_node.start_byte()..type_node.end_byte()];
            let name_text = &source[name_node.start_byte()..name_node.end_byte()];

            params.push(format!("{type_text} {name_text}"));
        }
    }
    params.join(", ")
}

fn indent(level: usize) -> String {
    "    ".repeat(level) // 4 spaces per level
}
