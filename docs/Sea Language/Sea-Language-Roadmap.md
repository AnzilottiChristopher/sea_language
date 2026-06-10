# Sea Language — Build Roadmap

> A Java/C-style OOP language that transpiles to C, inherits Sea's ownership rules, and is a full superset of plain C.

**Tags:** #sea-lang #compiler #project

---

## How to use this file

- Each phase has tasks and a reading list
- Check off references as you read them with `- [x]`
- The phases are meant to be done in order — later phases depend on earlier ones
- Plain C is always valid Sea, so you can test the transpiler incrementally

---

## Phase 1 — Language Design

> Nail down what Sea looks like *before* writing a single line of transpiler code. Write 10–20 Sea programs by hand first. Decisions made here affect everything downstream.

### Tasks
- [x] Write a BNF/EBNF grammar spec for Sea syntax
- [x] Define how classes, methods, and fields look
- [x] Define how plain C passes through untouched
- [x] Document the `*` vs `&` distinction in the type system
- [x] Define `let` type inference rules
- [ ] Define `Option<T>` and `Result<T>` types
- [ ] Write 10–20 sample Sea programs by hand (classes, plain C, mixed)
- [ ] Decide on file extension convention (`.sea`)
- [ ]  Read Beej's Guide to C — `void*` and pointer casting section
- [ ]  Study `qsort` as a `void*` polymorphism example
- [ ]  Understand Java generics (type erasure)
- [ ]  Understand C++ templates (code generation per type)
- [ ]  Understand how transpile-to-C languages fake generics under the hood
- [x]  Decide: expose `void*` directly, hide behind abstraction, or add real generics
- [x]  Decide: what does `void` mean in Sea-style functions vs C-style functions

