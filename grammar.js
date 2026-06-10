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

    rules: {
        source_file: ($) => repeat($._definitions),

        _definitions: ($) =>
            choice(
                $.class_declaration,
                $.struct_declaration,
                $.interface_declaration,
                $.main_declaration,
                $.translation_unit,
            ),

        class_declaration: ($) =>
            seq(
                "class",
                $.identifier,
                optional($.inherit_clause),
                optional($.implements_clause),
                "{",
                repeat($._class_member),
                "}",
            ),

        constructor_declaration: ($) =>
            seq(
                optional("pub"),
                $.identifier,
                "(",
                optional($.paramter_list),
                ")",
                $.compound_statement,
            ),
        method_declaration: ($) => choice($.sea_style_method, $.c_style_method),
        sea_style_method: ($) =>
            seq(
                optional("pub"),
                $.identifier,
                "(",
                optional($.paramter_list),
                ")",
                optional(seq("->", $.return_type)),
                $.compound_statement,
            ),
        c_style_method: ($) =>
            seq(
                optional("pub"),
                $.type,
                $.identifier,
                "(",
                optional($.paramter_list),
                ")",
                $.compound_statement,
            ),

        paramter_list: ($) => seq($.paramter, repeat(seq(",", $.paramter))),
        paramter: ($) => seq($.type, $.identifier),

        field_declaration: ($) =>
            seq(optional("pub"), $.type, $.identifier, ";"),

        base_type: ($) =>
            choice("int", "float", "double", "char", "String", $.identifier),
        type: ($) => seq($.base_type, optional(choice("*", "&"))),
        return_type: ($) => choice($.type, "void", seq("void", "*")),

        identifier: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,
        number: ($) => /[0-9]+/,
        string: ($) => /"[^"]*"/,
        comment: ($) => token(seq("//", /.*/)),
    },
});
