//! Valence Execution Package (.vep) Encoder & Virtual File System (VFS)

use std::collections::HashMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};

pub const VEP_MAGIC: &[u8; 4] = b"VEP1";

static VFS_MEMORY: OnceLock<Arc<RwLock<HashMap<String, Vec<u8>>>>> = OnceLock::new();
static EXTRACTED_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn get_vfs() -> &'static Arc<RwLock<HashMap<String, Vec<u8>>>> {
    VFS_MEMORY.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}

#[derive(Debug, Clone)]
pub struct VepManifest {
    pub entry_point: String,
    pub target_os: String,
    pub is_stealth: bool,
    pub files: HashMap<String, (u64, u64)>,
}

pub struct VepPackager {
    pub manifest: VepManifest,
    pub raw_buffer: Vec<u8>,
}

impl VepPackager {
    pub fn new(entry_point: &str, target_os: &str, is_stealth: bool) -> Self {
        VepPackager {
            manifest: VepManifest {
                entry_point: entry_point.to_string(),
                target_os: target_os.to_string(),
                is_stealth,
                files: HashMap::new(),
            },
            raw_buffer: Vec::new(),
        }
    }

    pub fn add_bytes(&mut self, virtual_path: &str, bytes: &[u8]) {
        let offset = self.raw_buffer.len() as u64;
        let length = bytes.len() as u64;

        self.raw_buffer.extend_from_slice(bytes);
        self.manifest.files.insert(virtual_path.to_string(), (offset, length));

        println!("\x1b[32m  [VFS COMPRESSOR] Encapsulated: '{}' ({} bytes)\x1b[0m", virtual_path, length);
    }

    pub fn add_file(&mut self, virtual_path: &str, disk_path: &Path) -> Result<(), String> {
        let mut file = File::open(disk_path)
            .map_err(|e| format!("Failed to open asset '{}': {e}", disk_path.display()))?;

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|e| format!("Failed to read asset '{}': {e}", disk_path.display()))?;

        self.add_bytes(virtual_path, &bytes);
        Ok(())
    }

    fn patch_pe_subsystem_to_gui(exe_path: &Path) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(exe_path)
            .map_err(|e| format!("Failed to open binary for PE patching: {e}"))?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

        if buffer.len() < 0x40 || &buffer[0..2] != b"MZ" {
            return Ok(());
        }

        let pe_offset = u32::from_le_bytes(buffer[0x3C..0x40].try_into().unwrap()) as usize;
        if buffer.len() < pe_offset + 0x60 || &buffer[pe_offset..pe_offset + 4] != b"PE\0\0" {
            return Ok(());
        }

        let subsystem_offset = pe_offset + 92;
        if subsystem_offset + 2 <= buffer.len() {
            buffer[subsystem_offset] = 0x02; // GUI mode
            buffer[subsystem_offset + 1] = 0x00;

            file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
            file.write_all(&buffer).map_err(|e| e.to_string())?;
            println!("\x1b[32m  🥷 [PE PATCHER] Subsystem flipped to GUI (0x02) - 0ms console!\x1b[0m");
        }

        Ok(())
    }

    pub fn write_compressed_payload<W: Write>(&self, out: &mut W) -> Result<u64, String> {
        let uncompressed_size = self.raw_buffer.len();
        println!(
            "\x1b[36m  🗜️ [VFS COMPRESSOR] Crushing {} KB with Zstd Level 19...\x1b[0m",
            uncompressed_size / 1024
        );

        let compressed_payload = zstd::encode_all(&self.raw_buffer[..], 19)
            .map_err(|e| format!("Zstd compression failed: {e}"))?;

        let compressed_size = compressed_payload.len();
        let ratio = 100.0 - ((compressed_size as f64 / uncompressed_size.max(1) as f64) * 100.0);

        println!(
            "\x1b[32m  ✅ [VFS COMPRESSOR] {} KB ➔ {} KB ({:.1}% smaller!)\x1b[0m",
            uncompressed_size / 1024,
            compressed_size / 1024,
            ratio
        );

        // 1. Flags (4 bytes)
        let flags: u32 = if self.manifest.is_stealth { 1 } else { 0 };
        out.write_all(&flags.to_le_bytes()).map_err(|e| e.to_string())?;

        // 2. Serialize manifest files map: {"file1":[off,len],"file2":[off,len]}
        let files_map_json = self
            .manifest
            .files
            .iter()
            .map(|(k, (off, len))| format!("\"{}\":[{},{}]", k, off, len))
            .collect::<Vec<_>>()
            .join(",");

        let manifest_json = format!(
            "{{\"entry\":\"{}\",\"os\":\"{}\",\"stealth\":{},\"files\":{{{}}}}}",
            self.manifest.entry_point,
            self.manifest.target_os,
            self.manifest.is_stealth,
            files_map_json
        );
        let manifest_bytes = manifest_json.as_bytes();
        let manifest_len = manifest_bytes.len() as u64;

        // 3. Manifest Length (8 bytes)
        out.write_all(&manifest_len.to_le_bytes()).map_err(|e| e.to_string())?;

        // 4. Manifest Data
        out.write_all(manifest_bytes).map_err(|e| e.to_string())?;

        // 5. Compressed VFS Data
        out.write_all(&compressed_payload).map_err(|e| e.to_string())?;

        Ok(4 + 8 + manifest_len + compressed_size as u64)
    }
}

