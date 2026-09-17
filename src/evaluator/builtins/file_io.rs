use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;
use std::io::Write as IoWrite;

impl Evaluator {
    pub fn eval_file_builtin(&mut self, method: &str, args: Vec<Node>) -> Value {
        match method {
            "read" => {
                if args.is_empty() { return Value::Null; }
                let path = match self.eval(args[0].clone()) { Value::StringVal(s) => s, _ => return Value::Null };
                match std::fs::read_to_string(&path) {
                    Ok(content) => Value::StringVal(content),
                    Err(_) => Value::Error(format!("Could not read file '{}'. That's a file with trust issues and zero cooperation.", path)),
                }
            }
            "write" => {
                if args.len() < 2 { return Value::Null; }
                let path = match self.eval(args[0].clone()) { Value::StringVal(s) => s, _ => return Value::Null };
                let content = format!("{}", self.eval(args[1].clone()));
                match std::fs::write(&path, content) {
                    Ok(_) => Value::Boolean(true),
                    Err(_) => Value::Error(format!("Could not write to '{}'. The file refused to take your nonsense.", path)),
                }
            }
            "append" => {
                if args.len() < 2 { return Value::Null; }
                let path = match self.eval(args[0].clone()) { Value::StringVal(s) => s, _ => return Value::Null };
                let content = format!("{}", self.eval(args[1].clone()));
                match std::fs::OpenOptions::new().append(true).create(true).open(&path) {
                    Ok(mut f) => {
                        let _ = f.write_all(content.as_bytes());
                        Value::Boolean(true)
                    }
                    Err(_) => Value::Error(format!("Could not append to '{}'. That file has more resistance than a bad Wi-Fi signal.", path)),
                }
            }
            "exists" => {
                if args.is_empty() { return Value::Boolean(false); }
                let path = match self.eval(args[0].clone()) { Value::StringVal(s) => s, _ => return Value::Boolean(false) };
                Value::Boolean(std::path::Path::new(&path).exists())
            }
            "delete" => {
                if args.is_empty() { return Value::Boolean(false); }
                let path = match self.eval(args[0].clone()) { Value::StringVal(s) => s, _ => return Value::Boolean(false) };
                match std::fs::remove_file(&path) {
                    Ok(_) => Value::Boolean(true),
                    Err(_) => Value::Boolean(false),
                }
            }
            _ => Value::Null,
        }
    }
}