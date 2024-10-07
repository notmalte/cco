# cco

**cco** is a (very limited) x86_64 C compiler. See [/assets](./assets/) for C source files that it can compile. It relies on `gcc` for preprocessing and linking.


## Example

```
$ cargo run -- ./assets/hello_world.c
$ ./assets/hello_world
Hello, World!
```


## Usage

```
$ ./cco -h

Usage: cco [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to the C source file

Options:
      --lex       Stop after lexing
      --parse     Stop after parsing
      --validate  Stop after semantic analysis
      --tacky     Stop after IR generation
      --codegen   Stop after code generation
  -S, --assembly  Emit assembly code, but do not link
  -c, --object    Emit object code, but do not link
  -h, --help      Print help
```
