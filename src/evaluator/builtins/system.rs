use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;

impl Evaluator {
    pub fn eval_system_builtin(&mut self, method: &str, args: Vec<Node>) -> Value {
        match method {
            "os" => {
                if cfg!(target_os = "windows") { Value::StringVal("windows".to_string()) }
                else if cfg!(target_os = "linux") { Value::StringVal("linux".to_string()) }
                else if cfg!(target_os = "macos") { Value::StringVal("mac".to_string()) }
                else { Value::StringVal("unknown".to_string()) }
            }
            "arch" => Value::StringVal(std::env::consts::ARCH.to_string()),
            "env" => {
                if args.is_empty() { return Value::Null; }
                let key = match self.eval(args[0].clone()) { Value::StringVal(s) => s, _ => return Value::Null };
                match std::env::var(&key) {
                    Ok(val) => Value::StringVal(val),
                    Err(_) => Value::Null,
                }
            }
            "exec" => {
                if args.is_empty() { return Value::Null; }
                let cmd = match self.eval(args[0].clone()) { Value::StringVal(s) => s, _ => return Value::Null };
                let output = if cfg!(target_os = "windows") {
                    std::process::Command::new("cmd").args(&["/C", &cmd]).output()
                } else {
                    std::process::Command::new("sh").args(&["-c", &cmd]).output()
                };
                match output {
                    Ok(out) => Value::StringVal(String::from_utf8_lossy(&out.stdout).to_string()),
                    Err(e) => Value::Error(format!("exec failed: {}", e)),
                }
            }
            "cwd" => match std::env::current_dir() {
                Ok(p) => Value::StringVal(p.to_string_lossy().to_string()),
                Err(_) => Value::Null,
            },
            "exit" => {
                let code = if args.is_empty() { 0 } else {
                    match self.eval(args[0].clone()) { Value::Integer(n) => n as i32, _ => 0 }
                };
                std::process::exit(code);
            }
            "sleep" => {
                if args.is_empty() { return Value::Null; }
                let ms = match self.eval(args[0].clone()) { Value::Integer(n) => n as u64, _ => return Value::Null };
                std::thread::sleep(std::time::Duration::from_millis(ms));
                Value::Null
            }
            "args" => {
                let args_vec: Vec<Value> = std::env::args().skip(1).map(Value::StringVal).collect();
                Value::Array(args_vec)
            }
            _ => Value::Error(format!("'{}' is not a valid system method", method)),
        }
    }
}