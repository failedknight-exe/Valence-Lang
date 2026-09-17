use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;

impl Evaluator {
    pub fn eval_crypto_builtin(&mut self, method: &str, args: Vec<Node>) -> Value {
        match method {
            "hash" => {
                if args.is_empty() { return Value::Error("crypto.hash needs data payload. You can't hash nothing and call it a strategy.".to_string()); }
                let data = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    other => format!("{}", other),
                };
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                data.hash(&mut hasher);
                let hash_val = hasher.finish();
                Value::StringVal(format!("{:016x}", hash_val))
            }
            "uuid" => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
                let rand1 = now % 0xFFFFFFFF;
                let rand2 = (now >> 32) % 0xFFFFFFFF;
                let rand3 = (now >> 64) % 0xFFFF;
                let rand4 = (now >> 80) % 0xFFFF;
                let rand5 = (now >> 96) % 0xFFFFFFFFFFFF;
                Value::StringVal(format!(
                    "{:08x}-{:04x}-{:03x}-{:04x}-{:012x}",
                    rand1, rand2 % 0xFFFF, rand3 % 0xFFFF, rand4, rand5
                ))
            }
            "randomInt" => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as f64;
                let r = (seed % 1000.0) / 1000.0;
                if args.len() >= 2 {
                    let min = match self.eval(args[0].clone()) { Value::Integer(n) => n as f64, _ => 0.0 };
                    let max = match self.eval(args[1].clone()) { Value::Integer(n) => n as f64, _ => 100.0 };
                    Value::Integer((min + r * (max - min)) as i64)
                } else {
                    Value::Integer((r * 100.0) as i64)
                }
            }
            "randomBytes" => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let count = if args.is_empty() { 16 } else {
                    match self.eval(args[0].clone()) { Value::Integer(n) => n as usize, _ => 16 }
                };
                let mut result = String::new();
                for i in 0..count {
                    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos().wrapping_add(i as u32);
                    result.push_str(&format!("{:02x}", seed % 256));
                }
                Value::StringVal(result)
            }
            "base64Encode" => {
                if args.is_empty() { return Value::Null; }
                let data = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    other => format!("{}", other),
                };
                let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
                let bytes = data.as_bytes();
                let mut result = String::new();
                let mut i = 0;
                while i < bytes.len() {
                    let b0 = bytes[i] as u32;
                    let b1 = if i + 1 < bytes.len() { bytes[i + 1] as u32 } else { 0 };
                    let b2 = if i + 2 < bytes.len() { bytes[i + 2] as u32 } else { 0 };
                    let triple = (b0 << 16) | (b1 << 8) | b2;
                    result.push(chars.as_bytes()[((triple >> 18) & 0x3F) as usize] as char);
                    result.push(chars.as_bytes()[((triple >> 12) & 0x3F) as usize] as char);
                    if i + 1 < bytes.len() {
                        result.push(chars.as_bytes()[((triple >> 6) & 0x3F) as usize] as char);
                    } else {
                        result.push('=');
                    }
                    if i + 2 < bytes.len() {
                        result.push(chars.as_bytes()[(triple & 0x3F) as usize] as char);
                    } else {
                        result.push('=');
                    }
                    i += 3;
                }
                Value::StringVal(result)
            }
            "base64Decode" => {
                if args.is_empty() { return Value::Null; }
                let data = match self.eval(args[0].clone()) { Value::StringVal(s) => s, _ => return Value::Null };
                let table = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
                let mut output = Vec::new();
                let mut buffer = 0u32;
                let mut bits = 0u8;
                for c in data.chars() {
                    if c == '=' { break; }
                    if let Some(pos) = table.find(c) {
                        buffer = (buffer << 6) | pos as u32;
                        bits += 6;
                        if bits >= 8 {
                            bits -= 8;
                            output.push((buffer >> bits) as u8);
                            buffer &= (1 << bits) - 1;
                        }
                    }
                }
                Value::StringVal(String::from_utf8_lossy(&output).to_string())
            }
            _ => Value::Error(format!("'{}' is not a valid crypto method. Even the crypto gods would laugh at that one.", method)),
        }
    }
}