mod expr;
mod parser;
mod tokenizer;
mod stmt;

use std::collections::HashMap;
use std::io::Write;
use std::{env, fs, io};

use parser::{parse, ParserError};
use tokenizer::{tokenize, TokenizerError};
use expr::RuntimeError;

fn run(source: &String) {
    // Tokenization
    let tokens = match tokenize(source) {
        Ok(tokens) => tokens,
        Err(err) => {
            match err {
                TokenizerError::BadChar(ch, line, col) => {
                    println!("Line {}, Col {} :: Unexpected character: {}", line, col, ch);
                }
                TokenizerError::UnterminatedStringLiteral(line, col) => {
                    println!(
                        "Line {}, Col {} :: Unterminated string literal begins here",
                        line, col
                    );
                }
                TokenizerError::InvalidNumberLiteral(lexeme, line, col) => {
                    println!(
                        "Line {}, Col {} :: Invalid number literal \"{}\" begins here",
                        line, col, lexeme
                    )
                }
                TokenizerError::UnterminatedBlockComment(line, col) => {
                    println!(
                        "Line {}, Col {} :: Unterminated block comment begins here",
                        line, col
                    )
                }
            }
            return;
        }
    };
    
    // Parsing
    let statements = match parse(tokens) {
        Ok(statements) => statements,
        Err(err) => {
            match err {
                ParserError::SyntaxError(token, msg) => println!("Syntax Error :: Line {}, Col {} :: {}", token.line, token.col, &msg),
            }
            return;
        },
    };

    // Interpreting
    let mut values = HashMap::new();
    for statement in statements {
        if let Err(err) = statement.interpret(&mut values) {
            match err {
                RuntimeError::TypeError(token, msg) =>  println!("Type Error :: Line {}, Col {} :: {}", token.line, token.col, &msg),
            }
        }
    }
}

fn run_file(path: &String) -> Result<(), std::io::Error> {
    let source = fs::read_to_string(path)?;
    run(&source);
    Ok(())
}

fn run_prompt() {
    loop {
        print!(">> ");
        io::stdout().flush().expect("Failed to flush stdout");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        if input.trim().is_empty() {
            break;
        }
        run(&input)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: lox-rs [script]");
    } else if args.len() == 2 {
        if let Err(err) = run_file(&args[1]) {
            println!("Failed to open the file {}: {err}", &args[1]);
        }
    } else {
        run_prompt();
    }
}
