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
                let con_name_node = child.child_by_field_name("name").unwrap();
                let con_name = &source[con_name_node.start_byte()..con_name_node.end_byte()];

                if con_name == name {
                    // name matches class — it's a real constructor
                    methods.push_str(&transpile_constructor(&child, source, name));
                } else {
                    // name doesn't match class — it's actually a method
                    // constructor_declaration has same fields as sea_style_method
                    // so we can transpile it directly without the wrapper
                    let method_name = con_name;
                    let params_str = match child.child_by_field_name("parameters") {
                        Some(p) => transpile_params(&p, source),
                        None => String::new(),
                    };
                    let params_part = if params_str.is_empty() {
                        String::new()
                    } else {
                        format!(", {params_str}")
                    };
                    let body = match child.child_by_field_name("body") {
                        Some(b) => transpile_body(&b, source, 1),
                        None => String::new(),
                    };

                    println!("GLR matched as constructor — name: {con_name}");
                    methods.push_str(&format!(
                        "void {name}_{method_name}({name} *self{params_part}) {{\n{body}}}\n"
                    ));
                }
            }
            "method_declaration" => {
                //TODO Create function
                methods.push_str(&transpile_methods(&child, source, name));
            }
            _ => {
                //TODO figure out why I am ignnoring everything else
            }
        }
    }

    output.push_str("};\n\n");
    output.push_str(&methods);
    output
}

fn transpile_field(node: &Node, source: &String, level: usize) -> String {
    let type_node = node.child_by_field_name("type").unwrap();
    let name_node = node.child_by_field_name("name").unwrap();

    let type_text = &source[type_node.start_byte()..type_node.end_byte()];
    let name_text = &source[name_node.start_byte()..name_node.end_byte()];

    format!("{}{type_text} {name_text};\n", indent(level))
}
fn transpile_constructor(node: &Node, source: &String, class_name: &str) -> String {
    let params_str = match node.child_by_field_name("parameters") {
        Some(params_node) => transpile_params(&params_node, source),
        None => String::new(),
    };
    let params_part = if params_str.is_empty() {
        String::new()
    } else {
        format!(", {params_str}")
    };

    let body = match node.child_by_field_name("body") {
        Some(n) => transpile_body(&n, source, 1),
        None => String::new(),
    };

    format!("void {class_name}_init({class_name} *self{params_part}) {{\n{body}}}\n")
}
fn transpile_methods(node: &Node, source: &String, class_name: &str) -> String {
    let method_node = node.child(0).unwrap();

    let name_node = method_node.child_by_field_name("name").unwrap();
    let name = &source[name_node.start_byte()..name_node.end_byte()];

    let param_str = match method_node.child_by_field_name("parameters") {
        Some(params_node) => transpile_params(&params_node, source),
        None => String::new(),
    };

    let params_part = if param_str.is_empty() {
        String::new()
    } else {
        format!(", {param_str}")
    };

    let return_type = match method_node.child_by_field_name("return_type") {
        Some(return_node) => match return_node.named_child(0) {
            Some(type_node) => match type_node.child_by_field_name("base") {
                Some(base_node) => source[base_node.start_byte()..base_node.end_byte()].to_string(),
                None => "void".to_string(),
            },
            None => "void".to_string(),
        },
        None => "void".to_string(),
    };

    let body = match method_node.child_by_field_name("body") {
        Some(n) => transpile_body(&n, source, 1),
        None => String::new(),
    };

    format!("{return_type} {class_name}_{name}({class_name} *self{params_part}) {{\n{body}}}\n")
}

fn transpile_body(node: &Node, source: &String, level: usize) -> String {
    let mut output = String::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "{" | "}" => {}
            _ => {
                let text = &source[child.start_byte()..child.end_byte()];
                let transformed = text.replace("this.", "self->");
                output.push_str(&format!("{}{transformed}\n", indent(level)));
            }
        }
    }
    output
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
