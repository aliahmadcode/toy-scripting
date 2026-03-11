use lox::lex::Lexer;
use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Tokenize { filename: PathBuf },
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Tokenize { filename } => {
            let contents = fs::read_to_string(filename)?;

            if !contents.is_empty() {
                let lexer = Lexer::new(&contents);
                for token in lexer {
                    println!("{:?}", token);
                }
            } else {
                eprintln!("EOF null");
            }
        }
    }

    Ok(())
}
