use std::collections::HashMap;
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
            "interface_declaration" => {
                //TODO
            }
            "main_declaration" => {
                output.push_str(&transpile_main(&child, source));
            }
            _ => {
                let text = &source[child.start_byte()..child.end_byte()];
                output.push_str(text);
                output.push_str("\n");
            }
        }
    }

    output
}

// This function essentially just takes any main and converts it into c's main
fn transpile_main(node: &Node, source: &String) -> String {
    let params_str = match node.child_by_field_name("parameters") {
        Some(p) => transpile_params(&p, source),
        None => String::new(),
    };

    let params_part = if params_str.is_empty() {
        String::new()
    } else {
        format!(", {params_str}")
    };

    let body = match node.child_by_field_name("body") {
        Some(n) => transpile_body(&n, source, 1, ""),
        None => String::new(),
    };

    format!("int main({params_part}) {{\n{body}}}\n")
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
                        Some(b) => transpile_body(&b, source, 1, name),
                        None => String::new(),
                    };

                    methods.push_str(&format!(
                        "void {name}_{method_name}({name} *self{params_part}) {{\n{body}}}\n"
                    ));
                }
            }
            "method_declaration" => {
                methods.push_str(&transpile_methods(&child, source, name));
            }
            "drop_declaration" => {
                methods.push_str(&transpile_drop(&child, source, name));
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

    let type_text = transpile_type(type_text);

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
        Some(n) => transpile_body(&n, source, 1, class_name),
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
                Some(base_node) => {
                    let type_text =
                        source[base_node.start_byte()..base_node.end_byte()].to_string();
                    transpile_type(&type_text)
                }
                None => "void".to_string(),
            },
            None => "void".to_string(),
        },
        None => "void".to_string(),
    };

    let body = match method_node.child_by_field_name("body") {
        Some(n) => transpile_body(&n, source, 1, class_name),
        None => String::new(),
    };

    format!("{return_type} {class_name}_{name}({class_name} *self{params_part}) {{\n{body}}}\n")
}

fn transpile_drop(node: &Node, source: &String, class_name: &str) -> String {
    let body = match node.child_by_field_name("body") {
        Some(n) => transpile_body(&n, source, 1, class_name),
        None => String::new(),
    };
    format!("void {class_name}_drop({class_name} *self) {{\n{body}}}\n")
}

fn transpile_body(node: &Node, source: &String, level: usize, class_name: &str) -> String {
    let mut output = String::new();
    let mut cursor = node.walk();
    let mut var_types: HashMap<String, String> = HashMap::new();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "{" | "}" => {}
            "expression_statement" => {
                output.push_str(&transpile_expression_statement(
                    &child, source, level, class_name, &var_types,
                ));
            }
            "return_statement" => {
                output.push_str(&transpile_return_statement(&child, source, level));
            }
            "declaration" => {
                let mut cursor = child.walk();
                let new_node = child
                    .children(&mut cursor)
                    .find(|c| c.kind() == "init_declarator")
                    .and_then(|init| init.child_by_field_name("value"))
                    .filter(|val| val.kind() == "new_expression");

                match new_node {
                    Some(new_expr) => {
                        let type_node = child
                            .child_by_field_name("type")
                            .or_else(|| {
                                let mut cursor3 = child.walk();
                                child
                                    .children(&mut cursor3)
                                    .find(|c| c.kind() == "type_identifier")
                            })
                            .unwrap();
                        let type_text = &source[type_node.start_byte()..type_node.end_byte()];

                        let mut cursor2 = child.walk();
                        let init_node = child
                            .children(&mut cursor2)
                            .find(|c| c.kind() == "init_declarator")
                            .unwrap();
                        let var_name_node = init_node.child_by_field_name("declarator").unwrap();
                        let var_name =
                            &source[var_name_node.start_byte()..var_name_node.end_byte()];

                        // record type mapping
                        var_types.insert(var_name.to_string(), type_text.to_string());

                        let args_text = match new_expr.child_by_field_name("arguments") {
                            Some(args_node) => transpile_args(&args_node, source),
                            None => String::new(),
                        };
                        let args_part = if args_text.is_empty() {
                            String::new()
                        } else {
                            format!(", {args_text}")
                        };

                        output.push_str(&format!("{}{type_text} {var_name};\n", indent(level)));
                        output.push_str(&format!(
                            "{}{type_text}_init(&{var_name}{args_part});\n",
                            indent(level)
                        ));
                    }
                    None => {
                        let text = &source[child.start_byte()..child.end_byte()];
                        output.push_str(&format!("{}{text}\n", indent(level)));
                    }
                }
            }
            _ => {
                let text = &source[child.start_byte()..child.end_byte()];
                output.push_str(&format!("{}{text}\n", indent(level)));
            }
        }
    }
    output
}

