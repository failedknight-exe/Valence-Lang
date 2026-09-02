use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;

impl Evaluator {
    pub fn eval_math_builtin(&mut self, method: &str, args: Vec<Node>) -> Value {
        match method {
            "sqrt" => { let n = self.eval_to_float(&args); Value::Float(n.sqrt()) }
            "abs" => {
                if args.is_empty() { return Value::Null; }
                match self.eval(args[0].clone()) {
                    Value::Integer(n) => Value::Integer(n.abs()),
                    Value::Float(f) => Value::Float(f.abs()),
                    _ => Value::Null,
                }
            }
            "floor" => { let n = self.eval_to_float(&args); Value::Integer(n.floor() as i64) }
            "ceil" => { let n = self.eval_to_float(&args); Value::Integer(n.ceil() as i64) }
            "round" => { let n = self.eval_to_float(&args); Value::Integer(n.round() as i64) }
            "random" => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as f64;
                let r = (seed % 1000.0) / 1000.0;
                if args.len() >= 2 {
                    let min = match self.eval(args[0].clone()) { Value::Integer(n) => n as f64, Value::Float(f) => f, _ => 0.0 };
                    let max = match self.eval(args[1].clone()) { Value::Integer(n) => n as f64, Value::Float(f) => f, _ => 1.0 };
                    Value::Integer((min + r * (max - min)) as i64)
                } else {
                    Value::Float(r)
                }
            }
            "max" => {
                if args.len() < 2 { return Value::Null; }
                let a = self.eval(args[0].clone());
                let b = self.eval(args[1].clone());
                match (&a, &b) {
                    (Value::Integer(x), Value::Integer(y)) => Value::Integer(*x.max(y)),
                    (Value::Float(x), Value::Float(y)) => Value::Float(x.max(*y)),
                    _ => Value::Null,
                }
            }
            "min" => {
                if args.len() < 2 { return Value::Null; }
                let a = self.eval(args[0].clone());
                let b = self.eval(args[1].clone());
                match (&a, &b) {
                    (Value::Integer(x), Value::Integer(y)) => Value::Integer(*x.min(y)),
                    (Value::Float(x), Value::Float(y)) => Value::Float(x.min(*y)),
                    _ => Value::Null,
                }
            }
            "pow" => {
                if args.len() < 2 { return Value::Null; }
                let base = self.eval_to_float(&args[0..1].to_vec());
                let exp = self.eval_to_float(&args[1..2].to_vec());
                Value::Float(base.powf(exp))
            }
            "sin" => { let n = self.eval_to_float(&args); Value::Float(n.sin()) }
            "cos" => { let n = self.eval_to_float(&args); Value::Float(n.cos()) }
            "tan" => { let n = self.eval_to_float(&args); Value::Float(n.tan()) }
            "log" => { let n = self.eval_to_float(&args); Value::Float(n.ln()) }
            "log10" => { let n = self.eval_to_float(&args); Value::Float(n.log10()) }
            "pi" => Value::Float(std::f64::consts::PI),
            "e" => Value::Float(std::f64::consts::E),
            _ => Value::Null,
        }
    }

    pub fn eval_to_float(&mut self, args: &Vec<Node>) -> f64 {
        if args.is_empty() { return 0.0; }
        match self.eval(args[0].clone()) {
            Value::Integer(n) => n as f64,
            Value::Float(f) => f,
            _ => 0.0,
        }
    }
}