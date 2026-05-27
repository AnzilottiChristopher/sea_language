/**
 * @file Sea is a systems programming language that extends C with object-oriented programming, modern syntax inspired by Java and Rust, and background ownership analysis. Valid C is always valid Sea.
 * @author Chris Anzilotti <anzilottichr@gmail.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: "sea",

  rules: {
    // TODO: add the actual grammar rules
    source_file: $ => "hello"
  }
});