fn transpile_return_statement(node: &Node, source: &String, level: usize) -> String {
    let mut output = String::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "return" => output.push_str("return "),
            ";" => output.push_str(";"),
            _ => {
                output.push_str(&transpile_expression(&child, source));
            }
        }
    }
    format!("{}{output}\n", indent(level))
}

fn transpile_expression_statement(
    node: &Node,
    source: &String,
    level: usize,
    class_name: &str,
    var_types: &HashMap<String, String>,
) -> String {
    let mut output = String::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "assignment_expression" => {
                output.push_str(&transpile_assignment(&child, source));
            }
            "call_expression" => {
                output.push_str(&transpile_call(&child, source, class_name, var_types));
            }
            ";" => output.push_str(";"),
            _ => {
                output.push_str(&transpile_expression(&child, source));
            }
        }
    }

    format!("{}{output}\n", indent(level))
}

fn transpile_assignment(node: &Node, source: &String) -> String {
    let left = node.child_by_field_name("left").unwrap();
    let right = node.child_by_field_name("right").unwrap();

    let left_text = transpile_expression(&left, source);
    let right_text = transpile_expression(&right, source);

    format!("{left_text} = {right_text}")
}

fn transpile_expression(node: &Node, source: &String) -> String {
    match node.kind() {
        "field_expression" => {
            let arg = node.child_by_field_name("argument").unwrap();
            let field = node.child_by_field_name("field").unwrap();
            let arg_text = &source[arg.start_byte()..arg.end_byte()];
            let field_text = &source[field.start_byte()..field.end_byte()];

            if arg_text == "this" {
                format!("self->{field_text}")
            } else {
                format!("{arg_text}.{field_text}")
            }
        }
        "call_expression" => {
            // call without class context — just copy as-is
            source[node.start_byte()..node.end_byte()].to_string()
        }
        "identifier" => source[node.start_byte()..node.end_byte()].to_string(),
        "string_literal" => source[node.start_byte()..node.end_byte()].to_string(),
        "number_literal" => source[node.start_byte()..node.end_byte()].to_string(),
        _ => source[node.start_byte()..node.end_byte()].to_string(),
    }
}

fn transpile_call(
    node: &Node,
    source: &String,
    class_name: &str,
    var_types: &HashMap<String, String>,
) -> String {
    let func = node.child_by_field_name("function").unwrap();
    let args = node.child_by_field_name("arguments").unwrap();

    match func.kind() {
        "field_expression" => {
            let obj = func.child_by_field_name("argument").unwrap();
            let method = func.child_by_field_name("field").unwrap();
            let obj_text = &source[obj.start_byte()..obj.end_byte()];
            let method_text = &source[method.start_byte()..method.end_byte()];

            if obj_text == "this" {
                let args_text = transpile_args(&args, source);
                let args_part = if args_text.is_empty() {
                    String::new()
                } else {
                    format!(", {args_text}")
                };
                format!("{class_name}_{method_text}(self{args_part})")
            } else {
                // look up actual class name from var_types
                let actual_class = var_types
                    .get(obj_text)
                    .map(|s| s.as_str())
                    .unwrap_or(obj_text);
                let args_text = transpile_args(&args, source);
                let args_part = if args_text.is_empty() {
                    String::new()
                } else {
                    format!(", {args_text}")
                };
                format!("{actual_class}_{method_text}(&{obj_text}{args_part})")
            }
        }
        _ => source[node.start_byte()..node.end_byte()].to_string(),
    }
}

fn transpile_args(node: &Node, source: &String) -> String {
    let mut args: Vec<String> = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "(" | ")" | "," => {}
            _ => {
                args.push(transpile_expression(&child, source));
            }
        }
    }
    args.join(", ")
}

fn transpile_params(node: &Node, source: &String) -> String {
    let mut params: Vec<String> = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "sea_parameter" {
            let type_node = child.child_by_field_name("type").unwrap();
            let name_node = child.child_by_field_name("name").unwrap();
            let type_text = &source[type_node.start_byte()..type_node.end_byte()];
            let type_text = transpile_type(type_text);
            let name_text = &source[name_node.start_byte()..name_node.end_byte()];

            params.push(format!("{type_text} {name_text}"));
        }
    }
    params.join(", ")
}

fn transpile_type(type_text: &str) -> String {
    match type_text {
        "String" => "char*".to_string(),
        _ => type_text.to_string(),
    }
}

fn indent(level: usize) -> String {
    "    ".repeat(level) // 4 spaces per level
}
