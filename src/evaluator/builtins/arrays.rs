//! Methods for reading and mutating evaluator arrays.
use crate::evaluator::value::Value;
use crate::evaluator::Evaluator;
use crate::parser::Node;

impl Evaluator {
    pub fn eval_array_method(
        &mut self,
        object: &str,
        method: &str,
        args: Vec<Node>,
    ) -> Value {
        let arr_val = self.lookup(object);
        let arr = match arr_val {
            Value::Array(a) => a,
            Value::Error(e) => return Value::Error(e),
            _ => return Value::Error(format!("Nice try, but '{object}' is not an array. That's not a collection; that's just a dramatic variable.")),
        };

        match method {
            "len" => {
                let v = arr.read().unwrap();
                Value::Integer(v.len() as i64)
            }
            "first" => {
                let v = arr.read().unwrap();
                v.first().cloned().unwrap_or(Value::Null)
            }
            "last" => {
                let v = arr.read().unwrap();
                v.last().cloned().unwrap_or(Value::Null)
            }
            "isEmpty" => {
                let v = arr.read().unwrap();
                Value::Boolean(v.is_empty())
            }
            "has" => {
                if args.is_empty() {
                    return Value::Null;
                }
                let target = self.eval(args[0].clone());
                let v = arr.read().unwrap();
                Value::Boolean(v.iter().any(|x| *x == target))
            }
            "indexOf" => {
                if args.is_empty() {
                    return Value::Integer(-1);
                }
                let target = self.eval(args[0].clone());
                let v = arr.read().unwrap();
                for (i, x) in v.iter().enumerate() {
                    if *x == target {
                        return Value::Integer(i as i64);
                    }
                }
                Value::Integer(-1)
            }
            "join" => {
                let sep = if args.is_empty() {
                    "".to_string()
                } else {
                    match self.eval(args[0].clone()) {
                        Value::StringVal(s) => s,
                        other => other.to_string(),
                    }
                };
                let v = arr.read().unwrap();
                let s = v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(&sep);
                Value::StringVal(s)
            }
            "slice" => {
                if args.len() < 2 {
                    return Value::Null;
                }
                let start = match self.eval(args[0].clone()) {
                    Value::Integer(i) if i >= 0 => i as usize,
                    _ => 0,
                };
                let end = match self.eval(args[1].clone()) {
                    Value::Integer(i) if i >= 0 => i as usize,
                    _ => {
                        let v = arr.read().unwrap();
                        v.len()
                    }
                };
                let v = arr.read().unwrap();
                if start > end || end > v.len() {
                    return Value::array(vec![]);
                }
                Value::array(v[start..end].to_vec())
            }

            // Mutating methods operate on the shared handle and return that same
            // handle where the language exposes the array as an expression.
            "push" => {
                if args.is_empty() {
                    return Value::Null;
                }
                let new_val = self.eval(args[0].clone());
                {
                    let mut v = arr.write().unwrap();
                    v.push(new_val);
                }
                Value::Array(arr)
            }
            "pop" => {
                let mut v = arr.write().unwrap();
                v.pop().unwrap_or(Value::Null)
            }
            "reverse" => {
                {
                    let mut v = arr.write().unwrap();
                    v.reverse();
                }
                Value::Array(arr)
            }
            "sort" => {
                {
                    let mut v = arr.write().unwrap();
                    v.sort_by(|a, b| match (a, b) {
                        (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                        (Value::Float(x), Value::Float(y)) => {
                            x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        (Value::StringVal(x), Value::StringVal(y)) => x.cmp(y),
                        _ => std::cmp::Ordering::Equal,
                    });
                }
                Value::Array(arr)
            }
            "clear" => {
                {
                    let mut v = arr.write().unwrap();
                    v.clear();
                }
                Value::Array(arr)
            }
            _ => Value::Error(format!("'{method}' is not a real array method. Even a toaster has better ideas.")),
        }
    }

    pub fn eval_index_access(&mut self, name: String, index: Node) -> Value {
        let idx = match self.eval(index) {
            Value::Integer(i) if i >= 0 => i as usize,
            _ => return Value::Error("Index needs to be a non-negative integer. Negative indexes are chaos, not code.".into()),
        };

        match self.lookup(&name) {
            Value::Array(arr) => {
                let v = arr.read().unwrap();
                if idx < v.len() {
                    v[idx].clone()
                } else {
                    Value::Error(format!("Index [{idx}] is so far out of bounds it needs a map and a prayer."))
                }
            }
            Value::Error(e) => Value::Error(e),
            _ => Value::Error(format!("'{name}' is not an array. That's not a list; that's just vibes.")),
        }
    }
}