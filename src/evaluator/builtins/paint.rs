//! ANSI styling and progress-style terminal output builtins.

use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;

impl Evaluator {
    pub fn eval_paint_builtin(&mut self, method: &str, args: Vec<Node>) -> Value {
        if args.is_empty() {
            return Value::StringVal("".to_string());
        }

        let text = format!("{}", self.eval(args[0].clone()));

        match method {
            // Each style is reset after the value so formatting does not leak into
            // text printed by subsequent runtime operations.
            "red"     => Value::StringVal(format!("\x1b[31m{}\x1b[0m", text)),
            "green"   => Value::StringVal(format!("\x1b[32m{}\x1b[0m", text)),
            "yellow"  => Value::StringVal(format!("\x1b[33m{}\x1b[0m", text)),
            "blue"    => Value::StringVal(format!("\x1b[34m{}\x1b[0m", text)),
            "magenta" => Value::StringVal(format!("\x1b[35m{}\x1b[0m", text)),
            "cyan"    => Value::StringVal(format!("\x1b[36m{}\x1b[0m", text)),
            "bold"    => Value::StringVal(format!("\x1b[1m{}\x1b[0m", text)),

            // The task is evaluated synchronously; the status messages therefore
            // bracket the actual work rather than merely announcing a spawn.
            "spin" => {
                println!("\x1b[36mStarting: {}\x1b[0m", text);
                if args.len() > 1 {
                    let task_node = args[1].clone();
                    self.eval(task_node);
                }
                println!("\x1b[32mCompleted: {}\x1b[0m", text);
                Value::Boolean(true)
            }

            _ => Value::Error(format!("Unknown paint method '{method}'. That color trick has less style than a burnt toaster.")),
        }
    }
}