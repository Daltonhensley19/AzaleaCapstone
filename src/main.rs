use std::{io::Read, path::Path};

use fuzzer::{Fuzzer, XORShiftState};
use lexer::lexer::Lexer;
use parser::ast_parser::Parser as AstParser;
use preprocessor::preprocessor::Preprocessor;

use clap::Parser as ClapParser;

/// Simple program to greet a person
#[derive(ClapParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the path to the source file
    #[arg(short, long)]
    source_path: String,
}

fn source_file_to_string<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    // Open file using with a buffer
    let file = std::fs::File::open(path)?;

    // Create the `BufReader` to read lines
    let mut reader = std::io::BufReader::new(&file);

    // Read file as string
    let mut content: String = String::with_capacity(reader.capacity());
    reader.read_to_string(&mut content)?;

    Ok(content)
}

fn run_fuzzer(source_content: String) -> String {
    // Create `Fuzzer` and load it with the source file
    let seed           = 2;
    let mut fuzzer     = Fuzzer::new(source_content, XORShiftState::new(seed));
    let source_content = fuzzer.fuzz();

    source_content
}

fn run_compiler() -> anyhow::Result<()> {

    let args = Args::parse();

    // Read source file content as a `String`
    let path: &str     = args.source_path.as_str();
    let source_content = source_file_to_string(path)?;
    

    // Fuzz the source code if "fuzz" feature is enabled
    #[cfg(feature = "fuzz")]
    let source_content = run_fuzzer(source_content);

    // Create `Preprocessor` and load it with the source file
    let preprocessor = Preprocessor::new(source_content, path)?;
    
    // Remove comments from source file and return a cleaned version
    println!("[1/4] Preprocessing source...");
    let cleaned_source = preprocessor
        .normalize_to_ascii()?
        .remove_multiline_comment()?
        .remove_singleline_comments()
        .get_cleaned_sources();

    // Create `Lexer`
    let mut lexer = Lexer::new(path, &cleaned_source);

    // Tokenize the source file; fail fast on error
    println!("[2/4] Tokenizing source...");
    let tokens = lexer.lex()?;

    // Create `Parser` using the tokens
    let path   = std::path::Path::new(path);
    let parser = AstParser::new(tokens, path, cleaned_source.as_str());

    // Parse tokens into the abstract syntax tree with `parser`
    println!("[3/4] Parsing tokens...");
    let ast = parser.parse();

    println!("{ast:#?}");
    

    Ok(())
}

fn main() -> anyhow::Result<()> {
    // Run the Morehead Lambda Compiler
    run_compiler()?;

    Ok(())
}
