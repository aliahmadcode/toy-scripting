use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} tokenize <filename>", args[0]);
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            eprintln!("logs from your program will appear here!");

            let contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("failed to read file {}!", filename);
                String::new()
            });

            if !contents.is_empty() {
                panic!("scanner not implemented!");
            } else {
                eprintln!("EOF null");
            }
        }
        _ => {
            eprintln!("Unknown command {}!", command);
        }
    }
}
