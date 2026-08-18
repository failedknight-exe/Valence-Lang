//! Runtime evaluator implementation for the Connect language.

use crate::parser::Node;
use std::collections::{HashMap, HashSet};
use std::io::{self, Read, Write};

/// Runtime values supported by the language.
#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    StringVal(String),
    Boolean(bool),
    Array(Vec<Value>),
    Map(std::collections::HashMap<String, Value>),
    Null,
    Return(Box<Value>),
    Break,
    Continue,
    Error(String),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::StringVal(a), Value::StringVal(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Break, Value::Break) => true,
            (Value::Continue, Value::Continue) => true,
            (Value::Error(a), Value::Error(b)) => a == b,
            _ => false,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::StringVal(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "| |"),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", items.join(", "))
                
            }
            Value::Map(map) => {
                let items: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k,v))
                .collect();
            write!(f, "{{{}}}", items.join(","))
            }
            Value::Return(v) => write!(f, "{}", v),
            Value::Break => write!(f, ""),
            Value::Continue => write!(f, ""),
            Value::Error(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug, Clone)]
/// Stored function metadata for user-defined functions.
///
/// `func_type` distinguishes normal, auto-scoped, and one-time functions.
/// Parameters and the AST body are evaluated when the function is called.
pub struct StoredFunc {
    pub func_type: String,
    pub params: Vec<String>,
    pub body: Vec<Node>,
}

/// Interpreter state for the Connect language.
///
/// Tracks local/global variables, constants, functions, and one-time call usage.
pub struct Evaluator {
    local_vars: HashMap<String, Value>,
    global_vars: HashMap<String, Value>,
    constants: HashMap<String, Value>,
    functions: HashMap<String, StoredFunc>,
    parent_locals: Vec<HashMap<String, Value>>,
    when_triggers: Vec<(String, Node, bool)>,
}

impl Evaluator {
    /// Create a new evaluator with empty state.
    pub fn new() -> Self {
        Evaluator {
            local_vars: HashMap::new(),
            global_vars: HashMap::new(),
            constants: HashMap::new(),
            functions: HashMap::new(),
            parent_locals: Vec::new(),
            when_triggers: Vec::new(),
        }
    }

    pub fn fork(&self) -> Evaluator {
        Evaluator {
            local_vars: self.local_vars.clone(),
            global_vars: self.global_vars.clone(),
            constants: self.constants.clone(),
            functions: self.functions.clone(),
            parent_locals: self.parent_locals.clone(),
            when_triggers: self.when_triggers.clone(),
        }
    }

    pub fn check_triggers(&mut self) {
        let triggers = self.when_triggers.clone();
        let mut to_fire: Vec<(String, bool)> = Vec::new();
        for (name, condition, is_once) in triggers.iter() {
            let result = self.eval(condition.clone());
            if let Value::Boolean(true) = result {
                to_fire.push((name.clone(), *is_once));
            }
        }
        let fired_once: Vec<String> = to_fire.iter()
            .filter(|(_, is_once)| *is_once)
            .map(|(name, _)| name.clone())
            .collect();
        self.when_triggers.retain(|(name, _, _)| {
            !fired_once.contains(name)
        });
        for (name, _) in to_fire {
            if let Some(func) = self.functions.get(&name).cloned() {
                let saved = self.local_vars.clone();
                self.parent_locals.push(saved.clone());
                self.local_vars = std::collections::HashMap::new();
                for node in func.body {
                    self.eval(node);
                }
                self.parent_locals.pop();
                self.local_vars = saved;
            }
        }
    }

