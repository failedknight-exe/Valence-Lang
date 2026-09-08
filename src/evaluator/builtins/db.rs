// src/evaluator/builtins/db.rs - Native Zero-Config Database Module

use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, RwLock};

pub struct DatabaseEngine {
    pub current_path: Option<String>,
    pub store: HashMap<String, String>,
}

impl DatabaseEngine {
    pub fn new() -> Self {
        DatabaseEngine {
            current_path: None,
            store: HashMap::new(),
        }
    }
}

// Global DB instance
static DB_INSTANCE: std::sync::OnceLock<Arc<RwLock<DatabaseEngine>>> = std::sync::OnceLock::new();

fn get_db() -> &'static Arc<RwLock<DatabaseEngine>> {
    DB_INSTANCE.get_or_init(|| Arc::new(RwLock::new(DatabaseEngine::new())))
}

impl Evaluator {
    pub fn eval_db_builtin(&mut self, method: &str, args: Vec<Node>) -> Value {
        let db_arc = get_db();

        match method {
            "open" => {
    if args.is_empty() {
        return Value::Error("db.open() expects a file path".into());
    }
    let path = match self.eval(args[0].clone()) {
        Value::StringVal(s) => s,
        _ => return Value::Error("Database path must be a string".into()),
    };

    let mut db = db_arc.write().unwrap();
    db.current_path = Some(path.clone());

    // 💡 TOUCH / CREATE FILE ON DISK IMMEDIATELY IF NEW!
    if !std::path::Path::new(&path).exists() {
        let _ = fs::write(&path, "");
    } else if let Ok(content) = fs::read_to_string(&path) {
        let mut map = HashMap::new();
        for line in content.lines() {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }
        db.store = map;
    }
    Value::Boolean(true)
}

            "set" => {
                if args.len() < 2 {
                    return Value::Error("db.set() requires key and value".into());
                }
                let key = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    other => other.to_string(),
                };
                let val = self.eval(args[1].clone());

                let mut db = db_arc.write().unwrap();
                db.store.insert(key, val.to_string());

                // Auto-flush to disk if path is open!
                if let Some(path) = &db.current_path {
                    let mut content = String::new();
                    for (k, v) in &db.store {
                        content.push_str(&format!("{k}={v}\n"));
                    }
                    let _ = fs::write(path, content);
                }
                val
            }

            "get" => {
                if args.is_empty() { return Value::Null; }
                let key = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    other => other.to_string(),
                };

                let db = db_arc.read().unwrap();
                if let Some(val_str) = db.store.get(&key) {
                    Value::StringVal(val_str.clone())
                } else {
                    Value::Null
                }
            }

            "has" => {
                if args.is_empty() { return Value::Boolean(false); }
                let key = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    other => other.to_string(),
                };

                let db = db_arc.read().unwrap();
                Value::Boolean(db.store.contains_key(&key))
            }

            "delete" => {
                if args.is_empty() { return Value::Boolean(false); }
                let key = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    other => other.to_string(),
                };

                let mut db = db_arc.write().unwrap();
                let existed = db.store.remove(&key).is_some();

                if existed {
                    if let Some(path) = &db.current_path {
                        let mut content = String::new();
                        for (k, v) in &db.store {
                            content.push_str(&format!("{k}={v}\n"));
                        }
                        let _ = fs::write(path, content);
                    }
                }
                Value::Boolean(existed)
            }

            _ => Value::Error(format!("Unknown db method '{method}'.")),
        }
    }
}