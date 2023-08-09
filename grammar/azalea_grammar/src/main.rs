use pest;

use pest_derive::Parser;

use pest::Parser;
use pest_ascii_tree::*;

#[derive(Parser)]
#[grammar = "formal_grammar.pest"]
struct AzaleaParser;

fn main() -> anyhow::Result<()> {
    // Read formal grammar spec from disk at compile time
    let content = include_str!("../grammar_test.txt");

    println!("{content}");
    
    // Test the grammar out on the test source file to see if the grammar works
    let tokens = AzaleaParser::parse(Rule::source_file, content)?;

    let ascii_tree = into_ascii_tree(tokens)?;
        
    println!("{ascii_tree}");

    Ok(())
}