pub struct VepReader;

impl VepReader {
    pub fn get_extracted_dir() -> Option<&'static PathBuf> {
        EXTRACTED_DIR.get()
    }

    pub fn is_vep_file(path: &Path) -> bool {
        if let Ok(mut f) = File::open(path) {
            if let Ok(metadata) = f.metadata() {
                let len = metadata.len();
                if len >= 12 {
                    if f.seek(SeekFrom::End(-12)).is_ok() {
                        let mut footer = [0u8; 12];
                        if f.read_exact(&mut footer).is_ok() {
                            return &footer[8..12] == VEP_MAGIC;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn mount_and_read_entry(target_path: &Path) -> Result<(String, Vec<u8>), String> {
        let mut file = File::open(target_path)
            .map_err(|e| format!("Failed to open package: {e}"))?;

        let file_len = file.metadata().map_err(|e| e.to_string())?.len();
        if file_len < 12 {
            return Err("File too small for VEP payload.".to_string());
        }

        // 1. Read 12-byte footer
        file.seek(SeekFrom::End(-12)).map_err(|e| e.to_string())?;
        let mut footer = [0u8; 12];
        file.read_exact(&mut footer).map_err(|e| e.to_string())?;

        if &footer[8..12] != VEP_MAGIC {
            return Err("Not a valid Valence VEP package.".to_string());
        }

        let payload_start = u64::from_le_bytes(footer[0..8].try_into().unwrap());
        file.seek(SeekFrom::Start(payload_start)).map_err(|e| e.to_string())?;

        let mut flags_buf = [0u8; 4];
        file.read_exact(&mut flags_buf).map_err(|e| e.to_string())?;

        let mut len_buf = [0u8; 8];
        file.read_exact(&mut len_buf).map_err(|e| e.to_string())?;
        let manifest_len = u64::from_le_bytes(len_buf) as usize;

        let mut manifest_bytes = vec![0u8; manifest_len];
        file.read_exact(&mut manifest_bytes).map_err(|e| e.to_string())?;
        let manifest_str = String::from_utf8_lossy(&manifest_bytes);

        // Decompress payload directly into RAM
        let compressed_data_len = (file_len - 12) - (payload_start + 4 + 8 + manifest_len as u64);
        let mut compressed_data = vec![0u8; compressed_data_len as usize];
        file.read_exact(&mut compressed_data).map_err(|e| e.to_string())?;

        let decompressed_data = zstd::decode_all(&compressed_data[..])
            .map_err(|e| format!("Zstd decompression failed: {e}"))?;

        // 2. Extract Temp Directory for Runtime Worker Execution
        let temp_dir = env::temp_dir().join("valence_runtime_vfs");
        let _ = fs::create_dir_all(&temp_dir);
        let _ = EXTRACTED_DIR.set(temp_dir.clone());

        let mut entry_point = "bridge.val".to_string();

        // 3. Unpack all embedded sidecar assets to temp dir so Python/Java find them!
        if let Some(files_start) = manifest_str.find("\"files\":{") {
            let files_str = &manifest_str[files_start + 9..];
            if let Some(files_end) = files_str.find('}') {
                let inner = &files_str[..files_end];
                for pair in inner.split(',') {
                    let kv: Vec<&str> = pair.splitn(2, ':').collect();
                    if kv.len() == 2 {
                        let fname = kv[0].trim().trim_matches('"');
                        let arr_str = kv[1].trim().trim_matches('[').trim_matches(']');
                        let nums: Vec<u64> = arr_str
                            .split(',')
                            .filter_map(|n| n.trim().parse().ok())
                            .collect();

                        if nums.len() == 2 {
                            let offset = nums[0] as usize;
                            let length = nums[1] as usize;

                            if offset + length <= decompressed_data.len() {
                                let asset_bytes = &decompressed_data[offset..offset + length];
                                
                                // Write sidecar file out to runtime temp folder
                                let out_file_path = temp_dir.join(fname);
                                let _ = fs::write(&out_file_path, asset_bytes);

                                // Also insert into VFS RAM
                                let mut vfs = get_vfs().write().unwrap();
                                vfs.insert(fname.to_string(), asset_bytes.to_vec());
                            }
                        }
                    }
                }
            }
        }

        // Get entry point name
        if let Some(start) = manifest_str.find("\"entry\":\"") {
            let rest = &manifest_str[start + 9..];
            if let Some(end) = rest.find('"') {
                entry_point = rest[..end].to_string();
            }
        }

        let entry_bytes = get_vfs()
            .read()
            .unwrap()
            .get(&entry_point)
            .cloned()
            .unwrap_or_else(|| decompressed_data.clone());

        Ok((entry_point, entry_bytes))
    }
}