use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;

impl Evaluator {
    pub fn eval_typeof(&mut self, expr: Node) -> Value {
        let val = self.eval(expr);
        let name = match val {
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::StringVal(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Array(_) => "array",
            Value::Map(_) => "map",
            Value::Null => "null",
            Value::Return(_) => "return",
            Value::Break => "break",
            Value::Continue => "continue",
            Value::Error(_) => "error",
        };
        Value::StringVal(name.to_string())
    }

    pub fn eval_to_int(&mut self, expr: Node) -> Value {
        let val = self.eval(expr);
        match val {
            Value::Integer(n) => Value::Integer(n),
            Value::Float(f) => Value::Integer(f as i64),
            Value::StringVal(s) => match s.parse::<i64>() {
                Ok(n) => Value::Integer(n),
                Err(_) => Value::Error(format!("Cannot convert '{}' to integer. That's not a number; that's a personality test.", s)),
            },
            Value::Boolean(b) => Value::Integer(if b { 1 } else { 0 }),
            _ => Value::Null,
        }
    }

    pub fn eval_to_float_expr(&mut self, expr: Node) -> Value {
        let val = self.eval(expr);
        match val {
            Value::Float(f) => Value::Float(f),
            Value::Integer(n) => Value::Float(n as f64),
            Value::StringVal(s) => match s.parse::<f64>() {
                Ok(f) => Value::Float(f),
                Err(_) => Value::Error(format!("Cannot convert '{}' to float. This value is less stable than a cardboard bridge.", s)),
            },
            Value::Boolean(b) => Value::Float(if b { 1.0 } else { 0.0 }),
            _ => Value::Null,
        }
    }

    pub fn eval_to_string(&mut self, expr: Node) -> Value {
        let val = self.eval(expr);
        Value::StringVal(format!("{}", val))
    }

    pub fn eval_to_bool(&mut self, expr: Node) -> Value {
        let val = self.eval(expr);
        match val {
            Value::Boolean(b) => Value::Boolean(b),
            Value::Integer(n) => Value::Boolean(n != 0),
            Value::Float(f) => Value::Boolean(f != 0.0),
            Value::StringVal(s) => Value::Boolean(!s.is_empty()),
            Value::Null => Value::Boolean(false),
            _ => Value::Boolean(true),
        }
    }
}