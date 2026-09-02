use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;
use std::io::{self, Write};

impl Evaluator {
    pub fn eval_print(&mut self, expr: Node) -> Value {
        let val = self.eval(expr);
        if let Value::Error(msg) = &val {
            Value::Error(msg.clone())
        } else {
            println!("{}", val);
            val
        }
    }

    pub fn eval_input(&mut self, prompt: Node) -> Value {
        let prompt_val = self.eval(prompt);
        print!("{}", prompt_val);
        let _ = io::stdout().flush();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let input = input.trim().to_string();
        if let Ok(n) = input.parse::<i64>() { return Value::Integer(n); }
        if let Ok(f) = input.parse::<f64>() { return Value::Float(f); }
        if input == "true" { return Value::Boolean(true); }
        if input == "false" { return Value::Boolean(false); }
        Value::StringVal(input)
    }
}