# cco

**cco** is a (very limited) x86_64 C compiler. See [/assets](./assets/) for C source files that it can compile. It relies on `gcc` for preprocessing and linking.

```
$ ./cco -h

Usage: cco [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to the C source file

Options:
      --lex       Only run lexer
      --parse     Only run lexer + parser
      --validate  Only run lexer + parser + semantic analysis
      --tacky     Only run lexer + parser + semantic analysis + tacky generator
      --codegen   Only run lexer + parser + semantic analysis + tacky generator + codegen
  -S, --assembly  Emit assembly code, but do not link
  -h, --help      Print help
  -V, --version   Print version
```
