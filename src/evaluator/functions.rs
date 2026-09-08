use super::Evaluator;
use super::value::{StoredFunc, Value};
use crate::parser::Node;

impl Evaluator {
    pub fn eval_func_decl(&mut self, name: String, func_type: String, params: Vec<String>, body: Vec<Node>) -> Value {
        self.functions
            .write()
            .unwrap()
            .insert(name, StoredFunc { func_type, params, body });
        Value::Null
    }

    pub fn eval_func_call(&mut self, name: String, args: Vec<Node>) -> Value {
        let func = {
            let map = self.functions.read().unwrap();
            match map.get(&name) {
                Some(f) => f.clone(),
                None => return Value::Error(format!("Function '{name}' does not exist.")),
            }
        };

        if func.params.len() != args.len() {
            return Value::Error("Argument count mismatch.".into());
        }

        let mut vals = Vec::new();
        for a in args {
            vals.push(self.eval(a));
        }

        let old_locals = self.locals.clone();
        self.locals = crate::evaluator::environment::Environment::child(old_locals.clone());

        {
            let mut env = self.locals.write().unwrap();
            for (p, v) in func.params.iter().zip(vals.into_iter()) {
                env.define(p.clone(), v);
            }
        }

        let mut result = Value::Null;
        for n in func.body {
            result = self.eval(n);
            if let Value::Return(v) = result {
                result = *v;
                break;
            }
        }

        self.locals = old_locals;
        result
    }

    pub fn eval_trigger_call(&mut self, name: String, trigger_type: String, value: Node) -> Value {
        match trigger_type.as_str() {
            "time" => {
                let ms = match self.eval(value) {
                    Value::Integer(n) => n as u64,
                    _ => return Value::Error("trigger[time] needs milliseconds integer".to_string()),
                };
                let func = match self.functions.read().unwrap().get(&name).cloned() {
                    Some(f) => f,
                    None => return Value::Error(format!("Function '{}' not found", name)),
                };
                let func_body = func.body.clone();
                let mut forked = self.fork();
                std::thread::spawn(move || {
                    loop {
                        std::thread::sleep(std::time::Duration::from_millis(ms));
                        let old_locals = forked.locals.clone();
                        forked.locals = crate::evaluator::environment::Environment::child(old_locals.clone());
                        for node in func_body.clone() {
                            forked.eval(node);
                        }
                        forked.locals = old_locals;
                    }
                });
                Value::Null
            }
            "when" => {
                self.when_triggers.write().unwrap().push((name, value, false));
                Value::Null
            }
            "once" => {
                self.when_triggers.write().unwrap().push((name, value, true));
                Value::Null
            }
            _ => Value::Error(format!("Unknown trigger type: '{}'", trigger_type)),
        }
    }

    pub fn check_triggers(&mut self) {
        let triggers = {
            let guard = self.when_triggers.read().unwrap();
            guard.clone()
        };

        let mut to_fire: Vec<(String, bool)> = Vec::new();
        for (name, condition, is_once) in triggers {
            let result = self.eval(condition);
            if let Value::Boolean(true) = result {
                to_fire.push((name, is_once));
            }
        }

        let fired_once: Vec<String> = to_fire
            .iter()
            .filter_map(|(name, is_once)| if *is_once { Some(name.clone()) } else { None })
            .collect();

        {
            let mut guard = self.when_triggers.write().unwrap();
            guard.retain(|(name, _, _)| !fired_once.contains(name));
        }

        for (name, _) in to_fire {
            let func = {
                let guard = self.functions.read().unwrap();
                guard.get(&name).cloned()
            };

            if let Some(func) = func {
                let old_locals = self.locals.clone();
                self.locals = crate::evaluator::environment::Environment::child(old_locals.clone());
                for node in func.body {
                    self.eval(node);
                }
                self.locals = old_locals;
            }
        }
    }

    pub fn eval_rest(&mut self, duration: Node) -> Value {
        let ms = match self.eval(duration) {
            Value::Integer(n) => n as u64,
            _ => return Value::Null,
        };
        std::thread::sleep(std::time::Duration::from_millis(ms));
        Value::Null
    }

    pub fn eval_wait(&mut self, duration: Node) -> Value {
        let ms = match self.eval(duration) {
            Value::Integer(n) => n as u64,
            _ => return Value::Null,
        };
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