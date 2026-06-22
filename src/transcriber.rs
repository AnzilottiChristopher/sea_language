use crate::parse_sea;
use std::{collections::HashMap, path::PathBuf};
use tree_sitter::{Node, Tree};

struct ClassInfo {
    name: String,
    fields: Vec<(String, String)>,
    parent: Option<String>,
    methods: Vec<String>,
    constructor_params: Vec<(String, String)>,
}

fn collect_class_info(node: &Node, source: &String) -> ClassInfo {
    let name_node = node.child_by_field_name("name").unwrap();
    let name = source[name_node.start_byte()..name_node.end_byte()].to_string();

    let parent = node
        .child_by_field_name("inherit")
        .and_then(|inherit| inherit.child_by_field_name("parent"))
        .map(|parent| source[parent.start_byte()..parent.end_byte()].to_string());

    let mut fields = Vec::new();
    let mut methods = Vec::new();
    let mut constructor_params: Vec<(String, String)> = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "field_declaration" => {
                let type_node = child.child_by_field_name("type").unwrap();
                let name_node = child.child_by_field_name("name").unwrap();
                let type_text =
                    transpile_type(&source[type_node.start_byte()..type_node.end_byte()]);
                let name_text = source[name_node.start_byte()..name_node.end_byte()].to_string();
                fields.push((type_text, name_text));
            }
            "method_declaration" => {
                let method_node = child.child(0).unwrap();
                let name_node = method_node.child_by_field_name("name").unwrap();
                let method_name =
                    source[name_node.start_byte()..name_node.end_byte()].to_string();
                methods.push(method_name);
            }
            "init_declaration" => {
                // collect constructor params — no name matching needed
                if let Some(params_node) = child.child_by_field_name("parameters") {
                    let mut cursor2 = params_node.walk();
                    for param in params_node.children(&mut cursor2) {
                        if param.kind() == "sea_parameter" {
                            let type_node = param.child_by_field_name("type").unwrap();
                            let name_node = param.child_by_field_name("name").unwrap();
                            let type_text = transpile_type(
                                &source[type_node.start_byte()..type_node.end_byte()],
                            );
                            let name_text = source
                                [name_node.start_byte()..name_node.end_byte()]
                                .to_string();
                            constructor_params.push((type_text, name_text));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    ClassInfo {
        name,
        fields,
        methods,
        constructor_params,
        parent,
    }
}

pub fn analyze(
    tree: Tree,
    source: &String,
    current_file: &PathBuf,
    imported_c_files: &mut Vec<PathBuf>,
) -> String {
    let root = tree.root_node();
    let mut output = String::new();
    let mut class_table: HashMap<String, ClassInfo> = HashMap::new();

    // pass 1 — collect all class info
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "class_declaration" {
            let info = collect_class_info(&child, source);
            class_table.insert(info.name.clone(), info);
        } else if child.kind() == "import_declaration" {
            let path_node = child.child_by_field_name("path").unwrap();
            let path_text = &source[path_node.start_byte()..path_node.end_byte()];
            let path_str = path_text.trim_matches('"');
            let current_dir = current_file.parent().unwrap_or(std::path::Path::new("."));
            let imported_path = current_dir.join(format!("{}.sea", path_str));

            if imported_path.exists() {
                let (imported_tree, imported_source) = parse_sea(&imported_path);
                let imported_root = imported_tree.root_node();
                let mut cursor2 = imported_root.walk();
                for imported_child in imported_root.children(&mut cursor2) {
                    if imported_child.kind() == "class_declaration" {
                        let info = collect_class_info(&imported_child, &imported_source);
                        class_table.insert(info.name.clone(), info);
                    }
                }
            }
        }
    }

    // pass 2 — transpile
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        match child.kind() {
            "class_declaration" => {
                output.push_str(&transpile_class(&child, source, &class_table));
            }
            "interface_declaration" => {
                let name_node = child.child_by_field_name("name").unwrap();
                let name = &source[name_node.start_byte()..name_node.end_byte()];
                output.push_str(&format!("/* interface {name} */\n"));
            }
            "main_declaration" => {
                output.push_str(&transpile_main(&child, source, &class_table));
            }
            "import_declaration" => {
                output.push_str(&transpile_import(
                    &child,
                    source,
                    current_file,
                    imported_c_files,
                ));
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

fn transpile_import(
    node: &Node,
    source: &String,
    current_file: &std::path::Path,
    imported_c_files: &mut Vec<PathBuf>,
) -> String {
    let path_node = node.child_by_field_name("path").unwrap();
    let path_text = &source[path_node.start_byte()..path_node.end_byte()];
    let path_str = path_text.trim_matches('"');

    let current_dir = current_file.parent().unwrap_or(std::path::Path::new("."));
    let imported_sea_path = current_dir.join(format!("{}.sea", path_str));
    let imported_c_path = current_dir.join(format!("{}.c", path_str));
    let imported_h_path = current_dir.join(format!("{}.h", path_str));

    if !imported_sea_path.exists() {
        eprintln!("Error: cannot find imported file {:?}", imported_sea_path);
        std::process::exit(1);
    }

    let (imported_tree, imported_source) = parse_sea(&imported_sea_path);
    let header = generate_header(&imported_tree, &imported_source, path_str);
    let imported_c = analyze(
        imported_tree,
        &imported_source,
        &imported_sea_path,
        imported_c_files,
    );

    imported_c_files.push(imported_c_path.clone());
    std::fs::write(&imported_c_path, &imported_c).unwrap();
    std::fs::write(&imported_h_path, &header).unwrap();

    format!("#include \"{}.h\"\n", path_str)
}

fn generate_header(
    imported_tree: &tree_sitter::Tree,
    imported_source: &String,
    module_name: &str,
) -> String {
    let guard = module_name
        .to_uppercase()
        .replace("/", "_")
        .replace("-", "_")
        .replace(".", "_")
        .trim_start_matches('_')
        .to_string();
    let mut output = String::new();

    output.push_str(&format!("#ifndef {}_H\n", guard));
    output.push_str(&format!("#define {}_H\n\n", guard));

    let root = imported_tree.root_node();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "class_declaration" {
            let name_node = child.child_by_field_name("name").unwrap();
            let class_name = &imported_source[name_node.start_byte()..name_node.end_byte()];

            output.push_str(&format!("typedef struct {class_name} {class_name};\n"));
            output.push_str(&format!("struct {class_name} {{\n"));

            let mut cursor2 = child.walk();
            for member in child.children(&mut cursor2) {
                if member.kind() == "field_declaration" {
                    let type_node = member.child_by_field_name("type").unwrap();
                    let name_node = member.child_by_field_name("name").unwrap();
                    let type_text =
                        &imported_source[type_node.start_byte()..type_node.end_byte()];
                    let name_text =
                        &imported_source[name_node.start_byte()..name_node.end_byte()];
                    let type_text = transpile_type(type_text);
                    output.push_str(&format!("    {type_text} {name_text};\n"));
                }
            }
            output.push_str("};\n\n");

            let mut cursor3 = child.walk();
            for member in child.children(&mut cursor3) {
                match member.kind() {
                    "init_declaration" => {
                        // no name matching needed — init is always the constructor
                        let params_str = match member.child_by_field_name("parameters") {
                            Some(p) => transpile_params(&p, imported_source),
                            None => String::new(),
                        };
                        let params_part = if params_str.is_empty() {
                            String::new()
                        } else {
                            format!(", {params_str}")
                        };
                        output.push_str(&format!(
                            "void {class_name}_init({class_name} *self{params_part});\n"
                        ));
                    }
                    "method_declaration" => {
                        let method_node = member.child(0).unwrap();
                        if let Some(method_name_node) = method_node.child_by_field_name("name") {
                            let method_name = &imported_source
                                [method_name_node.start_byte()..method_name_node.end_byte()];
                            output.push_str(&format!(
                                "void {class_name}_{method_name}({class_name} *self);\n"
                            ));
                        }
                    }
                    _ => {}
                }
            }

            output.push_str("\n");
        }
    }

    output.push_str(&format!("#endif // {}_H\n", guard));
    output
}

fn transpile_main(
    node: &Node,
    source: &String,
    class_table: &HashMap<String, ClassInfo>,
) -> String {
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
        Some(n) => transpile_body(&n, source, 1, "", class_table),
        None => String::new(),
    };

    format!("int main({params_part}) {{\n{body}}}\n")
}

fn transpile_class(
    node: &Node,
    source: &String,
    class_table: &HashMap<String, ClassInfo>,
) -> String {
    let mut output = String::new();
    let mut methods = String::new();

    let name_node = node.child_by_field_name("name").unwrap();
    let name = &source[name_node.start_byte()..name_node.end_byte()];

    output.push_str(&format!("typedef struct {name} {name};\n"));
    output.push_str(&format!("struct {name} {{\n"));

    // emit inherited fields first
    if let Some(class_info) = class_table.get(name) {
        if let Some(parent_name) = &class_info.parent {
            if let Some(parent_info) = class_table.get(parent_name) {
                for (field_type, field_name) in &parent_info.fields {
                    output.push_str(&format!(
                        "    {field_type} {field_name}; /* inherited from {parent_name} */\n"
                    ));
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "field_declaration" => {
                output.push_str(&transpile_field(&child, source, 1));
            }
            "init_declaration" => {
                // no name matching needed — init is always the constructor
                methods.push_str(&transpile_constructor(&child, source, name, class_table));
            }
            "method_declaration" => {
                methods.push_str(&transpile_methods(&child, source, name, class_table));
            }
            "drop_declaration" => {
                methods.push_str(&transpile_drop(&child, source, name, class_table));
            }
            _ => {}
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

fn transpile_constructor(
    node: &Node,
    source: &String,
    class_name: &str,
    class_table: &HashMap<String, ClassInfo>,
) -> String {
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
        Some(n) => transpile_body(&n, source, 1, class_name, class_table),
        None => String::new(),
    };

    // call parent constructor with only the params parent needs
    let parent_init = if let Some(class_info) = class_table.get(class_name) {
        if let Some(parent_name) = &class_info.parent {
            if let Some(parent_info) = class_table.get(parent_name) {
                if parent_info.constructor_params.is_empty() {
                    format!("    {parent_name}_init(({parent_name}*)self);\n")
                } else {
                    let forwarded = parent_info
                        .constructor_params
                        .iter()
                        .map(|(_, name)| name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("    {parent_name}_init(({parent_name}*)self, {forwarded});\n")
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    format!("void {class_name}_init({class_name} *self{params_part}) {{\n{parent_init}{body}}}\n")
}

fn transpile_methods(
    node: &Node,
    source: &String,
    class_name: &str,
    class_table: &HashMap<String, ClassInfo>,
) -> String {
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
        Some(n) => transpile_body(&n, source, 1, class_name, class_table),
        None => String::new(),
    };

    format!("{return_type} {class_name}_{name}({class_name} *self{params_part}) {{\n{body}}}\n")
}

fn transpile_drop(
    node: &Node,
    source: &String,
    class_name: &str,
    class_table: &HashMap<String, ClassInfo>,
) -> String {
    let body = match node.child_by_field_name("body") {
        Some(n) => transpile_body(&n, source, 1, class_name, class_table),
        None => String::new(),
    };
    format!("void {class_name}_drop({class_name} *self) {{\n{body}}}\n")
}

fn transpile_body(
    node: &Node,
    source: &String,
    level: usize,
    class_name: &str,
    class_table: &HashMap<String, ClassInfo>,
) -> String {
    let mut output = String::new();
    let mut cursor = node.walk();
    let mut var_types: HashMap<String, String> = HashMap::new();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "{" | "}" => {}
            "expression_statement" => {
                output.push_str(&transpile_expression_statement(
                    &child,
                    source,
                    level,
                    class_name,
                    &var_types,
                    class_table,
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
    class_table: &HashMap<String, ClassInfo>,
) -> String {
    let mut output = String::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "assignment_expression" => {
                output.push_str(&transpile_assignment(&child, source));
            }
            "call_expression" => {
                output.push_str(&transpile_call(
                    &child,
                    source,
                    class_name,
                    var_types,
                    class_table,
                ));
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
        "call_expression" => source[node.start_byte()..node.end_byte()].to_string(),
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
    class_table: &HashMap<String, ClassInfo>,
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
                let actual_class = var_types
                    .get(obj_text)
                    .map(|s| s.as_str())
                    .unwrap_or(obj_text);

                let owner_class = find_method_owner(method_text, actual_class, class_table);

                let args_text = transpile_args(&args, source);
                let args_part = if args_text.is_empty() {
                    String::new()
                } else {
                    format!(", {args_text}")
                };
                if owner_class != actual_class {
                    format!("{owner_class}_{method_text}(({owner_class}*)&{obj_text}{args_part})")
                } else {
                    format!("{owner_class}_{method_text}(&{obj_text}{args_part})")
                }
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

fn find_method_owner<'a>(
    method_name: &str,
    class_name: &'a str,
    class_table: &'a HashMap<String, ClassInfo>,
) -> &'a str {
    if let Some(class_info) = class_table.get(class_name) {
        if class_info.methods.contains(&method_name.to_string()) {
            return class_name;
        }
        if let Some(parent_name) = &class_info.parent {
            return find_method_owner(method_name, parent_name, class_table);
        }
    }
    class_name
}

fn indent(level: usize) -> String {
    "    ".repeat(level)
}
