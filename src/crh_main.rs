//! Command-line helper for Connect runtime projects.
//!
//! Supports initialization and package installation tasks.

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("crh: Connect Runtime Helper. Use 'crh help'");
        return;
    }

    match args[1].as_str() {
        "init" => {
            let project_name = if args.len() > 2 {
                &args[2]
            } else {
                println!("crh: What are you naming your project?");
                println!("Usage: crh init <project_name>");
                return;
            };

            println!("Initializing Connect project: {}...", project_name);

            fs::create_dir_all(format!("{}/src", project_name))
                .expect("Could not create src folder");
            fs::create_dir_all(format!("{}/modules", project_name))
                .expect("Could not create modules folder");

            let bridge_content = format!(
r#"// {} - Connect Project
// This is your bridge file. Everything starts here.
// Run with: cor run src/bridge.cor

// Start coding here!
"#, project_name);

            fs::write(
                format!("{}/src/bridge.cor", project_name),
                bridge_content
            ).expect("Could not create bridge.cor");

            let data_toml = format!(
r#"[project]
name = "{}"
version = "0.1.0"
author = ""
lang_version = "1.0"

[runtime]
memory_manager = "rust"

[dependencies]
"#, project_name);

            fs::write(
                format!("{}/data.toml", project_name),
                data_toml
            ).expect("Could not create data.toml");

            println!("");
            println!("Project '{}' created!", project_name);
            println!("");
            println!("{}/", project_name);
            println!("  src/");
            println!("    bridge.cor   <- Start coding here");
            println!("  modules/       <- Your packages");
            println!("  data.toml      <- Project config");
            println!("");
            println!("Run your project:");
            println!("  cd {}", project_name);
            println!("  cor run src/bridge.cor");
        }

        "install" => {
            let package = if args.len() > 2 {
                &args[2]
            } else {
                println!("crh: Install what exactly? Usage: crh install <package>");
                return;
            };
            println!("Installing {}...", package);
            println!("(Package registry coming in V1.5)");
        }

        "help" => {
            println!("");
            println!("crh - Connect Runtime Helper");
            println!("");
            println!("Commands:");
            println!("  crh init <name>          Create a new Connect project");
            println!("  crh install <package>    Install a Connect package");
            println!("  crh help                 Show this menu");
            println!("");
        }

        _ => {
            println!("crh: '{}' is not a command.", args[1]);
            println!("Try 'crh help'");
        }
    }
}