// src/bin/cor_main.rs - Valence Runtime & Interactive REPL

use std::env;
use std::fs;
use std::io::{self, Write};

use valence::evaluator::Evaluator;
use valence::lexer::Lexer;
use valence::parser::Parser;

fn main() {
    // 1. Self-Executing Check: If executable has an attached VEP payload, run it directly!
    if let Ok(current_exe) = env::current_exe() {
        if is_vep_file(&current_exe) {
            run_file(current_exe.to_str().unwrap());
            return;
        }
    }

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        start_repl();
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

            run_file(&filename);
        }

        "morph" => {
            let mut input_file: Option<String> = None;
            let mut output_exe: Option<String> = None;
            let mut is_stealth = true;

            let mut idx = 2;
            while idx < args.len() {
                let arg = &args[idx];
                if arg == "--console" || arg == "--dev" {
                    is_stealth = false;
                    idx += 1;
                } else if arg == "-o" && idx + 1 < args.len() {
                    output_exe = Some(args[idx + 1].clone());
                    idx += 2;
                } else if arg.ends_with(".vep") || arg.ends_with(".exe") || arg.ends_with(".app") {
                    output_exe = Some(arg.clone());
                    idx += 1;
                } else if arg.ends_with(".cor") {
                    input_file = Some(arg.clone());
                    idx += 1;
                } else {
                    if input_file.is_none() {
                        input_file = Some(arg.clone());
                    }
                    idx += 1;
                }
            }

            let target_input = match input_file {
                Some(f) => {
                    if std::path::Path::new(&f).exists() {
                        f
                    } else if std::path::Path::new(&format!("src/{f}")).exists() {
                        format!("src/{f}")
                    } else {
                        "src/bridge.cor".to_string()
                    }
                }
                None => "src/bridge.cor".to_string(),
            };

            let final_output = match output_exe {
                Some(out) => out,
                None => {
                    let stem = std::path::Path::new(&target_input)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("app");

                    if cfg!(target_os = "windows") {
                        format!("{stem}.exe")
                    } else if cfg!(target_os = "macos") {
                        format!("{stem}.app")
                    } else {
                        format!("{stem}.vep")
                    }
                }
            };

            let contents = match fs::read_to_string(&target_input) {
                Ok(c) => c,
                Err(_) => {
                    println!("cor morph error: Could not read '{}'.", target_input);
                    return;
                }
            };

            let mut lexer = Lexer::new(&contents);
            let tokens = lexer.tokenize();
            let mut parser = Parser::new(tokens);
            let ast = parser.parse();

            let engine = valence::morph::MorphEngine::new(&final_output, is_stealth);
            if let Err(e) = engine.morph(&target_input, ast) {
                println!("\x1b[31m[MORPH ERROR] {}\x1b[0m", e);
            }
        }

        "help" => {
            println!("\ncor - Valence Language Runtime");
            println!("Sirius Zenith Labs | v0.3.1\n");
            println!("Commands:");
            println!("  cor                    Open Interactive REPL Shell");
            println!("  cor run <file.cor>     Execute specific file");
            println!("  cor morph <file.cor>   Package into stealth binary (.exe / .app / .vep)");
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

fn run_file(filename: &str) {
    let path = std::path::Path::new(filename);

    if filename.ends_with(".vep") || (path.exists() && is_vep_file(path)) {
        match valence::morph::VepReader::mount_and_read_entry(path) {
            Ok((_entry_name, binary_val_bytes)) => {
                // Deserialize .val binary AST directly into Vec<Node> (0ms boot!)
                match bincode::deserialize::<Vec<valence::parser::Node>>(&binary_val_bytes) {
                    Ok(ast) => {
                        let mut evaluator = Evaluator::new();
                        evaluator.run(ast);
                    }
                    Err(e) => {
                        println!("\x1b[31m[VAL BINARY AST ERROR] Failed to load .val AST: {e}\x1b[0m");
                    }
                }
            }
            Err(e) => {
                println!("\x1b[31m[VEP EXECUTION ERROR] {}\x1b[0m", e);
            }
        }
    } else {
        let contents = match fs::read_to_string(filename) {
            Ok(c) => c,
            Err(_) => {
                println!("cor: Could not read '{}'.", filename);
                return;
            }
        };

        let mut lexer = Lexer::new(&contents);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        let mut evaluator = Evaluator::new();
        evaluator.run(ast);
    }
}

fn is_vep_file(path: &std::path::Path) -> bool {
    valence::morph::VepReader::is_vep_file(path)
}

fn start_repl() {
    println!("\x1b[36mValence Interactive REPL (v0.3.1)\x1b[0m");
    println!("Type 'exit' to quit.\n");

    let mut evaluator = Evaluator::new();

    loop {
        print!("\x1b[32mcor> \x1b[0m");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "exit" || trimmed == "quit" {
            break;
        }

        let mut lexer = Lexer::new(trimmed);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();

        let mut last_val = valence::evaluator::value::Value::Null;
        for node in ast {
            last_val = evaluator.eval(node);
        }

        if !matches!(last_val, valence::evaluator::value::Value::Null) {
            println!("\x1b[33m{}\x1b[0m", last_val);
        }
    }
}