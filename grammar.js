/**
 * @file Sea is a systems programming language that extends C with object-oriented programming, modern syntax inspired by Java and Rust, and background ownership analysis. Valid C is always valid Sea.
 * @author Chris Anzilotti <anzilottichr@gmail.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

import C from "tree-sitter-c/grammar.js";

export default grammar(C, {
    name: "sea",
    conflicts: ($) => [
        // from C grammar
        [$._type_specifier, $._declarator],
        [$._type_specifier, $._declarator, $.macro_type_specifier],
        [$._type_specifier, $._expression],
        [$._type_specifier, $._expression, $.macro_type_specifier],
        [$._type_specifier, $.macro_type_specifier],
        [$._type_specifier, $._expression_not_binary, $.macro_type_specifier],
        [$._type_specifier, $._expression_not_binary],
        [$._type_specifier, $.sized_type_specifier],
        [$.sized_type_specifier],
        [$.enum_specifier],
        [$.attributed_statement],
        [$._declaration_modifiers, $.attributed_statement],
        [$.function_declarator, $._function_declaration_declarator],
        [$.parameter_list, $._old_style_parameter_list],
        [$._type_specifier, $._old_style_parameter_list],
        // Sea specific
        [$.new_expression],
        [$.this_expression],
    ],

    rules: {
        translation_unit: ($) => repeat($._definitions),

        _definitions: ($) =>
            choice(
                $.class_declaration,
                $.interface_declaration,
                $.main_declaration,
                $.import_declaration,
                $.function_definition,
                $.declaration,
                $.linkage_specification,
                $.preproc_include,
                $.preproc_def,
            ),

        new_expression: ($) =>
            seq(
                "new",
                field("type", $.identifier),
                field("arguments", optional($.argument_list)),
            ),

        this_expression: ($) => "this",

        // @ts-ignore
        _expression: ($) =>
            choice(
                $.new_expression,
                $.this_expression,
                ...C.grammar.rules._expression.members,
            ),

        interface_declaration: ($) =>
            seq(
                "interface",
                field("name", $.identifier),
                "{",
                repeat($.interface_method),
                "}",
            ),

        import_declaration: ($) =>
            seq(
                "import",
                field(
                    "imports",
                    choice(
                        "*",
                        seq($.identifier, repeat(seq(",", $.identifier))),
                    ),
                ),
                "from",
                field("path", $.string),
                ";",
            ),

        interface_method: ($) =>
            seq(
                field("visibility", optional("pub")),
                field("name", $.identifier),
                "(",
                field("parameters", optional($.sea_parameter_list)),
                ")",
                field("return_type", optional(seq("->", $.return_type))),
                choice(field("body", $.compound_statement), ";"),
            ),

        main_declaration: ($) =>
            choice(
                // Sea style
                seq(
                    "main",
                    "(",
                    field("parameters", optional($.sea_parameter_list)),
                    ")",
                    "->",
                    "int",
                    field("body", $.compound_statement),
                ),
                // C style
                seq(
                    "int",
                    "main",
                    "(",
                    field("parameters", optional($.sea_parameter_list)),
                    ")",
                    field("body", $.compound_statement),
                ),
            ),

        implements_clause: ($) =>
            seq(
                "implements",
                field(
                    "interfaces",
                    seq($.identifier, repeat(seq(",", $.identifier))),
                ),
            ),

        inherit_clause: ($) => seq("inherit", field("parent", $.identifier)),

        _class_member: ($) =>
            choice(
                $.field_declaration,
                $.init_declaration,
                $.method_declaration,
                $.drop_declaration,
            ),

        class_declaration: ($) =>
            seq(
                "class",
                field("name", $.identifier),
                field("inherit", optional($.inherit_clause)),
                field("implements", optional($.implements_clause)),
                "{",
                field("body", repeat($._class_member)),
                "}",
            ),

        init_declaration: ($) =>
            seq(
                field("visibility", optional("pub")),
                "init",
                "(",
                field("parameters", optional($.sea_parameter_list)),
                ")",
                field("body", $.compound_statement),
            ),

        drop_declaration: ($) =>
            seq(
                field("visibility", optional("pub")),
                "drop",
                "(",
                ")",
                field("body", $.compound_statement),
            ),

        method_declaration: ($) => choice($.sea_style_method, $.c_style_method),

        sea_style_method: ($) =>
            seq(
                field("visibility", optional("pub")),
                field("name", $.identifier),
                "(",
                field("parameters", optional($.sea_parameter_list)),
                ")",
                optional(seq("->", field("return_type", $.return_type))),
                field("body", $.compound_statement),
            ),

        c_style_method: ($) =>
            seq(
                field("visibility", optional("pub")),
                field("return_type", $.type),
                field("name", $.identifier),
                "(",
                field("parameters", optional($.sea_parameter_list)),
                ")",
                field("body", $.compound_statement),
            ),

        sea_parameter_list: ($) =>
            seq($.sea_parameter, repeat(seq(",", $.sea_parameter))),

        sea_parameter: ($) =>
            seq(field("type", $.type), field("name", $.identifier)),

        field_declaration: ($) =>
            seq(
                field("visibility", optional("pub")),
                field("type", $.type),
                field("name", $.identifier),
                ";",
            ),

        base_type: ($) =>
            choice("int", "float", "double", "char", "String", $.identifier),

        type: ($) =>
            seq(
                field("base", $.base_type),
                field("modifier", optional(choice("*", "&"))),
            ),

        return_type: ($) => choice($.type, "void", seq("void", "*")),

        identifier: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,
        number: ($) => /[0-9]+/,
        string: ($) => /"[^"]*"/,
        comment: ($) => token(seq("//", /.*/)),
    },
});
