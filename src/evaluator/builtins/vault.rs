// src/evaluator/builtins/vault.rs - Explicit Virtual RAM Allocator

use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

pub struct VaultBlock {
    pub slot_id: usize,
    pub size_bytes: usize,
    pub data: Vec<u8>,
    pub is_free: bool,
}

pub struct VaultManager {
    pub blocks: HashMap<usize, VaultBlock>,
    pub next_slot_id: usize,
}

static VAULT_ARENA: OnceLock<Arc<Mutex<VaultManager>>> = OnceLock::new();

fn get_vault() -> &'static Arc<Mutex<VaultManager>> {
    VAULT_ARENA.get_or_init(|| {
        Arc::new(Mutex::new(VaultManager {
            blocks: HashMap::new(),
            next_slot_id: 1,
        }))
    })
}

impl Evaluator {
    pub fn eval_vault_builtin(&mut self, method: &str, args: Vec<Node>) -> Value {
        let vault_arc = get_vault();

        match method {
            // vault.alloc(size_in_bytes) -> Returns Slot ID
            "alloc" => {
                if args.is_empty() { return Value::Error("vault.alloc() requires size in bytes. You can't allocate a memory dream with no dimensions.".into()); }
                let size_bytes = match self.eval(args[0].clone()) {
                    Value::Integer(n) if n > 0 => n as usize,
                    _ => return Value::Error("Size must be a positive integer. A negative size is not memory management; it's chaos engineering.".into()),
                };

                let mut vault = vault_arc.lock().unwrap();
                let slot_id = vault.next_slot_id;
                vault.next_slot_id += 1;

                vault.blocks.insert(slot_id, VaultBlock {
                    slot_id,
                    size_bytes,
                    data: vec![0u8; size_bytes],
                    is_free: false,
                });

                println!("\x1b[36m[VAULT] Allocated {} bytes in Slot #{}\x1b[0m", size_bytes, slot_id);
                Value::Integer(slot_id as i64)
            }

            // vault.write(slot_id, data_string)
            "write" => {
                if args.len() < 2 { return Value::Error("vault.write() requires slot ID and data. You can't write a novel with no address and no content.".into()); }
                let slot_id = match self.eval(args[0].clone()) {
                    Value::Integer(n) if n > 0 => n as usize,
                    _ => return Value::Error("Slot ID must be a positive integer. Zero and negatives are not slots; they're vibes.".into()),
                };
                let payload = self.eval(args[1].clone()).to_string();
                let payload_bytes = payload.as_bytes();

                let mut vault = vault_arc.lock().unwrap();
                if let Some(block) = vault.blocks.get_mut(&slot_id) {
                    if block.is_free {
                        return Value::Error(format!("Vault Slot #{} is free/unallocated", slot_id));
                    }
                    let write_len = payload_bytes.len().min(block.size_bytes);
                    block.data[..write_len].copy_from_slice(&payload_bytes[..write_len]);
                    Value::Boolean(true)
                } else {
                    Value::Error(format!("Vault Slot #{} does not exist", slot_id))
                }
            }

            // vault.read(slot_id) -> Returns String Data
            "read" => {
                if args.is_empty() { return Value::Error("vault.read() requires slot ID. Reading nothing is just staring into a blank screen with confidence.".into()); }
                let slot_id = match self.eval(args[0].clone()) {
                    Value::Integer(n) if n > 0 => n as usize,
                    _ => return Value::Error("Slot ID must be a positive integer. Negative slots are drama, not memory.".into()),
                };

                let vault = vault_arc.lock().unwrap();
                if let Some(block) = vault.blocks.get(&slot_id) {
                    if block.is_free {
                        return Value::Error(format!("Vault Slot #{} is free/unallocated. You tried to read a ghost. Very brave, very wrong.", slot_id));
                    }
                    // Read up to the first null byte (0x00)
                    let valid_data: Vec<u8> = block.data.iter().copied().take_while(|&b| b != 0).collect();
                    let result_str = String::from_utf8_lossy(&valid_data).to_string();
                    Value::StringVal(result_str)
                } else {
                    Value::Error(format!("Vault Slot #{} does not exist", slot_id))
                }
            }

            // vault.free(slot_id) -> Frees RAM instantly!
            "free" => {
                if args.is_empty() { return Value::Error("vault.free() requires slot ID. Try freeing a slot without an address and watch your code panic in public.".into()); }
                let slot_id = match self.eval(args[0].clone()) {
                    Value::Integer(n) if n > 0 => n as usize,
                    _ => return Value::Error("Slot ID must be a positive integer. Negative numbers can't free memory; they just create more problems.".into()),
                };

                let mut vault = vault_arc.lock().unwrap();
                if let Some(block) = vault.blocks.get_mut(&slot_id) {
                    block.is_free = true;
                    block.data.clear();
                    println!("\x1b[33m[VAULT] Freed Slot #{}\x1b[0m", slot_id);
                    Value::Boolean(true)
                } else {
                    Value::Boolean(false)
                }
            }

            _ => Value::Error(format!("Unknown vault method '{method}'. That command has less structure than a closet full of mystery boxes.")),
        }
    }
}