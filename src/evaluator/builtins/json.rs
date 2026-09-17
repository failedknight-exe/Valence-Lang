use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;

impl Evaluator {
    pub fn eval_json_builtin(&mut self, method: &str, args: Vec<Node>) -> Value {
        match method {
            "stringify" => {
                if args.is_empty() { return Value::Null; }
                let val = self.eval(args[0].clone());
                match val {
                    Value::Map(ref map) => {
                        let guard = map.read().unwrap();
                        let pairs: Vec<String> = guard
                            .iter()
                            .map(|(k, v)| {
                                let val_str = match v {
                                    Value::StringVal(s) => format!("\"{}\"", s),
                                    Value::Integer(n) => format!("{}", n),
                                    Value::Float(f) => format!("{}", f),
                                    Value::Boolean(b) => format!("{}", b),
                                    Value::Null => "null".to_string(),
                                    other => format!("{}", other),
                                };
                                format!("\"{}\":{}", k, val_str)
                            })
                            .collect();
                        Value::StringVal(format!("{{{}}}", pairs.join(",")))
                    }
                    Value::Array(ref arr) => {
                        let guard = arr.read().unwrap();
                        let items: Vec<String> = guard
                            .iter()
                            .map(|v| match v {
                                Value::StringVal(s) => format!("\"{}\"", s),
                                other => format!("{}", other),
                            })
                            .collect();
                        Value::StringVal(format!("[{}]", items.join(",")))
                    }
                    other => Value::StringVal(format!("{}", other)),
                }
            }
            "parse" => {
                if args.is_empty() { return Value::Null; }
                let text = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("json.parse needs a string argument. A number is not JSON, and neither is a shrug.".to_string()),
                };
                let trimmed = text.trim();
                if trimmed.starts_with('{') {
                    let mut map = std::collections::HashMap::new();
                    let inner = &trimmed[1..trimmed.len() - 1];
                    for pair in inner.split(',') {
                        let kv: Vec<&str> = pair.splitn(2, ':').collect();
                        if kv.len() == 2 {
                            let key = kv[0].trim().trim_matches('"').to_string();
                            let val_str = kv[1].trim();
                            let val = if val_str.starts_with('"') {
                                Value::StringVal(val_str.trim_matches('"').to_string())
                            } else if val_str == "true" {
                                Value::Boolean(true)
                            } else if val_str == "false" {
                                Value::Boolean(false)
                            } else if val_str == "null" {
                                Value::Null
                            } else if let Ok(n) = val_str.parse::<i64>() {
                                Value::Integer(n)
                            } else if let Ok(f) = val_str.parse::<f64>() {
                                Value::Float(f)
                            } else {
                                Value::StringVal(val_str.to_string())
                            };
                            map.insert(key, val);
                        }
                    }
                    Value::map(map)
                } else {
                    Value::Error("json.parse: invalid JSON structure format. That payload is less structured than a toddler's toy box.".to_string())
                }
            }
            _ => Value::Error(format!("'{}' is not a valid json method. That's not JSON logic; that's just pretend programming.", method)),
        }
    }
}