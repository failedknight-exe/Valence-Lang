use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;
use std::collections::HashMap;

impl Evaluator {
    pub fn eval_map_method(&mut self, object: &str, method: &str, map: &HashMap<String, Value>, args: Vec<Node>) -> Value {
        match method {
            "keys" => {
                let k: Vec<Value> = map.keys().map(|key| Value::StringVal(key.clone())).collect();
                Value::Array(k)
            }
            "values" => {
                let v: Vec<Value> = map.values().cloned().collect();
                Value::Array(v)
            }
            "size" => Value::Integer(map.len() as i64),
            "has" => {
                if args.is_empty() { return Value::Null; }
                let key = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Boolean(false),
                };
                Value::Boolean(map.contains_key(&key))
            }
            "get" => {
                if args.is_empty() { return Value::Null; }
                let key = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Null,
                };
                map.get(&key).cloned().unwrap_or(Value::Null)
            }
            "delete" => {
                if args.is_empty() { return Value::Null; }
                let key = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Null,
                };
                let mut new_map = map.clone();
                new_map.remove(&key);
                let result = Value::Map(new_map);
                if self.local_vars.contains_key(object) {
                    self.local_vars.insert(object.to_string(), result.clone());
                } else if self.global_vars.contains_key(object) {
                    self.global_vars.insert(object.to_string(), result.clone());
                }
                result
            }
            _ => map.get(method).cloned().unwrap_or(Value::Null),
        }
    }

    pub fn eval_map_lit(&mut self, keys: Vec<String>, values: Vec<Box<Node>>) -> Value {
        let mut map = HashMap::new();
        for (i, key) in keys.iter().enumerate() {
            let val = self.eval(*values[i].clone());
            map.insert(key.clone(), val);
        }
        Value::Map(map)
    }
}