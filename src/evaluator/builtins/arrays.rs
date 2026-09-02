use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;

impl Evaluator {
    pub fn eval_array_method(&mut self, object: &str, method: &str, elements: &[Value], args: Vec<Node>) -> Value {
        match method {
            "len" => Value::Integer(elements.len() as i64),
            "first" => if elements.is_empty() { Value::Null } else { elements[0].clone() },
            "last" => if elements.is_empty() { Value::Null } else { elements[elements.len() - 1].clone() },
            "isEmpty" => Value::Boolean(elements.is_empty()),
            "has" => {
                if args.is_empty() { return Value::Null; }
                let check = self.eval(args[0].clone());
                Value::Boolean(elements.contains(&check))
            }
            "indexOf" => {
                if args.is_empty() { return Value::Integer(-1); }
                let search = self.eval(args[0].clone());
                for (i, el) in elements.iter().enumerate() {
                    if *el == search { return Value::Integer(i as i64); }
                }
                Value::Integer(-1)
            }
            "join" => {
                let sep = if args.is_empty() { "".to_string() }
                    else { match self.eval(args[0].clone()) { Value::StringVal(s) => s, _ => "".to_string() } };
                let joined: Vec<String> = elements.iter().map(|v| format!("{}", v)).collect();
                Value::StringVal(joined.join(&sep))
            }
            "slice" => {
                if args.len() < 2 { return Value::Null; }
                let start = match self.eval(args[0].clone()) { Value::Integer(i) => i as usize, _ => 0 };
                let end = match self.eval(args[1].clone()) { Value::Integer(i) => i as usize, _ => elements.len() };
                if start > end || end > elements.len() { return Value::Array(Vec::new()); }
                Value::Array(elements[start..end].to_vec())
            }
            "push" => {
                if args.is_empty() { return Value::Null; }
                let new_val = self.eval(args[0].clone());
                let mut new_arr = elements.to_vec();
                new_arr.push(new_val);
                let new_array = Value::Array(new_arr);
                if self.local_vars.contains_key(object) { self.local_vars.insert(object.to_string(), new_array.clone()); }
                else { self.global_vars.insert(object.to_string(), new_array.clone()); }
                new_array
            }
            "pop" => {
                let mut new_arr = elements.to_vec();
                let popped = if new_arr.is_empty() { Value::Null } else { new_arr.pop().unwrap_or(Value::Null) };
                let new_array = Value::Array(new_arr);
                if self.local_vars.contains_key(object) { self.local_vars.insert(object.to_string(), new_array); }
                else { self.global_vars.insert(object.to_string(), new_array); }
                popped
            }
            "reverse" => {
                let mut new_arr = elements.to_vec();
                new_arr.reverse();
                let new_array = Value::Array(new_arr);
                if self.local_vars.contains_key(object) { self.local_vars.insert(object.to_string(), new_array.clone()); }
                else { self.global_vars.insert(object.to_string(), new_array.clone()); }
                new_array
            }
            "sort" => {
                let mut new_arr = elements.to_vec();
                new_arr.sort_by(|a, b| match (a, b) {
                    (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                    (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                    (Value::StringVal(x), Value::StringVal(y)) => x.cmp(y),
                    _ => std::cmp::Ordering::Equal,
                });
                let new_array = Value::Array(new_arr);
                if self.local_vars.contains_key(object) { self.local_vars.insert(object.to_string(), new_array.clone()); }
                else { self.global_vars.insert(object.to_string(), new_array.clone()); }
                new_array
            }
            "clear" => {
                let empty = Value::Array(Vec::new());
                if self.local_vars.contains_key(object) { self.local_vars.insert(object.to_string(), empty.clone()); }
                else { self.global_vars.insert(object.to_string(), empty.clone()); }
                empty
            }
            _ => Value::Null,
        }
    }

    pub fn eval_index_access(&mut self, name: String, index: Node) -> Value {
        let idx = match self.eval(index) { Value::Integer(i) => i as usize, _ => return Value::Null };
        let arr = if let Some(val) = self.local_vars.get(&name) { val.clone() }
            else if let Some(val) = self.global_vars.get(&name) { val.clone() }
            else if let Some(val) = self.constants.get(&name) { val.clone() }
            else { return Value::Error(format!("'{}' was never declared", name)); };
        match arr {
            Value::Array(elements) => {
                if idx < elements.len() { elements[idx].clone() }
                else { Value::Error(format!("Index [{}] out of bounds.", idx)) }
            }
            _ => Value::Error(format!("'{}' is not an array.", name)),
        }
    }
}