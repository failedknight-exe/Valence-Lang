use crate::parser::Node;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    StringVal(String),
    Boolean(bool),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
    Null,
    Return(Box<Value>),
    Break,
    Continue,
    Error(String),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::StringVal(a), Value::StringVal(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Break, Value::Break) => true,
            (Value::Continue, Value::Continue) => true,
            (Value::Error(a), Value::Error(b)) => a == b,
            _ => false,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::StringVal(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "| |"),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Map(map) => {
                let items: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            Value::Return(v) => write!(f, "{}", v),
            Value::Break => write!(f, ""),
            Value::Continue => write!(f, ""),
            Value::Error(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StoredFunc {
    pub func_type: String,
    pub params: Vec<String>,
    pub body: Vec<Node>,
}