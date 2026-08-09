//! Command-line runtime launcher for the Connect language.
//!
//! Handles `cor run`, `cor check`, `cor help`, and `cor version`.

use std::env;
use std::fs;

mod lexer;
mod parser;
mod evaluator;

use lexer::Lexer;
use parser::Parser;
use evaluator::Evaluator;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("");
        println!("  ██████╗ ██████╗ ██████╗ ");
        println!(" ██╔════╝██╔═══██╗██╔══██╗");
        println!(" ██║     ██║   ██║██████╔╝");
        println!(" ██║     ██║   ██║██╔══██╗");
        println!(" ╚██████╗╚██████╔╝██║  ██║");
        println!("  ╚═════╝ ╚═════╝ ╚═╝  ╚═╝");
        println!("");
        println!(" Connect Language Runtime v1.0.0");
        println!(" Sirius Zenith Labs");
        println!(" Type 'cor help' for commands");
        println!("");
        return;
    }

    match args[1].as_str() {
        "run" => {
            let filename = if args.len() > 2 {
                args[2].clone()
            } else {
                if std::path::Path::new("src/bridge.cor").exists() {
                    "src/bridge.cor".to_string()
                } else if std::path::Path::new("bridge.cor").exists() {
                    "bridge.cor".to_string()
                } else {
                    println!("cor: No file specified and no bridge.cor found.");
                    return;
                }
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

            println!("▶ Running {}...\n", filename);
            run_file(&contents, &filename);
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
            println!("");
            println!("cor - Connect Language Runtime");
            println!("Sirius Zenith Labs | v1.0.0");
            println!("");
            println!("Commands:");
            println!("  cor run                Execute bridge.cor");
            println!("  cor run <file.cor>     Execute specific file");
            println!("  cor check <file.cor>   Static syntax analysis");
            println!("  cor help               Display this menu");
            println!("  cor version            Runtime version");
            println!("");
        }

        "version" => {
            println!("Connect Runtime v1.0.0");
            println!("Sirius Zenith Labs");
        }

        _ => {
            println!("cor: '{}' is not a command.", args[1]);
            println!("Try 'cor help'");
        }
    }
}

/// Run a source file through the lexer, parser, and evaluator.
fn run_file(contents: &str, _filename: &str) {
    if !syntax_check(contents) {
        return;
    }

    let mut lexer = Lexer::new(contents);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    let mut evaluator = Evaluator::new();
    evaluator.run(ast);
}

/// Perform a lightweight syntax sanity check on parentheses and braces.
fn syntax_check(contents: &str) -> bool {
    let mut errors_found = false;

    let open_p = contents.matches('(').count();
    let close_p = contents.matches(')').count();

    if open_p > close_p {
        println!("[SYNTAX ERROR] Unclosed parenthesis detected.");
        println!("Damn... much like your household, this code is missing a father figure.");
        errors_found = true;
    } else if close_p > open_p {
        println!("[SYNTAX ERROR] Extra closing parenthesis detected.");
        println!("You closed something that was never opened. Deep.");
        errors_found = true;
    }

    let open_b = contents.matches('{').count();
    let close_b = contents.matches('}').count();

    if open_b != close_b {
        println!("[SYNTAX ERROR] Mismatched curly braces.");
        println!("Your code block has no closure. Like your emotional wounds.");
        errors_found = true;
    }

    if contents.trim().is_empty() {
        println!("[ERROR] This file is empty.");
        errors_found = true;
    }

    !errors_found
}

fn check_file(contents: &str) {
    let passed = syntax_check(contents);
    if passed {
        println!("No syntax errors found. Looking clean.");
    }
}