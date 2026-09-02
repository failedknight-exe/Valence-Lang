use super::Evaluator;
use super::value::{Value, StoredFunc};
use crate::parser::Node;

impl Evaluator {
    pub fn eval_func_decl(&mut self, name: String, func_type: String, params: Vec<String>, body: Vec<Node>) -> Value {
        self.functions.insert(name, StoredFunc { func_type, params, body });
        Value::Null
    }

    pub fn eval_func_call(&mut self, name: String, args: Vec<Node>) -> Value {
        let func = match self.functions.get(&name) {
            Some(f) => f.clone(),
            None => return Value::Error(format!("Function '{}' does not exist.", name)),
        };

        if func.func_type == "auto" {
            return Value::Error(format!("'{}' is an auto function. Call it via triggers.", name));
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

        let mut result = Value::Null;
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

    pub fn eval_trigger_call(&mut self, name: String, trigger_type: String, value: Node) -> Value {
        match trigger_type.as_str() {
            "time" => {
                let ms = match self.eval(value) {
                    Value::Integer(n) => n as u64,
                    _ => return Value::Error("trigger[time] needs milliseconds integer".to_string()),
                };
                let func = match self.functions.get(&name).cloned() {
                    Some(f) => f,
                    None => return Value::Error(format!("Function '{}' not found", name)),
                };
                let mut forked = self.fork();
                std::thread::spawn(move || {
                    loop {
                        std::thread::sleep(std::time::Duration::from_millis(ms));
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
                self.when_triggers.push((name, value, false));
                Value::Null
            }
            "once" => {
                self.when_triggers.push((name, value, true));
                Value::Null
            }
            _ => Value::Error(format!("Unknown trigger type: '{}'", trigger_type)),
        }
    }

    pub fn check_triggers(&mut self) {
        let triggers = self.when_triggers.clone();
        let mut to_fire = Vec::new();
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
        self.when_triggers.retain(|(name, _, _)| !fired_once.contains(name));
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

    pub fn eval_rest(&mut self, duration: Node) -> Value {
        let ms = match self.eval(duration) { Value::Integer(n) => n as u64, _ => return Value::Null };
        std::thread::sleep(std::time::Duration::from_millis(ms));
        Value::Null
    }

    pub fn eval_wait(&mut self, duration: Node) -> Value {
        let ms = match self.eval(duration) { Value::Integer(n) => n as u64, _ => return Value::Null };
        std::thread::sleep(std::time::Duration::from_millis(ms));
        Value::Null
    }

    pub fn eval_async_block(&mut self, body: Vec<Node>) -> Value {
        let mut forked = self.fork();
        std::thread::spawn(move || {
            for node in body {
                forked.eval(node);
            }
        });
        Value::Null
    }
}