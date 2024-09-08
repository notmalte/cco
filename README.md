# cco

**cco** is a (very limited) x86_64 C compiler. See [/assets](./assets/) for C source files that it can compile. It relies on `gcc` for preprocessing and linking.

```
$ ./cco -h

Usage: cco [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to the C source file

Options:
      --lex       Only run the lexer
      --parse     Only run the lexer and parser
      --codegen   Only run the lexer, parser, and code generator, but stop before emitting assembly
  -S, --assembly  Emit assembly code, but do not link
  -h, --help      Print help
  -V, --version   Print version
```
