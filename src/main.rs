use std::io::Write;
use std::{io::Read, path::Path};
use std::fs;

use fuzzer::{Fuzzer, XORShiftState};
use lexer::lexer::Lexer;
use parser::ast_parser::Parser as AstParser;
use parser::ast;
use preprocessor::preprocessor::Preprocessor;
use symbol_table::SymbolTable;
use symbol_table::{check_for_dup_funcs_syms, check_for_dup_choice_syms, check_for_dup_structs_syms};
use semantic_analyzer::check_for_missing_varbind;

use clap::Parser as ClapParser;


/// Azalea compiler (Dalton's capstone)
#[derive(ClapParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the path to the source file
    #[arg(short, long)]
    source_path: String,

    #[arg(long)]
    verbose_lex: bool,

    #[arg(long)]
    verbose_parse: bool
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

fn seralize_ast_to_path<P: AsRef<Path>>(ast: &ast::Program, path: P) -> anyhow::Result<()> {
    // Seralize AST into JSON 
    let serialized = serde_json::to_string_pretty(ast)?;

    // Open file for writing 
    let file = fs::OpenOptions::new()
	.write(true)
	.truncate(true)
	.open(path)?;

    // Create a `writer` using the above file settings
    let mut writer = std::io::BufWriter::new(file);

    // Write out serialized AST to disk
    writer.write_all(serialized.as_bytes())?;
    
    Ok(())
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
    let tokens = lexer.lex(args.verbose_lex)?;

    // Create `Parser` using the tokens
    let path   = std::path::Path::new(path);
    let parser = AstParser::new(tokens, path, cleaned_source.as_str());

    // Intialize `SymbolTable`
    let mut sym_table = SymbolTable::new();

    // Parse tokens into the abstract syntax tree with `parser`
    println!("[3/4] Parsing tokens...");
    let ast = parser.parse(args.verbose_parse, &mut sym_table)?;

    println!("{sym_table:#?}");

    check_for_dup_funcs_syms(&sym_table, path, cleaned_source.as_str())?;
    check_for_dup_choice_syms(&sym_table, path, cleaned_source.as_str())?;
    check_for_dup_structs_syms(&sym_table, path, cleaned_source.as_str())?;
    check_for_missing_varbind(&sym_table, &ast)?;

    // Seralize AST to disk for analysis (can be disabled!)
    #[cfg(feature = "serialize")]
    seralize_ast_to_path(&ast, "ast_dump/ast.json")?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    // Run the Morehead Azalea Compiler
    run_compiler()?;

    Ok(())
}
