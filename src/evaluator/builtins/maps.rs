//! Methods for inspecting, updating, and field-accessing evaluator maps.
use crate::evaluator::value::Value;
use crate::evaluator::Evaluator;
use crate::parser::Node;
use std::collections::HashMap;

impl Evaluator {
    pub fn eval_map_method(
        &mut self,
        object: &str,
        method: &str,
        args: Vec<Node>,
    ) -> Value {
        let map_val = self.lookup(object);
        let map = match map_val {
            Value::Map(m) => m,
            Value::Error(e) => return Value::Error(e),
            _ => return Value::Error(format!("'{object}' is not a map. That's not a dictionary; that's just a dramatic bucket.")),
        };

        match method {
            "keys" => {
                let m = map.read().unwrap();
                let keys = m
                    .keys()
                    .map(|k| Value::StringVal(k.clone()))
                    .collect::<Vec<_>>();
                Value::array(keys)
            }
            "values" => {
                let m = map.read().unwrap();
                let values = m.values().cloned().collect::<Vec<_>>();
                Value::array(values)
            }
            "size" => {
                let m = map.read().unwrap();
                Value::Integer(m.len() as i64)
            }
            "has" => {
                if args.is_empty() {
                    return Value::Boolean(false);
                }
                let key = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    other => other.to_string(),
                };
                let m = map.read().unwrap();
                Value::Boolean(m.contains_key(&key))
            }
            "get" => {
                if args.is_empty() {
                    return Value::Null;
                }
                let key = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    other => other.to_string(),
                };
                let m = map.read().unwrap();
                m.get(&key).cloned().unwrap_or(Value::Null)
            }
            "delete" => {
                if args.is_empty() {
                    return Value::Null;
                }
                let key = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    other => other.to_string(),
                };
                {
                    let mut m = map.write().unwrap();
                    m.remove(&key);
                }
                Value::Map(map)
            }
            // Unknown methods are treated as field access to support `user.name`
            // without adding a separate AST node for map fields.
            other => {
                let m = map.read().unwrap();
                m.get(other).cloned().unwrap_or(Value::Null)
            }
        }
    }

    pub fn eval_map_lit(&mut self, keys: Vec<String>, values: Vec<Box<Node>>) -> Value {
        let mut map = HashMap::new();
        for (i, k) in keys.into_iter().enumerate() {
            let v = self.eval(*values[i].clone());
            map.insert(k, v);
        }
        Value::map(map)
    }
}