### References to read
- [x] [Crafting Interpreters — Representing Code (grammars)](https://craftinginterpreters.com/representing-code.html)
- [x] [EBNF on Wikipedia](https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form)
- [ ] [Tree-sitter grammar guide](https://tree-sitter.github.io/tree-sitter/creating-parsers) (2.2 where I left off)
- [ ] [Crafting Interpreters — Types of Values](https://craftinginterpreters.com/types-of-values.html)
- [ ] [Java OOP reference (for syntax inspiration)](https://docs.oracle.com/javase/tutorial/java/javaOO/index.html)
- [ ] [Rust structs (for syntax inspiration)](https://doc.rust-lang.org/book/ch05-00-structs.html)
- [ ] [Beej Guide for void*](https://beej.us/guide/bgc/)

---

## Phase 2 — Lexer & Parser

> Turn raw Sea source text into a structured AST your transpiler can reason about. This is the input stage. The hardest single step here is the parser — take your time with it.

### Tasks
- [x] Build the lexer (tokenise keywords, identifiers, symbols, literals)
- [x] Handle plain C tokens (no changes needed, C tokens are well-known)
- [x] Add Sea-specific tokens (`class`, `extends`, `let`, `match`, `own` if used)
- [ ] Design AST node types as Rust enums/structs
  - [ ] `ClassDecl` — class name, fields, methods, parent
  - [ ] `MethodDecl` — name, params, return type, body
  - [ ] `FieldDecl` — name, type, pointer kind (`*` or `&`)
  - [ ] `LetDecl` — name, inferred type, initialiser
  - [ ] `CPassthrough` — raw C text, pass verbatim
  - [ ] `ForEach`, `MatchExpr`, `LambdaExpr`
- [ ] Build the parser (recursive descent recommended)
- [ ] Handle operator precedence (Pratt parsing for expressions)
- [ ] Store source location (file, line, col) in every AST node — needed for error messages later

### References to read
- [ ] [Crafting Interpreters — Scanning (lexer)](https://craftinginterpreters.com/scanning.html)
- [ ] [Crafting Interpreters — Parsing Expressions](https://craftinginterpreters.com/parsing-expressions.html)
- [ ] [Pratt Parsing Explained (matklad)](https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html)
- [ ] [Rust enums for AST nodes](https://doc.rust-lang.org/book/ch06-00-enums.html)
- [ ] [Flex lexer manual (alternative to hand-written lexer)](https://westes.github.io/flex/manual/)
- [ ] [Rust token reference (for C/Sea token ideas)](https://doc.rust-lang.org/stable/reference/tokens.html)
- [ ] [Tree-sitter (alternative parser approach)](https://tree-sitter.github.io/tree-sitter/)

---
## Phase 2.5 — Formatter (Optional)
- Create the formatter that takes private/public into (nothing)/pub
- Format x=1+2 to x = 1 + 2
- etc

## Phase 3 — Semantic Analysis

> Walk the AST and check that the program makes sense — type checking, scope resolution, and Sea's ownership rules. This is where Sea's existing analyzer plugs in.

### Tasks
- [ ] Build a symbol table — track every variable, class, and method by scope
- [ ] Implement scope resolution (variables in inner scopes shadow outer ones)
- [ ] Implement type inference for `let` declarations
- [ ] Resolve class inheritance — which methods does `Dog` inherit from `Animal`?
- [ ] Integrate Sea's existing ownership analyzer
  - [ ] `*` pointers tracked as owning
  - [ ] `&` references tracked as borrows
  - [ ] Double free detection → error
  - [ ] Use after free detection → error
  - [ ] Leaked owning pointer → warning
- [ ] Emit diagnostics with original `.sea` source locations (not generated C lines)

### References to read
- [ ] [Crafting Interpreters — Resolving and Binding](https://craftinginterpreters.com/resolving-and-binding.html)
- [ ] [Symbol table on Wikipedia](https://en.wikipedia.org/wiki/Symbol_table)
- [ ] [Type inference walkthrough (Eli Bendersky)](https://eli.thegreenplace.net/2018/type-inference/)
- [ ] [Hindley-Milner type system](https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system)
- [ ] [Rust ownership model (nomicon)](https://doc.rust-lang.org/nomicon/ownership.html)
- [ ] [Clang static analysis developer docs](https://clang.llvm.org/docs/analyzer/developer-docs/DebugChecks.html)

---

## Phase 4 — C Code Generation

> Walk the AST and emit valid C. Plain C nodes pass through verbatim. Sea OOP constructs become vtable C. This is the core output of the transpiler.

### Tasks

#### C passthrough (do this first)
- [ ] Any AST node tagged `CPassthrough` emits its text verbatim
- [ ] Test: `seac file.c` should produce identical output to the input
- [ ] Test: `seac file.c -o program` should compile cleanly with gcc

#### Class → vtable codegen
- [ ] Emit a `typedef struct ClassName` for the class fields
- [ ] Emit a `typedef struct ClassNameVTable` for the method function pointers
- [ ] Emit a `ClassName_new()` constructor function
- [ ] Emit a `ClassName_destroy()` destructor
- [ ] Emit each method as a standalone C function `ClassName_methodName(ClassName* self, ...)`
- [ ] Wire the vtable struct with the correct function pointers in the constructor

#### Inheritance
- [ ] When `Dog extends Animal`, Dog's vtable includes all of Animal's function pointer slots
- [ ] Overridden methods replace the parent's slot; inherited methods copy the parent's pointer
- [ ] `this->vt->speak(this)` dispatch pattern generated for every method call

#### Syntax sugar rewrites
- [ ] `let x = expr` → infer type from expr, emit `InferredType x = expr`
- [ ] `for (n : arr)` → `for (int _i = 0; _i < arr.length; _i++) { int n = arr.data[_i]; ... }`
- [ ] `match (val) { case A => ... }` → switch/if-else chain
- [ ] `x => expr` lambda → emit a static C function + pointer
- [ ] `.map()`, `.filter()`, `.fold()` → call Sea stdlib functions
- [ ] `String + String` → call `sea_string_concat()`
- [ ] `arr.length` → emit `arr.len` field access
- [ ] `new ClassName(args)` → emit `ClassName_new(args)`

### References to read
- [ ] [Crafting Interpreters — Compiling Expressions](https://craftinginterpreters.com/compiling-expressions.html)
- [ ] [Cfront — the original C++ to C transpiler](https://en.wikipedia.org/wiki/Cfront)
- [ ] [GObject OOP in C (GTK)](https://docs.gtk.org/gobject/concepts.html)
- [ ] [Object-Oriented C — free book (PDF)](https://www.cs.rit.edu/~ats/books/ooc.pdf)
- [ ] [Virtual tables explained (Pablo Arias)](https://pabloariasal.github.io/2017/06/10/understanding-virtual-tables/)
- [ ] [C++ vtable deep dive (Eli Bendersky)](https://eli.thegreenplace.net/2012/12/17/dumping-a-c-objects-vtable)
- [ ] [Rust match expression reference](https://doc.rust-lang.org/reference/expressions/match-expr.html)
- [ ] [Syntactic sugar on Wikipedia](https://en.wikipedia.org/wiki/Syntactic_sugar)

---

## Phase 5 — Tooling & Standard Library

> Everything that makes Sea usable day-to-day. The stdlib is written in Sea itself — eating your own cooking.

### Tasks

#### `seac` compiler driver
- [ ] Accept `.sea` files and `.c` files interchangeably
- [ ] Run the transpiler on `.sea` files, pass `.c` files straight through
- [ ] Write the generated `.c` to a temp file
- [ ] Call gcc/clang on the temp file, passing through all flags (`-o`, `-O2`, etc.)
- [ ] Delete the temp file after compilation (unless `--keep-c` flag passed)
- [ ] Test: `seac main.c -o program` works identically to `gcc main.c -o program`
- [ ] Test: `seac main.sea main.c -o program` compiles mixed files correctly

#### Standard library — core types
- [ ] `String` — wraps `char*` + length, with `.length`, `.contains()`, `+` concat
- [ ] `List<T>` — dynamic array with `.add()`, `.get()`, `.length`, `.map()`, `.filter()`, `.fold()`
- [ ] `Map<K,V>` — hash map with `.put()`, `.get()`, `.contains()`
- [ ] `Stack<T>` — `.push()`, `.pop()`, `.peek()`
- [ ] `Queue<T>` — `.enqueue()`, `.dequeue()`
- [ ] `Option<T>` — `.some(val)`, `.none()`, `.isPresent()`, `??` null coalescing
- [ ] `Result<T>` — `.ok(val)`, `.err(msg)`, `.ok` field, `.value`, `.error`

#### Standard library — networking & I/O
- [ ] `File` — wraps `fopen`/`fread`/`fwrite`/`fclose`
- [ ] `TcpServer` — wraps `socket()`, `bind()`, `listen()`, `accept()`
- [ ] `TcpClient` — wraps `connect()`, `send()`, `recv()`

#### Error messages
- [ ] All errors display the original `.sea` file, line number, and column
- [ ] Errors from the ownership analyzer display the relevant variable name
- [ ] Never display the generated `.c` file path in user-facing errors

### References to read
- [ ] [Beej's Guide to Network Programming](https://beej.us/guide/bgnet/)
- [ ] [socket(2) man page](https://man7.org/linux/man-pages/man2/socket.2.html)
- [ ] [C string functions reference (cppreference)](https://en.cppreference.com/w/c/string/byte)
- [ ] [stb — single-file C libraries (good stdlib patterns)](https://github.com/nothings/stb)
- [ ] [GCC driver options](https://gcc.gnu.org/onlinedocs/gcc/Overall-Options.html)
- [ ] [Rust process::Command (for calling gcc from Rust)](https://doc.rust-lang.org/std/process/struct.Command.html)
- [ ] [Rustc diagnostics guide (error message design)](https://rustc-dev-guide.rust-lang.org/diagnostics.html)
- [ ] [Clang diagnostics design](https://clang.llvm.org/diagnostics.html)

---

## Language Design Decisions (reference)

Keep this section as a record of the choices made so far.

| Feature | Decision |
|---|---|
| OOP approach | Transpiler (Method 2) — Sea → C → binary |
| Plain C support | Full superset — any `.c` file is valid Sea |
| Pointer syntax | `Dog*` = owning pointer, `Dog&` = borrow/reference |
| Ownership enforcement | Silent background analysis — warnings, not mandatory annotations |
| Null safety | `?` suffix for nullable types, `??` for null coalescing |
| Type inference | `let` keyword, Hindley-Milner style |
| Error handling | `Result<T>` and `match`, not try/catch |
| Iteration | `for (n : arr)` foreach, `.map()` / `.filter()` / `.fold()` |
| Unsafe escape hatch | `unsafe { }` block allows pointer arithmetic |
| File extension | `.sea` for Sea files, `.c` still works |
| Compiler binary | `seac` — thin wrapper over transpiler + gcc/clang |

---

## Comparable projects (read for inspiration)

- [ ] [Vala language](https://vala.dev/) — Java-like language that transpiles to C, used in GNOME
- [ ] [Cfront history](https://en.wikipedia.org/wiki/Cfront) — original C++ to C transpiler, same architecture
- [ ] [Crafting Interpreters (full book, free online)](https://craftinginterpreters.com/) — the single best resource for this project
- [ ] [Rustc dev guide](https://rustc-dev-guide.rust-lang.org/) — how a real production compiler is structured
- [ ] [GObject reference manual](https://docs.gtk.org/gobject/) — OOP in C done at scale

---

For networks when we get there:
![[Pasted image 20260610000522.png]]


*Generated with Claude — Sea Language planning session*
