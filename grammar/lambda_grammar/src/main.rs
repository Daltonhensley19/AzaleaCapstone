use pest;

use pest_derive::Parser;

use pest::Parser;

#[derive(Parser)]
#[grammar = "formal_grammar.pest"]
struct LambdaParser;

fn main() -> anyhow::Result<()> {
    // Read formal grammar spec from disk at compile time
    let content = include_str!("../grammar_test.txt");

    println!("{content}");
    
    // Test the grammar out on the test source file to see if the grammar works
    let tokens = LambdaParser::parse(Rule::source_file, content)?;

    println!("{tokens:#?}");

    println!("Hello, world!");

    Ok(())
}
