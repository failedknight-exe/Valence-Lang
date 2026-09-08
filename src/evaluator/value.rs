// src/evaluator/value.rs
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

use crate::parser::Node;

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    StringVal(String),
    Boolean(bool),
    Null,

    // shared heavy values (zero-copy)
    Array(Arc<RwLock<Vec<Value>>>),
    Map(Arc<RwLock<HashMap<String, Value>>>),

    // control-flow sentinels
    Return(Box<Value>),
    Break,
    Continue,
    Error(String),
}

impl Value {
    pub fn array(items: Vec<Value>) -> Self {
        Value::Array(Arc::new(RwLock::new(items)))
    }

    pub fn map(map: HashMap<String, Value>) -> Self {
        Value::Map(Arc::new(RwLock::new(map)))
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Integer(0) => false,
            Value::Float(f) if *f == 0.0 => false,
            Value::StringVal(s) if s.is_empty() => false,
            Value::Error(_) => false,
            _ => true,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::StringVal(a), Value::StringVal(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Array(a), Value::Array(b)) => {
                let aa = a.read().ok();
                let bb = b.read().ok();
                match (aa, bb) {
                    (Some(a), Some(b)) => *a == *b,
                    _ => false,
                }
            }
            (Value::Map(a), Value::Map(b)) => {
                let aa = a.read().ok();
                let bb = b.read().ok();
                match (aa, bb) {
                    (Some(a), Some(b)) => *a == *b,
                    _ => false,
                }
            }
            (Value::Error(a), Value::Error(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{n}"),
            Value::Float(n) => write!(f, "{n}"),
            Value::StringVal(s) => write!(f, "{s}"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Null => write!(f, "| |"),
            Value::Array(a) => {
                if let Ok(v) = a.read() {
                    let s = v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
                    write!(f, "[{s}]")
                } else {
                    write!(f, "[<locked>]")
                }
            }
            Value::Map(m) => {
                if let Ok(map) = m.read() {
                    let s = map
                        .iter()
                        .map(|(k, v)| format!("{k}: {v}"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    write!(f, "{{{s}}}")
                } else {
                    write!(f, "{{<locked>}}")
                }
            }
            Value::Return(v) => write!(f, "{v}"),
            Value::Break | Value::Continue => write!(f, ""),
            Value::Error(e) => write!(f, "{e}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StoredFunc {
    pub func_type: String,
    pub params: Vec<String>,
    pub body: Vec<Node>,
}