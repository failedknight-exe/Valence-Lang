// src/bin/cor_main.rs - Valence Runtime & Interactive REPL

use std::env;
use std::fs;
use std::io::{self, Write};

use valence::lexer::Lexer;
use valence::parser::Parser;
use valence::evaluator::Evaluator;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        start_repl(); // 👈 OPENS INTERACTIVE SHELL!
        return;
    }

    match args[1].as_str() {
        "run" => {
            let filename = if args.len() > 2 {
                args[2].clone()
            } else if std::path::Path::new("src/bridge.cor").exists() {
                "src/bridge.cor".to_string()
            } else if std::path::Path::new("bridge.cor").exists() {
                "bridge.cor".to_string()
            } else {
                println!("cor: No file specified and no bridge.cor found.");
                return;
            };

            if !filename.ends_with(".cor") {
                println!("cor: '{}' is not a .cor file.", filename);
                return;
            }

            let contents = match fs::read_to_string(&filename) {
                Ok(c) => c,
                Err(_) => {
                    println!("cor: Could not read '{}'.", filename);
                    return;
                }
            };

            println!("\x1b[36m▶ Running {}...\x1b[0m\n", filename);
            run_file(&contents);
        }

        "check" => {
            if args.len() < 3 {
                println!("cor: Specify a file to check.");
                return;
            }
            let filename = &args[2];
            let contents = match fs::read_to_string(filename) {
                Ok(c) => c,
                Err(_) => {
                    println!("cor: Could not find '{}'.", filename);
                    return;
                }
            };
            println!("Checking {}...", filename);
            check_file(&contents);
        }

        "help" => {
            println!("\ncor - Valence Language Runtime");
            println!("Sirius Zenith Labs | v0.3.1\n");
            println!("Commands:");
            println!("  cor                    Open Interactive REPL Shell");
            println!("  cor run                Execute bridge.cor");
            println!("  cor run <file.cor>     Execute specific file");
            println!("  cor check <file.cor>   Static syntax analysis");
            println!("  cor help               Display this menu");
            println!("  cor version            Runtime version\n");
        }

        "version" => {
            println!("Valence Runtime v0.3.1");
            println!("Sirius Zenith Labs / failedknight-exe");
        }

        _ => {
            println!("cor: '{}' is not a command. Try 'cor help'", args[1]);
        }
    }
}

/// 🐚 INTERACTIVE REPL SHELL
fn start_repl() {
    println!("\x1b[36m  ██╗   ██╗ █████╗ ██╗     ███████╗███╗   ██╗██╗████████╗\x1b[0m");
    println!("\x1b[36m  ██║   ██║██╔══██╗██║     ██╔════╝████╗  ██║██║╚══██╔══╝\x1b[0m");
    println!("\x1b[36m  ██║   ██║███████║██║     █████╗  ██╔██╗ ██║██║   ██║   \x1b[0m");
    println!("\x1b[36m  ╚██╗ ██╔╝██╔══██║██║     ██╔══╝  ██║╚██╗██║██║   ██║   \x1b[0m");
    println!("\x1b[36m   ╚████╔╝ ██║  ██║███████╗███████╗██║ ╚████║██║   ██║   \x1b[0m");
    println!("\x1b[36m    ╚═══╝  ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝  ╚═══╝╚═╝   ╚═╝   \x1b[0m");
    println!("\n\x1b[1mValence Interactive REPL (v0.3.1)\x1b[0m");
    println!("Type 'exit' to quit.\n");

    let mut evaluator = Evaluator::new();

    loop {
        print!("\x1b[32mcor> \x1b[0m");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() { break; }
        
        let trimmed = input.trim();
        if trimmed.is_empty() { continue; }
        if trimmed == "exit" || trimmed == "quit" { break; }

        let mut lexer = Lexer::new(trimmed);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();

        // Print the result of the last evaluated node dynamically!
        let mut last_val = valence::evaluator::value::Value::Null;
        for node in ast {
            last_val = evaluator.eval(node);
        }

        // Only print if it's not Null (prevents annoying null spam)
        if !matches!(last_val, valence::evaluator::value::Value::Null) {
            println!("\x1b[33m{}\x1b[0m", last_val);
        }
    }
}

fn run_file(contents: &str) {
    let mut lexer = Lexer::new(contents);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    let mut evaluator = Evaluator::new();
    evaluator.run(ast);
}

fn check_file(contents: &str) {
    if contents.trim().is_empty() {
        println!("[ERROR] This file is empty.");
        return;
    }
    let mut lexer = Lexer::new(contents);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let _ast = parser.parse();
    println!("No syntax errors found. Looking clean.");
}