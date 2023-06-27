use std::{io::Read, path::Path};

use preprocessor::preprocessor::Preprocessor;
use lexer::lexer::Lexer;

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

fn main() -> anyhow::Result<()> {  
    // Read source file content as a `String`
    let path           = "source_test.txt";
    let source_content = source_file_to_string(path)?;

    // Create `Preprocessor` and load it with the source file
    let preprocessor = Preprocessor::new(source_content, path);

    // Remove comments from source file and return a cleaned version
    let cleaned_source = preprocessor
        .remove_multiline_comment()?
        .remove_singleline_comments()
        .get_cleaned_sources();

    // Create `Lexer`
    let mut lexer = Lexer::new(path, cleaned_source);

    // Tokenize the source file; fail fast on error
    let tokens = lexer.lex()?;

    // Parse tokens into the abstract syntax tree
    //let ast = parser.parse(&tokens);
    
    println!("{tokens:#?}");

    Ok(())
}
