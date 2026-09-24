// src/morph/morph_builder.rs
// 🛠️ MORPH DRAWER 2: THE COMPILER
// Pre-compiles .cor text into .val binary AST (0ms boot + source code protection)
// and pre-builds sidecars (Java -> .class, C -> binary) before packing!

use crate::evaluator::builtins::vbp_runner;
use crate::morph::morph_scanner::SidecarAsset;
use crate::parser::Node;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct MorphBuilder;

impl MorphBuilder {
    /// Pre-compiles Valence AST (Vec<Node>) into a .val binary AST file!
    pub fn compile_cor_to_val(ast: &[Node], out_val_path: &Path) -> Result<(), String> {
        println!("\x1b[36m  ⚡ [VAL COMPILER] Pre-compiling .cor AST -> .val binary...\x1b[0m");

        let binary_ast = bincode::serialize(ast)
            .map_err(|e| format!("Failed to serialize Valence AST into .val: {e}"))?;

        let mut file = File::create(out_val_path)
            .map_err(|e| format!("Failed to create .val binary file: {e}"))?;

        file.write_all(&binary_ast)
            .map_err(|e| format!("Failed to write .val binary bytes: {e}"))?;

        println!(
            "\x1b[32m  ✅ [VAL COMPILER] Created .val binary ({} bytes | 0ms boot ready)\x1b[0m",
            binary_ast.len()
        );
        Ok(())
    }

    /// Pre-compiles bonded sidecar scripts (Java -> .class, C -> binary, etc.)
    pub fn compile_sidecar(asset: &SidecarAsset) -> Result<PathBuf, String> {
        let path = &asset.file_path;
        let lang = &asset.lang;

        match lang.as_str() {
            "java" | "c" | "cpp" | "rs" => {
                println!(
                    "\x1b[36m  🛠️ [SIDE COMPILER] Pre-building {} asset: {}...\x1b[0m",
                    lang.to_uppercase(),
                    path.display()
                );

                // Uses Drawer 2 of VBP (vbp_runner) to run javac / gcc / rustc!
                let _ = vbp_runner::prepare_command(lang, path)?;

                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
                let parent = path.parent().unwrap_or_else(|| Path::new("."));
                let exe_ext = if std::env::consts::OS == "windows" { ".exe" } else { "" };

                let compiled_file = match lang.as_str() {
                    "java" => parent.join(format!("{stem}.class")),
                    "c" => parent.join(format!("{stem}_c{exe_ext}")),
                    "cpp" => parent.join(format!("{stem}_cpp{exe_ext}")),
                    "rs" => parent.join(format!("{stem}_rs{exe_ext}")),
                    _ => path.clone(),
                };

                if compiled_file.exists() {
                    Ok(compiled_file)
                } else {
                    Ok(path.clone())
                }
            }
            _ => Ok(path.clone()), // Raw script (Python, JS, text files)
        }
    }
}