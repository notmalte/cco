use clap::Parser;
use compiler::CompilerStage;

mod compiler;
mod linker;
mod preprocessor;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        help = "Path to the C source file",
    )]
    path: String,

    #[arg(
        long,
        group = "stage",
        conflicts_with = "assembly",
        help = "Only run the lexer"
    )]
    lex: bool,

    #[arg(
        long,
        group = "stage",
        conflicts_with = "assembly",
        help = "Only run the lexer and parser"
    )]
    parse: bool,

    #[arg(
        long,
        group = "stage",
        conflicts_with = "assembly",
        help = "Only run the lexer, parser, and code generator, but stop before emitting assembly"
    )]
    codegen: bool,

    #[arg(long, short = 'S', help = "Emit assembly code, but do not link")]
    assembly: bool,
}

fn main() {
    let args = Args::parse();

    let input_path = std::fs::canonicalize(&args.path).unwrap();
    assert!(input_path.is_file());

    let input_filename = input_path.file_name().unwrap().to_str().unwrap();
    assert!(input_filename.ends_with(".c"));

    let input_filename_stem = input_path.file_stem().unwrap().to_str().unwrap();

    let preprocessed_filename = format!("{}.i", input_filename_stem);
    let preprocessed_path = input_path.with_file_name(preprocessed_filename);

    let assembly_filename = format!("{}.s", input_filename_stem);
    let assembly_path = input_path.with_file_name(assembly_filename);

    let binary_path = input_path.with_file_name(input_filename_stem);

    preprocessor::preprocess(&input_path, &preprocessed_path);

    let stage = if args.lex {
        CompilerStage::Lex
    } else if args.parse {
        CompilerStage::Parse
    } else if args.codegen {
        CompilerStage::Codegen
    } else {
        CompilerStage::Full
    };

    compiler::compile(&preprocessed_path, &assembly_path, stage);
    std::fs::remove_file(&preprocessed_path).unwrap();

    if args.assembly || stage != CompilerStage::Full {
        return;
    }

    linker::link(&assembly_path, &binary_path);
    std::fs::remove_file(&assembly_path).unwrap();
}
