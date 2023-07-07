use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Name of the person to greet
    #[command(subcommand)]
    command: LynxCommand,
}

#[derive(Subcommand, Debug)]
enum LynxCommand {
    New { project_name: String },
}

fn run_new_command<S: AsRef<str>>(project_name: S) -> std::io::Result<()> {


    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        LynxCommand::New { project_name } => println!("Project name: `{project_name:?}`"),
    }
}
