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
        [$.constructor_declaration, $.sea_style_method],
    ],

    rules: {
        translation_unit: ($) => repeat($._definitions),

        _definitions: ($) =>
            choice(
                $.class_declaration,
                $.interface_declaration,
                $.main_declaration,
                $.function_definition, // C functions
                $.declaration, // C variable declarations
                $.linkage_specification, // extern "C" etc
                $.preproc_include, // #include
                $.preproc_def,
            ),

        interface_declaration: ($) =>
            seq(
                "interface",
                $.identifier,
                "{",
                repeat($.interface_method),
                "}",
            ),
        interface_method: ($) =>
            seq(
                optional("pub"),
                $.identifier,
                "(",
                optional($.sea_parameter_list),
                ")",
                optional(seq("->", $.return_type)),
                choice(
                    $.compound_statement, // default implementation
                    ";", // no implementation — required by subclass
                ),
            ),

        main_declaration: ($) =>
            choice(
                // Sea style
                seq(
                    "main",
                    "(",
                    optional($.sea_parameter_list),
                    ")",
                    "->",
                    "int",
                    $.compound_statement,
                ),
                // C style — optional, might be handled by C grammar already
                seq(
                    "int",
                    "main",
                    "(",
                    optional($.sea_parameter_list),
                    ")",
                    $.compound_statement,
                ),
            ),
        implements_clause: ($) =>
            seq(
                "implements",
                seq($.identifier, repeat(seq(",", $.identifier))),
            ),
        inherit_clause: ($) => seq("inherit", $.identifier),
        _class_member: ($) =>
            choice(
                $.field_declaration,
                $.constructor_declaration,
                $.method_declaration,
                $.drop_declaration,
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
                optional($.sea_parameter_list),
                ")",
                $.compound_statement,
            ),
        drop_declaration: ($) =>
            seq(optional("pub"), "drop", "(", ")", $.compound_statement),
        method_declaration: ($) => choice($.sea_style_method, $.c_style_method),
        sea_style_method: ($) =>
            seq(
                optional("pub"),
                $.identifier,
                "(",
                optional($.sea_parameter_list),
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
                optional($.sea_parameter_list),
                ")",
                $.compound_statement,
            ),

        sea_parameter_list: ($) =>
            seq($.sea_parameter, repeat(seq(",", $.sea_parameter))),
        sea_parameter: ($) => seq($.type, $.identifier),

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
