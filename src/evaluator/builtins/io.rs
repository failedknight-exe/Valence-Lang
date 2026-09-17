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
    match io::stdin().read_line(&mut input) {
        Ok(0) => {
            // EOF or empty read: return empty string, NEVER Break!
            Value::StringVal(String::new())
        }
        Ok(_) => {
            let clean = input.trim().to_string();
            Value::StringVal(clean)
        }
        Err(_) => Value::StringVal(String::new()),
    }
}
}