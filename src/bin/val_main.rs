// val_main.rs
// Valence Library & Helper
// Sirius Zenith Labs

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("val: Valence Helper. Type 'val help' for commands.");
        return;
    }

    match args[1].as_str() {
        "init" => {
            let project_name = if args.len() > 2 {
                &args[2]
            } else {
                println!("val: What are you naming your project?");
                println!("Usage: val init <project_name>");
                return;
            };

            println!("Initializing Valence project: {}...", project_name);

            fs::create_dir_all(format!("{}/src", project_name)).unwrap();
            fs::create_dir_all(format!("{}/modules", project_name)).unwrap();
            fs::create_dir_all(format!("{}/.valAI", project_name)).unwrap();

            let bridge_content = format!(
r#"// {} — Valence Core File
// Run with: cor run src/bridge.cor

// Start coding here!
"#, project_name);

            fs::write(format!("{}/src/bridge.cor", project_name), bridge_content).unwrap();

            let data_toml = format!(
r#"[project]
name = "{}"
version = "0.1.0"
lang_version = "1.0"

[runtime]
engine = "valence-cor"

[dependencies]
"#, project_name);

            fs::write(format!("{}/data.toml", project_name), data_toml).unwrap();

            println!("Valence project '{}' initialized!", project_name);
            println!("Run it with: cd {} && cor run", project_name);
        }

        "valAI" => {
            let prompt = if args.len() > 2 {
                args[2..].join(" ")
            } else {
                println!("valAI: You called me and said nothing. Try again.");
                return;
            };

            println!("valAI: On it. Processing \"{}\"...", prompt);
        }

        "install" => {
            let package = if args.len() > 2 { &args[2] } else { "package" };
            println!(" Installing {}...", package);
            println!("(Valence Registry coming in v1.5)");
        }

        "help" => {
            println!("\nval — Valence Helper Toolchain");
            println!("Commands:");
            println!("  val init <name>       Create a new Valence project");
            println!("  val install <pkg>     Install a Valence library");
            println!("  val valAI \"<prompt>\"  Query local AI assistant");
            println!("  val help              Show this menu\n");
        }

        _ => {
            println!("val: Unknown command '{}'. Try 'val help'", args[1]);
        }
    }
}