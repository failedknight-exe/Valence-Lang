// src/morph/mod.rs - Valence Standalone Morph Engine

pub mod morph_builder;
pub mod morph_packer;
pub mod morph_scanner;
pub mod morph_vfs;
pub use morph_vfs as vep;

use crate::parser::Node;
use morph_builder::MorphBuilder;
use morph_packer::MorphPacker;
use morph_scanner::MorphScanner;
use morph_vfs::VepPackager;
use std::fs;
use std::path::{Path, PathBuf};

pub struct MorphEngine {
    pub output_path: String,
    pub is_stealth: bool,
}

impl MorphEngine {
    pub fn new(output_path: &str, is_stealth: bool) -> Self {
        MorphEngine {
            output_path: output_path.to_string(),
            is_stealth,
        }
    }

    pub fn morph(&self, target_input: &str, ast: Vec<Node>) -> Result<(), String> {
        let os_target = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        };

        println!("\x1b[35m🛸 [VALENCE MORPH ENGINE v0.4]\x1b[0m");
        println!("  • Entry Script: {}", target_input);
        println!("  • Target Output: {}", self.output_path);

        let temp_dir = PathBuf::from("val_build_temp");
        let _ = fs::create_dir_all(&temp_dir);

        let entry_stem = Path::new(target_input)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("bridge");
        let val_binary_path = temp_dir.join(format!("{entry_stem}.val"));

        // Drawer 2: Pre-compile .cor text AST into .val binary AST
        MorphBuilder::compile_cor_to_val(&ast, &val_binary_path)?;

        let virt_entry_name = format!("{entry_stem}.val");
        let mut packager = VepPackager::new(&virt_entry_name, os_target, self.is_stealth);

        // Pack the .val binary AST into VFS
        packager.add_file(&virt_entry_name, &val_binary_path)?;

        // Drawer 1: Detector scans AST for bonded sidecars
        let sidecars = MorphScanner::scan(&ast);
        for asset in sidecars {
            if !asset.file_path.exists() {
                println!(
                    "\x1b[31m  [WARN] Bonded file '{}' missing during morph\x1b[0m",
                    asset.file_path.display()
                );
                continue;
            }

            // Drawer 2: Compiler pre-builds sidecar (Java -> .class, C -> binary)
            let compiled_sidecar = MorphBuilder::compile_sidecar(&asset)?;
            let virt_filename = compiled_sidecar
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();

            // Pack sidecar into VFS Space-Bag
            packager.add_file(virt_filename, &compiled_sidecar)?;
        }

        // Drawer 4: Gluer assembles <2MB engine + Space-Bag VFS + PE Stealth patch!
        let out_path = Path::new(&self.output_path);
        MorphPacker::pack(out_path, &packager, self.is_stealth)?;

        // Clean temp build folder
        let _ = fs::remove_dir_all(&temp_dir);

        println!(
            "\x1b[32m  🎉 SUCCESS: Standalone native app '{}' ready!\x1b[0m\n",
            self.output_path
        );
        Ok(())
    }
}

// VEP VFS Reader Export
pub use morph_vfs::VepReader;