    /// Execute a list of top-level AST nodes.
   pub fn run(&mut self, nodes: Vec<Node>) {
    for node in nodes {
        let result = self.eval(node);
        if let Value::Error(msg) = result {
            println!("[RUNTIME ERROR] {}", msg);
        }
    }
}
    /// Evaluate a single AST node and return its runtime value.
    pub fn eval(&mut self, node: Node) -> Value {
        match node {
            // Variable and constant declarations
            Node::VarLDecl { name, value } => {
                let val = self.eval(*value);
                self.local_vars.insert(name, val.clone());
                val
            }

            Node::VarGDecl { name, value } => {
                let val = self.eval(*value);
                self.global_vars.insert(name, val.clone());
                self.check_triggers();
                val
            }

            Node::ConstDecl { name, value } => {
                if self.constants.contains_key(&name) {
                    println!("[RUNTIME ERROR] Constant '{}' already exists.", name);
                    return Value::Null;
                }
                let val = self.eval(*value);
                self.constants.insert(name, val.clone());
                val
            }

            Node::UpdateDecl { name, value } => {
                if self.constants.contains_key(&name) {
                    return Value::Error(format!("'{}' is a constant. Can not change. Use varG instead.", name));
                }
                let val = self.eval(*value);
                if self.local_vars.contains_key(&name) {
                    self.local_vars.insert(name, val.clone());
                } else if self.global_vars.contains_key(&name) {
                    self.global_vars.insert(name, val.clone());
                    self.check_triggers();
                } else {
                    return Value::Error(format!("'{}' doesn't exist. Can not update what was never declared.", name));
                }
                val
            }

            Node::UpdateIndex { name, index, value } => {
                let idx = match self.eval(*index) {
                    Value::Integer(i) => i as usize,
                    _ => return Value::Null,
                };
                let val = self.eval(*value);
                if let Some(arr) = self.local_vars.get_mut(&name) {
                    if let Value::Array(elements) = arr {
                        if idx < elements.len() {
                            elements[idx] = val.clone();
                        }
                    }
                } else if let Some(arr) = self.global_vars.get_mut(&name) {
                    if let Value::Array(elements) = arr {
                        if idx < elements.len() {
                            elements[idx] = val.clone();
                        }
                    }
                }
                val
            }

            Node::VarAccess(name) => {
                if let Some(val) = self.local_vars.get(&name) {
                    return val.clone();
                }
                if let Some(val) = self.global_vars.get(&name) {
                    return val.clone();
                }
                if let Some(val) = self.constants.get(&name) {
                    return val.clone();
                }
                Value::Error(format!("'{}' was never declared. Check spelling or use varL/varG.", name))
            }

            Node::Summon(name) => {
    if let Some(val) = self.local_vars.get(&name) {
        return val.clone();
    }
    for (i, parent) in self.parent_locals.iter().rev().enumerate() {
        if let Some(val) = parent.get(&name) {
            self.local_vars.insert(name.clone(), val.clone());
            return val.clone();
        }
    }
    if let Some(val) = self.global_vars.get(&name) {
        return val.clone();
    }
    if let Some(val) = self.constants.get(&name) {
        return val.clone();
    }
    Value::Error(format!("'{}' could not be summoned. Not in any scope.", name))
}

            Node::Print(expr) => {
    let val = self.eval(*expr);
    match val {
        Value::Error(ref msg) => {
            return Value::Error(msg.clone());
        }
        _ => {
            println!("{}", val);
            val
        }
    }
}

            Node::InputExpr(prompt) => {
                let prompt_val = self.eval(*prompt);
                print!("{}", prompt_val);
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim().to_string();
                if let Ok(n) = input.parse::<i64>() { return Value::Integer(n); }
                if let Ok(f) = input.parse::<f64>() { return Value::Float(f); }
                if input == "true" { return Value::Boolean(true); }
                if input == "false" { return Value::Boolean(false); }
                Value::StringVal(input)
            }

            Node::TypeOf(expr) => {
                let val = self.eval(*expr);
                let type_name = match val {
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
                    _ => "unknown",
                };
                Value::StringVal(type_name.to_string())
            }

            Node::ToInt(expr) => {
                let val = self.eval(*expr);
                match val {
                    Value::Integer(n) => Value::Integer(n),
                    Value::Float(f) => Value::Integer(f as i64),
                    Value::StringVal(s) => match s.parse::<i64>() {
                        Ok(n) => Value::Integer(n),
                        Err(_) => Value::Error(format!("Can not convert '{}' to integer.", s)),
                    },
                    Value::Boolean(b) => Value::Integer(if b { 1 } else { 0 }),
                    _ => Value::Null,
                }
            }

            Node::ToFloat(expr) => {
                let val = self.eval(*expr);
                match val {
                    Value::Float(f) => Value::Float(f),
                    Value::Integer(n) => Value::Float(n as f64),
                    Value::StringVal(s) => match s.parse::<f64>() {
                        Ok(f) => Value::Float(f),
                        Err(_) => Value::Error(format!("Can not convert '{}' to float.", s)),
                    },
                    _ => Value::Null,
                }
            }

            Node::ToString(expr) => {
                let val = self.eval(*expr);
                Value::StringVal(format!("{}", val))
            }

            Node::ToBool(expr) => {
                let val = self.eval(*expr);
                match val {
                    Value::Boolean(b) => Value::Boolean(b),
                    Value::Integer(n) => Value::Boolean(n != 0),
                    Value::StringVal(s) => Value::Boolean(!s.is_empty()),
                    Value::Null => Value::Boolean(false),
                    _ => Value::Boolean(true),
                }
            }

            Node::MathCall { method, args } => {
                match method.as_str() {
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

            Node::FileCall { method, args } => {
                match method.as_str() {
                    "read" => {
                        if args.is_empty() { return Value::Null; }
                        let path = match self.eval(args[0].clone()) { Value::StringVal(s) => s, _ => return Value::Null };
                        match std::fs::read_to_string(&path) {
                            Ok(content) => Value::StringVal(content),
                            Err(_) => Value::Error(format!("Could not read file '{}'.", path)),
                        }
                    }
                    "write" => {
                        if args.len() < 2 { return Value::Null; }
                        let path = match self.eval(args[0].clone()) { Value::StringVal(s) => s, _ => return Value::Null };
                        let content = format!("{}", self.eval(args[1].clone()));
                        match std::fs::write(&path, content) {
                            Ok(_) => Value::Boolean(true),
                            Err(_) => Value::Error(format!("Could not write to'{}'.", path)),
                        }
                    }
                    "append" => {
                        if args.len() < 2 { return Value::Null; }
                        let path = match self.eval(args[0].clone()) { Value::StringVal(s) => s, _ => return Value::Null };
                        let content = format!("{}", self.eval(args[1].clone()));
                        use std::io::Write as IoWrite;
                        match std::fs::OpenOptions::new().append(true).create(true).open(&path) {
                            Ok(mut f) => { let _ = f.write_all(content.as_bytes()); Value::Boolean(true) }
                            Err(_) => Value::Error(format!("Could not append to '{}'.", path)),
                        }
                    }
                    "exists" => {
                        if args.is_empty() { return Value::Boolean(false); }
                        let path = match self.eval(args[0].clone()) { Value::StringVal(s) => s, _ => return Value::Boolean(false) };
                        Value::Boolean(std::path::Path::new(&path).exists())
                    }
                    "delete" => {
                        if args.is_empty() { return Value::Boolean(false); }
                        let path = match self.eval(args[0].clone()) { Value::StringVal(s) => s, _ => return Value::Boolean(false) };
                        match std::fs::remove_file(&path) {
                            Ok(_) => Value::Boolean(true),
                            Err(_) => Value::Boolean(false),
                        }
                    }
                    _ => Value::Null,
                }
            }

            Node::JsonCall { method, args } => {
                match method.as_str() {
                    "stringify" => {
                        if args.is_empty() { return Value::Null; }
                        let val = self.eval(args[0].clone());
                        match val {
                            Value::Map(ref map) => {
                                let pairs: Vec<String> = map 
                                    .iter()
                                    .map(|(k, v)| {
                                        let val_str = match v {
                                            Value::StringVal(s) => format!("\"{}\"", s),
                                            Value::Integer(n) => format!("{}", n),
                                            Value::Float(f) => format!("{}", f),
                                            Value::Boolean(b) => format!("{}", b),
                                            Value::Null => "null".to_string(),
                                            other => format!("{}", other),
                                        };
                                        format!("\"{}\":{}", k, val_str)
                                    })
                                    .collect();
                                Value::StringVal(format!("{{{}}}", pairs.join(",")))
                            }
                            Value::Array (ref arr) => {
                                let items: Vec<String> = arr
                                    .iter()
                                    .map(|v| match v {
                                        Value::StringVal(s) => format!("\"{}\"", s),
                                        other => format!("{}", other)
                                    })
                                    .collect();
                                Value::StringVal(format!("[{}]", items.join(",")))
                            }
                            other => Value::StringVal(format!("{}", other))
                        }
                    }
                    "parse" => {
                        if args.is_empty() { return Value::Null; }
                        let text = match self.eval(args[0].clone()) {
                            Value::StringVal(s) => s,
                            _ => return Value::Error("json.parse needs a string".to_string()),
                        };
                        let trimmed = text.trim();
                        if trimmed.starts_with('{') {
                            let mut map = std::collections::HashMap::new();
                            let inner = &trimmed[1..trimmed.len() - 1];
                            for pair in inner.split(',') {
                                let kv: Vec<&str> = pair.splitn(2, ':').collect();
                                if kv.len() == 2 {
                                    let key = kv[0].trim().trim_matches('"').to_string();
                                    let val_str = kv[1].trim();
                                    let val = if val_str.starts_with('"') {
                                        Value::StringVal(val_str.trim_matches('"').to_string())
                                    } else if val_str == "true" {
                                        Value::Boolean(true)
                                    } else if val_str == "false" {
                                        Value::Boolean(false)
                                    } else if val_str == "null" {
                                        Value::Null
                                    } else if let Ok(n) = val_str.parse::<i64>() {
                                        Value::Integer(n)
                                    } else if let Ok(f) = val_str.parse::<f64>() {
                                        Value::Float(f)
                                    } else {
                                        Value::StringVal(val_str.to_string())
                                    };
                                    map.insert(key, val);
                                    }
                                }
                                Value::Map(map)
                            } else {
                                Value::Error("json.parse: invalid format".to_string())
                            }
                        }
                        _ => Value::Error(format!("'{}' is not a valid json method", method)),
                    }
                }
            
            Node::DateCall { method, args } => {
                    match method.as_str() {
                        "now" => {
                            let now = chrono::Local::now();
                            Value::StringVal(now.format("%Y-%m-%d %H:%M:%S").to_string())
                        }
                        "today" => {
                            let now = chrono::Local::now();
                            Value::StringVal(now.format("%Y-%m-%d").to_string())
                        }
                        "time" => {
                            let now = chrono::Local::now();
                            Value::StringVal(now.format("%H:%M:%S").to_string())
                        }
                        "year" => {
                            let now = chrono::Local::now();
                            Value::Integer(now.format("%Y").to_string().parse().unwrap_or(0))
                        }
                        "month" => {
                            let now = chrono::Local::now();
                            Value::Integer(now.format("%m").to_string().parse().unwrap_or(0))
                        }
                        "day" => {
                            let now = chrono::Local::now();
                            Value::Integer(now.format("%d").to_string().parse().unwrap_or(0))
                        }
                        "hour" => {
                            let now = chrono::Local::now();
                            Value::Integer(now.format("%H").to_string().parse().unwrap_or(0))
                        }
                        "minute" => {
                            let now = chrono::Local::now();
                            Value::Integer(now.format("%M").to_string().parse().unwrap_or(0))
                        }
                        "second" => {
                            let now = chrono::Local::now();
                            Value::Integer(now.format("%S").to_string().parse().unwrap_or(0))
                        }
                        "timestamp" => {
                            let now = chrono::Local::now();
                            Value::Integer(now.timestamp())
                        }
                        "format" => {
                            if args.is_empty() { return Value::Null; }
                            let fmt = match self.eval(args[0].clone()) {
                                Value::StringVal(s) => s,
                                _ => return Value::Null,
                            };
                            let now = chrono::Local::now();
                            Value::StringVal(now.format(&fmt).to_string())
                        }
                        "dayName" => {
                            let now = chrono::Local::now();
                            Value::StringVal(now.format("%A").to_string())
                        }
                        "monthName" => {
                            let now = chrono::Local::now();
                            Value::StringVal(now.format("%B").to_string())
                        }
                        _ => Value::Error(format!("'{}' is not a valid date method", method)),
                    }
                }

            Node::SystemCall { method, args } => {
                match method.as_str() {
                    "os" => {
                        if cfg!(target_os = "windows") {
                            Value::StringVal("windows".to_string())
                        } else if cfg!(target_os = "linux") {
                            Value::StringVal("linux".to_string())
                        } else if cfg!(target_os = "macos") {
                            Value::StringVal("mac".to_string())
                        } else {
                            Value::StringVal("unknown".to_string())
                        }
                    }
                    "arch" => {
                        Value::StringVal(std::env::consts::ARCH.to_string())
                    }
                    "env" => {
                        if args.is_empty() { return Value::Null; }
                        let key = match self.eval(args[0].clone()) {
                            Value::StringVal(s) => s,
                            _ => return Value::Null,
                        };
                        match std::env::var(&key) {
                            Ok(val) => Value::StringVal(val),
                            Err(_) => Value::Null,
                        }
                    }
                    "exec" => {
                        if args.is_empty() { return Value::Null; }
                        let cmd = match self.eval(args[0].clone()) {
                            Value::StringVal(s) => s,
                            _ => return Value::Null,
                        };
                        let output = if cfg!(target_os = "windows") {
                            std::process::Command::new("cmd")
                                .args(&["/C", &cmd])
                                .output()
                        } else {
                            std::process::Command::new("sh")
                                .args(&["-c", &cmd])
                                .output()
                        };
                        match output {
                            Ok(out) => {
                                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                                Value::StringVal(stdout)
                            }
                            Err(e) => Value::Error(format!("exec failed: {}", e)),
                        }
                    }
                    "cwd" => {
                        match std::env::current_dir() {
                            Ok(path) => Value::StringVal(path.to_string_lossy().to_string()),
                            Err(_) => Value::Null,
                        }
                    }
                    "exit" => {
                        let code = if args.is_empty() {
                            0
                        } else {
                            match self.eval(args[0].clone()) {
                                Value::Integer(n) => n as i32,
                                _ => 0,
                            }
                        };
                        std::process::exit(code)
                    }
                    "sleep" => {
                        if args.is_empty() { return Value::Null; }
                        let ms = match self.eval(args[0].clone()) {
                            Value::Integer(n) => n as u64,
                            _ => return Value::Null,
                        };
                        std::thread::sleep(std::time::Duration::from_millis(ms));
                        Value::Null
                    }
                    "args" => {
                        let args_vec: Vec<Value> = std::env::args()
                            .skip(1)
                            .map(|a| Value::StringVal(a))
                            .collect();
                        Value::Array(args_vec)
                    }
                    _ => Value::Error(format!("'{}' is not a valid system method", method)),
                }
            }

            Node::HttpCall { method, args } => {
                match method.as_str() {
                    "get" => {
                        if args.is_empty() { return Value::Error("http.get needs a URL".to_string()); }
                        let url = match self.eval(args[0].clone()) {
                            Value::StringVal(s) => s,
                            _ => return Value::Error("http.get URL must be a string".to_string()),
                        };
                        match ureq::get(&url).call() {
                            Ok(response) => {
                                match response.into_string() {
                                    Ok(body) => Value::StringVal(body),
                                    Err(e) => Value::Error(format!("Failed to read response: {}", e)),
                                }
                            }
                            Err(e) => Value::Error(format!("HTTP GET failed: {}", e)),
                        }
                    }
                    "post" => {
                        if args.len() < 2 { return Value::Error("http.post needs URL and body".to_string()); }
                        let url = match self.eval(args[0].clone()) {
                            Value::StringVal(s) => s,
                            _ => return Value::Error("http.post URL must be a string".to_string()),
                        };
                        let body = match self.eval(args[1].clone()) {
                            Value::StringVal(s) => s,
                            other => format!("{}", other),
                        };
                        match ureq::post(&url)
                            .set("Content-Type", "application/json")
                            .send_string(&body) {
                            Ok(response) => {
                                match response.into_string() {
                                    Ok(body) => Value::StringVal(body),
                                    Err(e) => Value::Error(format!("Failed to read response: {}", e)),
                                }
                            }
                            Err(e) => Value::Error(format!("HTTP POST failed: {}", e)),
                        }
                    }
                    "status" => {
                        if args.is_empty() { return Value::Error("http.status needs a URL".to_string()); }
                        let url = match self.eval(args[0].clone()) {
                            Value::StringVal(s) => s,
                            _ => return Value::Error("URL must be a string".to_string()),
                        };
                        match ureq::get(&url).call() {
                            Ok(response) => Value::Integer(response.status() as i64),
                            Err(ureq::Error::Status(code, _)) => Value::Integer(code as i64),
                            Err(e) => Value::Error(format!("HTTP request failed: {}", e)),
                        }
                    }
                    "download" => {
                        if args.len() < 2 { return Value::Error("http.download needs URL and filename".to_string()); }
                        let url = match self.eval(args[0].clone()) {
                            Value::StringVal(s) => s,
                            _ => return Value::Error("URL must be a string".to_string()),
                        };
                        let filename = match self.eval(args[1].clone()) {
                            Value::StringVal(s) => s,
                            _ => return Value::Error("Filename must be a string".to_string()),
                        };
                        match ureq::get(&url).call() {
                            Ok(response) => {
                                let mut bytes = Vec::new();
                                match response.into_reader().read_to_end(&mut bytes) {
                                    Ok(_) => {
                                        match std::fs::write(&filename, &bytes) {
                                            Ok(_) => Value::Boolean(true),
                                            Err(e) => Value::Error(format!("Failed to save: {}", e)),
                                        }
                                    }
                                    Err(e) => Value::Error(format!("Failed to read: {}", e)),
                                }
                            }
                            Err(e) => Value::Error(format!("Download failed: {}", e)),
                        }
                    }
                    "put" => {
                        if args.len() < 2 { return Value::Error("http.put needsURL and body".to_string()); }
                        let url = match self.eval(args[0].clone()) {
                            Value::StringVal(s) => s,
                            _ => return Value::Error("URL must be a string".to_string()),
                        };
                        let body = match self.eval(args[1].clone()) {
                            Value::StringVal(s) => s,
                            other => format!("{}", other),
                        };
                        match ureq::put(&url)
                            .set("Content-Type", "application/json")
                            .send_string(&body) {
                            Ok(response) => {
                                match response.into_string() {
                                Ok(body) => Value::StringVal(body),
                                Err(e) => Value::Error(format!("Failed to read: {}", e)),
                                }
                            }
                            Err(e) => Value::Error(format!("HTTP PUT failed: {}", e))
                        }
                    }
                    "delete" => {
                        if args.is_empty() { return Value::Error("http.delete needs URL".to_string()); }
                        let url = match self.eval(args[0].clone()) {
                            Value::StringVal(s) => s,
                            _ => return Value::Error("URL must be a string".to_string()),
                        };
                        match ureq::delete(&url).call() {
                            Ok(response) => {
                                match response.into_string() {
                                    Ok(body) => Value::StringVal(body),
                                    Err(s) => Value::Error(format!("Failed to read: {}", s)),
                                }
                            }
                            Err(e) => Value::Error(format!("HTTP DELETE failed: {}", e)),
                        }
                    }
                    "getJson" => {
                        if args.is_empty() { return Value::Error("http.getJson needs URL".to_string()); }
                        let url = match self.eval(args[0].clone()) {
                            Value::StringVal(s) => s,
                            _ => return Value::Error("URL must be a string".to_string()),
                        };
                        match ureq::get(&url)
                            .set("Accept", "application/json")
                            .call() {
                            Ok(response) => {
                                match response.into_string() {
                                    Ok(body) => {
                                        let trimmed = body.trim();
                                        if trimmed.starts_with('{') {
                                            let mut map = std::collections::HashMap::new();
                                            let inner = &trimmed[1..trimmed.len() - 1];
                                            for pair in inner.split(',') {
                                                let kv: Vec<&str> = pair.splitn(2, ':').collect();
                                                if kv.len() == 2 {
                                                    let key = kv[0].trim().trim_matches('"').to_string();
                                                    let val_str = kv[1].trim();
                                                    let val = if val_str.starts_with('"') {
                                                        Value::StringVal(val_str.trim_matches('"').to_string())

                                                    } else if let Ok(n) = val_str.parse::<i64>() {
                                                        Value::Integer(n)
                                                    } else if let Ok(f) = val_str.parse::<f64>() {
                                                        Value::Float(f)
                                                    } else {
                                                        Value::StringVal(val_str.to_string())
                                                    };
                                                    map.insert(key, val);
                                                }
                                            }
                                            Value::Map(map)
                                        } else {
                                            Value::StringVal(body)
                                        }
                                    }
                                    Err(e) => Value::Error(format!("HTTP GET failed: {}", e))
                                }
                            }
                            Err(e) => Value::Error(format!("HTTP GET failed: {}", e))
                        }    
                    }
                    "getH" => {
                        if args.len() < 2 { return Value::Error("http.getH needs URL and headers map".to_string()); }
                        let url = match self.eval(args[0].clone()) {
                            Value::StringVal(s) => s,
                            _ => return Value::Error("URL must be a string".to_string()),
                        };
                        let headers = match self.eval(args[1].clone()) {
                            Value::Map(m) => m,
                            _ => return Value::Error("Headers must be a map".to_string()),
                        };
                        let mut req = ureq::get(&url);
                        for (key, val) in headers.iter() {
                            req = req.set(key, &format!("{}", val));
                        }
                        match req.call() {
                            Ok(response) => {
                                match response.into_string() {
                                    Ok(body) => Value::StringVal(body),
                                    Err(e) => Value::Error(format!("Failed to read: {}", e)),
                                }
                            }
                            Err(e) => Value::Error(format!("HTTP GET failed: {}", e)),
                        }
                    }
                    "postH" => {
                        if args.len() < 3 { return Value::Error("http.postH needsURL, body, headers".to_string()); }
                        let url = match self.eval(args[0].clone()) {
                            Value::StringVal(s) => s,
                            _ => return Value::Error("URL must be a string".to_string()),
                        };
                        let body = match self.eval(args[1].clone()) {
                            Value::StringVal(s) => s,
                            other => format!("{}", other),
                        };
                        let headers = match self.eval(args[2].clone()) {
                            Value::Map(m) => m,
                            _ => return Value::Error("Headers must be a map".to_string()),
                        };
                        let mut req = ureq::post(&url);
                        for (key, val) in headers.iter() {
                            req = req.set(key, &format!("{}", val));
                        }
                        match req.send_string(&body) {
                            Ok(response) => {
                                match response.into_string() {
                                    Ok(body) => Value::StringVal(body),
                                    Err(e) => Value::Error(format!("Failed to read: {}", e)),
                                }
                            }
                            Err(e) => Value::Error(format!("HTTP POST failed: {}", e)),
                        }
                    }
                    _ => Value::Error(format!("'{}' is not a valid http method", method)),
                }
            }

            Node::CryptoCall { method, args } => {
                match method.as_str() {
                    "hash" => {
                        if args.is_empty() { return Value::Error("crypto.hash needs data".to_string()); }
                        let data = match self.eval(args[0].clone()) {
                            Value::StringVal(s) => s,
                            other => format!("{}", other)
                        };
                        let algo = if args.len() > 1 {
                            match self.eval(args[1].clone()) {
                                Value::StringVal(s) => s,
                                _ => "sha256".to_string(),
                            }
                        } else {
                            "sha256".to_string()
                        };
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        data.hash(&mut hasher);
                        let hash_val = hasher.finish();
                        Value::StringVal(format!("{:016x}", hash_val))
                    }
                    "uuid" => {
                        use std::time::{SystemTime, UNIX_EPOCH};
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_nanos();
                        let rand1 = now % 0xFFFFFFFF;
                        let rand2 = (now >> 32) % 0xFFFFFFFF;
                        let rand3 = (now >> 64) % 0xFFFF;
                        let rand4 = (now >> 80) % 0xFFFF;
                        let rand5 = (now >> 96) % 0xFFFFFFFFFFFF;
                        Value::StringVal(format!(
                            "{:08x}-{:04x}-{:03x}-{:04x}-{:012x}",
                            rand1, rand2 % 0xFFFF, rand3 % 0xFFFF, rand4, rand5 
                        ))
                    }
                    "randomInt" => {
                        use std::time::{SystemTime, UNIX_EPOCH};
                    let seed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .subsec_nanos() as f64;
                    let r = (seed % 1000.0) / 1000.0;
                    if args.len() >= 2 {
                        let min = match self.eval(args[0].clone()) {
                            Value::Integer(n) => n as f64,
                            _ => 0.0,
                        };
                        let max = match self.eval(args[1].clone()) {
                            Value::Integer(n) => n as f64,
                            _ => 100.0,
                        };
                        Value::Integer((min + r * (max - min)) as i64)
                    } else {
                        Value::Integer((r * 100.0) as i64)
                    }
                    },
                    "randomBytes" => {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let count = if args.is_empty() {
                        16
                    } else {
                        match self.eval(args[0].clone()) {
                            Value::Integer(n) => n as usize,
                            _ => 16,
                        }
                    };
                    let mut result = String::new();
                    for i in 0..count {
                        let seed = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .subsec_nanos()
                            .wrapping_add(i as u32);
                        result.push_str(&format!("{:02x}", seed % 256));
                    }
                    Value::StringVal(result)
                }
                "base64Encode" => {
                    if args.is_empty() { return Value::Null; }
                    let data = match self.eval(args[0].clone()) {
                        Value::StringVal(s) => s,
                        other => format!("{}", other),
                    };
                    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
                    let bytes = data.as_bytes();
                    let mut result = String::new();
                    let mut i = 0;
                    while i < bytes.len() {
                        let b0 = bytes[i] as u32;
                        let b1 = if i + 1 < bytes.len() { bytes[i + 1] as u32 } else { 0 };
                        let b2 = if i + 2 < bytes.len() { bytes[i + 2] as u32 } else { 0 };
                        let triple = (b0 << 16) | (b1 << 8) | b2;
                        result.push(chars.as_bytes()[(((triple >> 18) & 0x3F) as usize)] as char);
                        result.push(chars.as_bytes()[(((triple >> 12) & 0x3F) as usize)] as char);
                        if i + 1 < bytes.len() {
                            result.push(chars.as_bytes()[(((triple >> 6) & 0x3F) as usize)] as char);
                        } else {
                            result.push('=');
                        }
                        if i + 2 < bytes.len() {
                            result.push(chars.as_bytes()[(triple & 0x3F) as usize] as char);
                        } else {
                            result.push('=');
                        }
                        i += 3;
                    }
                    Value::StringVal(result)
                }
                "base64Decode" => {
                    if args.is_empty() { return Value::Null; }
                    let data = match self.eval(args[0].clone()) {
                        Value::StringVal(s) => s,
                        _ => return Value::Null,
                    };
                    let table = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
                    let mut output = Vec::new();
                    let mut buffer: u32 = 0;
                    let mut bits: u8 = 0;
                    for c in data.chars() {
                        if c == '=' { break; }
                        if let Some(pos) = table.find(c) {
                            buffer = (buffer << 6) | pos as u32;
                            bits += 6;
                            if bits >= 8 {
                                bits -= 8;
                                output.push((buffer >> bits) as u8);
                                buffer &= (1 << bits) - 1;
                            }
                        }
                    }
                    Value::StringVal(String::from_utf8_lossy(&output).to_string())
                }
                _ => Value::Error(format!("'{}' is not a valid crypto method", method)),
            }
        }

            Node::UseModule(path) => {
                match std::fs::read_to_string(&path) {
                    Ok(source) => {
                        let mut lexer = crate::lexer::Lexer::new(&source);
                        let tokens = lexer.tokenize();
                        let mut parser = crate::parser::Parser::new(tokens);
                        let ast = parser.parse();
                        for node in ast { self.eval(node); }
                    }
                    Err(_) => { return Value::Error(format!("Could not load module '{}'.", path)); }
                }
                Value::Null
            }

            Node::Integer(n) => Value::Integer(n),
            Node::Float(f) => Value::Float(f),
            Node::StringLit(s) => Value::StringVal(s),
            Node::Boolean(b) => Value::Boolean(b),
            Node::Null => Value::Null,

            Node::Array(elements) => {
                let mut arr = Vec::new();
                for el in elements { arr.push(self.eval(el)); }
                Value::Array(arr)
            }

            Node::IndexAccess { name, index } => {
                let idx = match self.eval(*index) { Value::Integer(i) => i as usize, _ => return Value::Null };
                let arr = if let Some(val) = self.local_vars.get(&name) { val.clone() }
                    else if let Some(val) = self.global_vars.get(&name) { val.clone() }
                    else { return Value::Error(format!("'{}' was never declared", name)); };
                match arr {
                    Value::Array(elements) => {
                        if idx < elements.len() {
                            elements[idx].clone()
                        } else {
                            Value::Error(format!("Index [{}] out of bounds. Array has {} elements.", idx, elements.len()))
                        }
                    }
                    _ => Value::Error(format!("'{}' is not an array.", name)),
                }
            }

            // Object and array method dispatch
            Node::MethodCall { object, method, args } => {
                let val = if let Some(v) = self.local_vars.get(&object) { v.clone() }
                    else if let Some(v) = self.global_vars.get(&object) { v.clone() }
                    else { return Value::Null; };

                match val {
                    Value::Array(ref elements) => {
                        match method.as_str() {
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
                                let mut new_arr = elements.clone();
                                new_arr.push(new_val);
                                let new_array = Value::Array(new_arr);
                                if self.local_vars.contains_key(&object) { self.local_vars.insert(object, new_array.clone()); }
                                else { self.global_vars.insert(object, new_array.clone()); }
                                new_array
                            }
                            "pop" => {
                                let mut new_arr = elements.clone();
                                let popped = if new_arr.is_empty() { Value::Null } else { new_arr.pop().unwrap_or(Value::Null) };
                                let new_array = Value::Array(new_arr);
                                if self.local_vars.contains_key(&object) { self.local_vars.insert(object, new_array); }
                                else { self.global_vars.insert(object, new_array); }
                                popped
                            }
                            "reverse" => {
                                let mut new_arr = elements.clone();
                                new_arr.reverse();
                                let new_array = Value::Array(new_arr);
                                if self.local_vars.contains_key(&object) { self.local_vars.insert(object, new_array.clone()); }
                                else { self.global_vars.insert(object, new_array.clone()); }
                                new_array
                            }
                            "sort" => {
                                let mut new_arr = elements.clone();
                                new_arr.sort_by(|a, b| match (a, b) {
                                    (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                                    (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                                    (Value::StringVal(x), Value::StringVal(y)) => x.cmp(y),
                                    _ => std::cmp::Ordering::Equal,
                                });
                                let new_array = Value::Array(new_arr);
                                if self.local_vars.contains_key(&object) { self.local_vars.insert(object, new_array.clone()); }
                                else { self.global_vars.insert(object, new_array.clone()); }
                                new_array
                            }
                            "clear" => {
                                let empty = Value::Array(Vec::new());
                                if self.local_vars.contains_key(&object) { self.local_vars.insert(object, empty.clone()); }
                                else { self.global_vars.insert(object, empty.clone()); }
                                empty
                            }
                            _ => Value::Null,
                        }
                    }
                    Value::StringVal(ref s) => {
                        match method.as_str() {
                            "len" => Value::Integer(s.len() as i64),
                            "upper" => Value::StringVal(s.to_uppercase()),
                            "lower" => Value::StringVal(s.to_lowercase()),
                            "trim" => Value::StringVal(s.trim().to_string()),
                            "reverse" => Value::StringVal(s.chars().rev().collect()),
                            "contains" => {
                                if args.is_empty() { return Value::Null; }
                                let check = self.eval(args[0].clone());
                                if let Value::StringVal(search) = check { Value::Boolean(s.contains(&search)) } else { Value::Boolean(false) }
                            }
                            "startsWith" => {
                                if args.is_empty() { return Value::Boolean(false); }
                                let prefix = self.eval(args[0].clone());
                                if let Value::StringVal(p) = prefix { Value::Boolean(s.starts_with(&p)) } else { Value::Boolean(false) }
                            }
                            "endsWith" => {
                                if args.is_empty() { return Value::Boolean(false); }
                                let suffix = self.eval(args[0].clone());
                                if let Value::StringVal(suf) = suffix { Value::Boolean(s.ends_with(&suf)) } else { Value::Boolean(false) }
                            }
                            "replace" => {
                                if args.len() < 2 { return Value::Null; }
                                let find = self.eval(args[0].clone());
                                let rep = self.eval(args[1].clone());
                                if let (Value::StringVal(f), Value::StringVal(r)) = (find, rep) { Value::StringVal(s.replace(&f, &r)) } else { Value::Null }
                            }
                            "split" => {
                                if args.is_empty() { return Value::Null; }
                                let sep = self.eval(args[0].clone());
                                if let Value::StringVal(separator) = sep {
                                    let parts: Vec<Value> = s.split(&separator).map(|p| Value::StringVal(p.to_string())).collect();
                                    Value::Array(parts)
                                } else { Value::Null }
                            }
                            "slice" => {
                                if args.len() < 2 { return Value::Null; }
                                let start = match self.eval(args[0].clone()) { Value::Integer(i) => i as usize, _ => 0 };
                                let end = match self.eval(args[1].clone()) { Value::Integer(i) => i as usize, _ => s.len() };
                                if start > end || end > s.len() { return Value::StringVal("".to_string()); }
                                Value::StringVal(s[start..end].to_string())
                            }
                            "charAt" => {
                                if args.is_empty() { return Value::Null; }
                                let idx = match self.eval(args[0].clone()) { Value::Integer(i) => i as usize, _ => return Value::Null };
                                if idx < s.len() { Value::StringVal(s.chars().nth(idx).unwrap().to_string()) } else { Value::Null }
                            }
                            "repeat" => {
                                if args.is_empty() { return Value::StringVal(s.clone()); }
                                let times = match self.eval(args[0].clone()) { Value::Integer(i) => i as usize, _ => 1 };
                                Value::StringVal(s.repeat(times))
                            }
                            "capitalize" => {
                                let words: Vec<String> = s.split(' ')
                                    .map(|w| {
                                        let mut chars = w.chars();
                                        match chars.next() {
                                            Some(first) => {
                                                let upper = first.to_uppercase().to_string();
                                                upper + &chars.collect::<String>()
                                            }
                                            None => String::new(),
                                        }
                                    })
                                    .collect();
                                Value::StringVal(words.join(" "))
                            }
                            "camelCase" => {
                                let words: Vec<&str> = s.split(' ').collect();
                                let mut result = String::new();
                                for (i, word) in words.iter().enumerate() {
                                    if i == 0 {
                                        result.push_str(&word.to_lowercase());
                                    } else {
                                        let mut chars = word.chars();
                                        if let Some(first) = chars.next() {
                                            result.push_str(&first.to_uppercase().to_string());
                                            result.push_str(&chars.collect::<String>().to_lowercase());
                                        }
                                    }
                                }
                                Value::StringVal(result)
                            }
                            "snakeCase" => {
                                let mut result = String::new();
                                for (i, c) in s.chars().enumerate() {
                                    if c.is_uppercase() {
                                        if i > 0 { result.push('_'); }
                                        result.push(c.to_lowercase().next().unwrap());
                                    } else if c == ' ' {
                                        result.push('_');
                                    } else {
                                        result.push(c);
                                    }
                                }
                                Value::StringVal(result)
                            }
                            "padLeft" => {
                                if args.len() < 2 { return Value::Null; }
                                let width = match self.eval(args[0].clone()) {
                                    Value::Integer(n) => n as usize,
                                    _ => return Value::Null,
                                };
                                let pad_char = match self.eval(args[1].clone()) {
                                    Value::StringVal(c) => c.chars().next().unwrap_or(' '),
                                    _ => ' ',
                                };
                                if s.len() >= width {
                                    Value::StringVal(s.clone())
                                } else {
                                    let padding = std::iter::repeat(pad_char).take(width - s.len()).collect::<String>();
                                    Value::StringVal(format!("{}{}", padding, s))
                                }
                            }
                            "padRight" => {
                                if args.len() < 2 { return Value::Null; }
                                let width = match self.eval(args[0].clone()) {
                                    Value::Integer(n) => n as usize,
                                    _ => return Value::Null,
                                };
                                let pad_char = match self.eval(args[1].clone()) {
                                    Value::StringVal(c) => c.chars().next().unwrap_or(' '),
                                    _ => ' ',
                                };
                                if s.len() >= width {
                                    Value::StringVal(s.clone())
                                } else {
                                    let padding = std::iter::repeat(pad_char).take(width - s.len()).collect::<String>();
                                    Value::StringVal(format!("{}{}", s, padding))
                                }
                            }
                            "isNumeric" => {
                                Value::Boolean(s.chars().all(|c| c.is_numeric()))
                            }
                            "isAlpha" => {
                                Value::Boolean(s.chars().all(|c| c.is_alphabetic()))
                            }
                            "isEmail" => {
                                Value::Boolean(s.contains('@') && s.contains('.') && s.len() > 5)
                            }
                            "wordCount" => {
                                Value::Integer(s.split_whitespace().count() as i64)
                            }
                            "truncate" => {
                                if args.is_empty() { return Value::StringVal(s.clone()); }
                                let max_len = match self.eval(args[0].clone()) {
                                    Value::Integer(n) => n as usize,
                                    _ => return Value::StringVal(s.clone()),
                                };
                                if s.len() <= max_len {
                                    Value::StringVal(s.clone())
                                } else {
                                    Value::StringVal(format!("{}...", &s[..max_len]))
                                }
                            }
                            "slug" => {
                                let result = s.to_lowercase()
                                    .chars()
                                    .map(|c| {
                                        if c.is_alphanumeric() { c }
                                        else if c == ' ' { '-' }
                                        else { ' ' }
                                    })
                                    .collect::<String>();
                                let cleaned = result.split_whitespace().collect::<Vec<&str>>().join("");
                                Value::StringVal(cleaned)
                            }
                            "urlEncode" => {
                                let mut result = String::new();
                                for c in s.chars() {
                                    match c {
                                        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                                            result.push(c);
                                        }
                                        ' ' => result.push_str("%20"),
                                        _ => {
                                            for b in c.to_string().bytes() {
                                                result.push_str(&format!("%{:02X}", b));
                                            }
                                        }
                                    }
                                }
                                Value::StringVal(result)
                            }
                            "urlDecode" => {
                                let mut result = String::new();
                                let mut chars = s.chars();
                                while let Some(c) = chars.next() {
                                    if c == '%' {
                                        let hex: String = chars.by_ref().take(2).collect();
                                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                            result.push(byte as char);
                                        }
                                    } else if c == '+' {
                                        result.push(' ');
                                    } else {
                                        result.push(c);
                                    }
                                }
                                Value::StringVal(result)
                            }
                            _ => Value::Null,
                        }
                    }
                    Value::Map(ref map) => {
                        match method.as_str() {
                            "keys" => {
                                let k: Vec<Value> = map
                                .keys()
                                .map(|k| Value::StringVal(k.clone()))
                                .collect();
                            Value::Array(k)
                            }
                            "values" => {
                                let v: Vec<Value> = map
                                .values()
                                .cloned()
                                .collect();
                            Value::Array(v)
                            }
                            "size" => {
                                Value::Integer(map.len() as i64)
                            }
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
                                if self.local_vars.contains_key(&object) {
                                    self.local_vars.insert(object.clone(), result.clone());
                                } else {
                                    self.global_vars.insert(object.clone(), result.clone());
                                }
                                result
                            }
                            _ => {
                                map.get(&method).cloned().unwrap_or(Value::Null)
                            }
                        }
                    }
                    _ => Value::Null,
                }
            }

            Node::BinaryOp { left, op, right } => {
                let l = self.eval(*left);
                let r = self.eval(*right);
                self.apply_op(l, op, r)
            }

            Node::UnaryOp { op, operand } => {
                let val = self.eval(*operand);
                match op.as_str() {
                    "Not" => match val { Value::Boolean(b) => Value::Boolean(!b), _ => Value::Null },
                    "Negate" => match val { Value::Integer(n) => Value::Integer(-n), Value::Float(n) => Value::Float(-n), _ => Value::Null },
                    _ => Value::Null,
                }
            }

            Node::Check { condition, body, or_checks, else_body} => {
                let cond = self.eval(*condition);
                if let Value::Boolean(true) = cond {
                    for node in body {
                        let result = self.eval(node);
                        if let Value::Return(_) = result {
                            return result;
                        }
                        if let Value::Break = result { return result; }
                        if let Value::Continue = result { return result; }
                    }
                    return Value::Null;
                }
                for (or_cond, or_body) in or_checks{
                    let result = self.eval(or_cond);
                    if let Value::Boolean(true) = result {
                        for node in or_body {
                            let result = self.eval(node);
                            if let Value::Return(_) = result {
                                return result;
                            }
                            if let Value::Break = result { return result; }
                            if let Value::Continue = result { return result; }
                        }
                        return Value::Null;
                    }
                }
                if let Some(else_nodes) = else_body {
                    for node in else_nodes {
                        let result = self.eval(node);
                        if let Value::Return(_) = result {
                            return result;
                        }
                        if let Value::Break = result { return result; }
                        if let Value::Continue = result { return result; }
                    }
                }
                Value::Null
            }

            Node::Circle { name, count, body } => {
                let count_val = self.eval(*count);
                let times = match count_val { Value::Integer(n) => n, _ => return Value::Null };
                for i in 0..times {
                    self.local_vars.insert(name.clone(), Value::Integer(i));
                    let mut should_shatter = false;
                    let mut should_skip = false;
                    for node in body.clone() {
                        let result = self.eval(node);
                        match result {
                            Value::Return(_) => return result,
                            Value::Break => { should_shatter = true; break; },
                            Value::Continue => { should_skip = true; break; },
                            _ => {}
                        }
                    }
                    if should_shatter { break;}
                }
                Value::Null
            }

            Node::Shatter => Value::Break,
            Node::Skip => Value::Continue,

            Node::FuncDecl { name, func_type, params, body } => {
                self.functions.insert(name, StoredFunc { func_type, params, body });
                Value::Null
            }

            Node::FuncCall { name, args } => {
                let func = match self.functions.get(&name) {
                    Some(f) => f.clone(),
                    None => { return Value::Error(format!("Function '{}' does not exist.", name)); }
                };

                if func.func_type == "auto" {
                    return Value::Error(format! (
                        "'{}' is an auto function. Use trigger {}[time/when/once]() to activate.",
                        name, name
                    ));
                }

                if func.params.len() != args.len() {
                    return Value::Error(format!("'{}' expects {} args but got {}.", name, func.params.len(), args.len()));
                }

                let mut evaluated_args = Vec::new();
                for arg in args { evaluated_args.push(self.eval(arg)); }

                let save_locals = self.local_vars.clone();
                self.parent_locals.push(save_locals.clone());
                self.local_vars = std::collections::HashMap::new();
                for (param, val) in func.params.iter().zip(evaluated_args.into_iter()) {
                    self.local_vars.insert(param.clone(), val);
                }

                let mut result =Value::Null;
                for node in func.body {
                    result = self.eval(node);
                    if let Value::Return(val) = result {
                        result = *val;
                        break;
                    }
                }
                self.parent_locals.pop();
                self.local_vars = save_locals;
                result
            }

            Node::Rest(duration) => {
                let ms = match self.eval(*duration) { Value::Integer(n) => n as u64, _ => return Value::Null };
                std::thread::sleep(std::time::Duration::from_millis(ms));
                Value::Null
            }

            Node::Wait(duration) => {
                let ms = match self.eval(*duration) { Value::Integer(n) => n as u64, _ => return Value::Null };
                std::thread::sleep(std::time::Duration::from_millis(ms));
                Value::Null
            }

            Node::TriggerCall { name, trigger_type, value } => {
                match trigger_type.as_str() {
                    "time" => {
                        let ms = match self.eval(*value) {
                            Value::Integer(n) => n as u64,
                            _ => {
                                return Value::Error("trigger[time] needs milliseconds".to_string());
                            }
                        };
                        let func = match self.functions.get(&name).cloned() {
                            Some(f) => f,
                            None => {
                                return Value::Error(format!("Function '{}' not found", name));
                            }
                        };
                        let mut forked = self.fork();
                        std::thread::spawn(move || {
                            loop {
                                std::thread::sleep( std::time::Duration::from_millis(ms) );
                                let saved = forked.local_vars.clone();
                                forked.parent_locals.push(saved.clone());
                                forked.local_vars = std::collections::HashMap::new();
                                for node in func.body.clone() {
                                    forked.eval(node);
                                }
                                forked.parent_locals.pop();
                                forked.local_vars = saved;
                            }
                        });
                        Value::Null
                    }
                    "when" => {
                        self.when_triggers.push((
                            name,
                            *value,
                            false,
                        ));
                        Value::Null
                    }
                    "once" => {
                        self.when_triggers.push((
                            name,
                            *value,
                            true,
                        ));
                        Value::Null
                    }
                    _ => {
                        Value::Error(format!("Unknown trigger type: '{}'", trigger_type))
                    }
                }
            }

            Node::Guard { condition, body } => {
                let cond = self.eval(*condition);
                if let Value::Boolean(true) = cond {
                    for statement in body {
                        match statement {
                            Node::Shatter => return Value::Boolean(true),
                            _ => { self.eval(statement); }
                        }
                    }
                }
                Value::Null
            }

                        Node::MultiVarL { names, values } => {
                for (i, name) in names.iter().enumerate() {
                    let val = self.eval(*values[i].clone());
                    self.local_vars.insert(name.clone(), val);
                }
                Value::Null
            }

            Node::MapLit { keys, values } => {
                let mut map = std::collections::HashMap::new();
                for (i, key) in keys.iter().enumerate() {
                    let val = self.eval(*values[i].clone());
                    map.insert(key.clone(), val);
                }
                Value::Map(map)
            }

            Node::Attempt { body, rescue_param, rescue_body, always_body} => {
                let mut error_occurred = false;
                let mut error_message = String::new();
                for node in body {
                    let result = self.eval(node);
                    match result {
                        Value::Error(ref msg) => {
                            error_occurred = true;
                            error_message = msg.clone();
                            break;
                        }
                        Value::Return(_) => return result,
                        Value::Break => return result,
                        Value::Continue => return result,
                        _ => {}
                    }
                }
                if error_occurred {
                    if let Some(rbody) = rescue_body {
                        if let Some(param_name) = rescue_param {
                            self.local_vars.insert(
                                param_name,
                                Value::StringVal(error_message.clone())
                            );
                        }
                        for node in rbody {
                            self.eval(node);
                        }
                    }
                }
                if let Some(abody) = always_body {
                    for node in abody {
                        self.eval(node);
                    }
                }
                Value::Null
            }

            Node::Reply(expr) => {
                let val = self.eval(*expr);
                Value::Return(Box::new(val))
            }
        }
    }

