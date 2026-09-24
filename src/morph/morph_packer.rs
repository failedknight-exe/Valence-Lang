// src/morph/morph_packer.rs
// 🛠️ MORPH DRAWER 4: THE GLUER
// Copies <2MB cor.exe runtime, glues compressed Space-Bag VFS to the end,
// and flips Windows PE Subsystem byte from Console (0x03) -> GUI (0x02) for 0ms stealth!

use crate::morph::morph_vfs::VepPackager;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

pub struct MorphPacker;

impl MorphPacker {
    /// Copies current cor.exe, patches PE header to GUI mode, and glues VFS payload onto the end!
    pub fn pack(output_path: &Path, packager: &VepPackager, is_stealth: bool) -> Result<(), String> {
        let is_exe = output_path.extension().map_or(false, |ext| ext == "exe" || ext == "app");

        if is_exe {
            // 1. Locate current cor.exe engine
            let current_cor = env::current_exe()
                .map_err(|e| format!("Could not locate cor runtime binary: {e}"))?;

            // 2. Copy <2MB engine to target location
            fs::copy(&current_cor, output_path)
                .map_err(|e| format!("Could not create executable '{}': {e}", output_path.display()))?;

            // 3. Patch PE Subsystem Header to GUI Mode (0x02) on Windows if stealth requested!
            if is_stealth && cfg!(target_os = "windows") {
                let _ = Self::patch_pe_subsystem_to_gui(output_path);
            }

            // 4. Open binary in append mode to glue Space-Bag
            let mut out = OpenOptions::new()
                .append(true)
                .open(output_path)
                .map_err(|e| format!("Could not open binary for payload appending: {e}"))?;

            let payload_start_pos = out.seek(SeekFrom::End(0)).unwrap_or(0);

            // 5. Write Zstd Compressed VFS Payload
            packager.write_compressed_payload(&mut out)?;

            // 6. Write 12-Byte Footer: [payload_start_pos: 8 bytes][VEP_MAGIC ("VEP1"): 4 bytes]
            out.write_all(&payload_start_pos.to_le_bytes()).map_err(|e| e.to_string())?;
            out.write_all(crate::morph::morph_vfs::VEP_MAGIC).map_err(|e| e.to_string())?;

            println!(
                "\x1b[32m  🎉 [GLUER] Self-executing binary assembled: '{}' (0ms Stealth Active)\x1b[0m",
                output_path.display()
            );
        } else {
            // Standalone raw .vep package
            let mut out = File::create(output_path)
                .map_err(|e| format!("Could not create .vep file: {e}"))?;

            let payload_start_pos = 0u64;
            packager.write_compressed_payload(&mut out)?;

            out.write_all(&payload_start_pos.to_le_bytes()).map_err(|e| e.to_string())?;
            out.write_all(crate::morph::morph_vfs::VEP_MAGIC).map_err(|e| e.to_string())?;

            println!(
                "\x1b[32m  🎉 [GLUER] Standalone .vep package created: '{}'\x1b[0m",
                output_path.display()
            );
        }

        Ok(())
    }

    /// Flips byte 0x5C in PE optional header from 0x03 (Console) to 0x02 (GUI)
    fn patch_pe_subsystem_to_gui(exe_path: &Path) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(exe_path)
            .map_err(|e| format!("Failed to open executable for PE patching: {e}"))?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

        // Check DOS header "MZ"
        if buffer.len() < 0x40 || &buffer[0..2] != b"MZ" {
            return Ok(());
        }

        let pe_offset = u32::from_le_bytes(buffer[0x3C..0x40].try_into().unwrap()) as usize;
        if buffer.len() < pe_offset + 0x60 || &buffer[pe_offset..pe_offset + 4] != b"PE\0\0" {
            return Ok(());
        }

        let subsystem_offset = pe_offset + 92;
        if subsystem_offset + 2 <= buffer.len() {
            buffer[subsystem_offset] = 0x02; // GUI Mode
            buffer[subsystem_offset + 1] = 0x00;

            file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
            file.write_all(&buffer).map_err(|e| e.to_string())?;
            println!("\x1b[36m  🥷 [PE PATCHER] Subsystem flipped to GUI (0x02) - 0ms console window!\x1b[0m");
        }

        Ok(())
    }
}