    /// Convert the first argument to a float for numeric math functions.
    fn eval_to_float(&mut self, args: &Vec<Node>) -> f64 {
        if args.is_empty() { return 0.0; }
        match self.eval(args[0].clone()) {
            Value::Integer(n) => n as f64,
            Value::Float(f) => f,
            _ => 0.0,
        }
    }

    /// Apply a binary operator to two runtime values.
    fn apply_op(&self, left: Value, op: String, right: Value) -> Value {
        if op == "And" {
            return match (&left, &right) {
                (Value::Boolean(l), Value::Boolean(r)) => Value::Boolean(*l && *r),
                _ => Value::Null,
            };
        }
        if op == "Or" {
            return match (&left, &right) {
                (Value::Boolean(l), Value::Boolean(r)) => Value::Boolean(*l || *r),
                _ => Value::Null,
            };
        }
        match (&left, &right) {
            (Value::Integer(l), Value::Integer(r)) => match op.as_str() {
                "Plus" => Value::Integer(l + r),
                "Minus" => Value::Integer(l - r),
                "Star" => Value::Integer(l * r),
                "Slash" => if *r == 0 {
                    Value::Error("Division by zero. You divided by 0! Math ain't mathing bro.".to_string())
                } else { Value::Integer(l / r) },
                "Percent" => Value::Integer(l % r),
                "DoubleStar" => if *r < 0 { Value::Null } else { Value::Integer(l.pow(*r as u32)) },
                "EqualEqual" => Value::Boolean(l == r),
                "NotEqual" => Value::Boolean(l != r),
                "Greater" => Value::Boolean(l > r),
                "Less" => Value::Boolean(l < r),
                "GreaterEqual" => Value::Boolean(l >= r),
                "LessEqual" => Value::Boolean(l <= r),
                _ => Value::Null,
            },
            (Value::Float(l), Value::Float(r)) => match op.as_str() {
                "Plus" => Value::Float(l + r),
                "Minus" => Value::Float(l - r),
                "Star" => Value::Float(l * r),
                "Slash" => Value::Float(l / r),
                "Percent" => Value::Float(l % r),
                "DoubleStar" => Value::Float(l.powf(*r)),
                "EqualEqual" => Value::Boolean(l == r),
                "NotEqual" => Value::Boolean(l != r),
                "Greater" => Value::Boolean(l > r),
                "Less" => Value::Boolean(l < r),
                "GreaterEqual" => Value::Boolean(l >= r),
                "LessEqual" => Value::Boolean(l <= r),
                _ => Value::Null,
            },
            (Value::Integer(l), Value::Float(r)) => {
                let l = *l as f64;
                match op.as_str() {
                    "Plus" => Value::Float(l + r),
                    "Minus" => Value::Float(l - r),
                    "Star" => Value::Float(l * r),
                    "Slash" => Value::Float(l / r),
                    _ => Value::Null,
                }
            }
            (Value::Float(l), Value::Integer(r)) => {
                let r = *r as f64;
                match op.as_str() {
                    "Plus" => Value::Float(l + r),
                    "Minus" => Value::Float(l - r),
                    "Star" => Value::Float(l * r),
                    "Slash" => Value::Float(l / r),
                    _ => Value::Null,
                }
            }
            (Value::StringVal(l), Value::StringVal(r)) => match op.as_str() {
                "Plus" => Value::StringVal(format!("{}{}", l, r)),
                "EqualEqual" => Value::Boolean(l == r),
                "NotEqual" => Value::Boolean(l != r),
                _ => Value::Null,
            },
            (Value::StringVal(l), other) => {
                if op == "Plus" { Value::StringVal(format!("{}{}", l, other)) } else { Value::Null }
            }
            (other, Value::StringVal(r)) => {
                if op == "Plus" { Value::StringVal(format!("{}{}", other, r)) } else { Value::Null }
            }
            (Value::Boolean(l), Value::Boolean(r)) => match op.as_str() {
                "EqualEqual" => Value::Boolean(l == r),
                "NotEqual" => Value::Boolean(l != r),
                _ => Value::Null,
            },
            _ => Value::Null,
        }
    }
}