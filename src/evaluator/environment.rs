// src/evaluator/environment.rs
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::value::Value;

#[derive(Debug)]
pub struct Environment {
    pub vars: HashMap<String, Value>,
    pub parent: Option<Arc<RwLock<Environment>>>,
}

impl Environment {
    pub fn new() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            vars: HashMap::new(),
            parent: None,
        }))
    }

    pub fn child(parent: Arc<RwLock<Environment>>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            vars: HashMap::new(),
            parent: Some(parent),
        }))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.vars.get(name) {
            return Some(v.clone()); // clone is cheap for handles
        }
        if let Some(parent) = &self.parent {
            if let Ok(p) = parent.read() {
                return p.get(name);
            }
        }
        None
    }

    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        if self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), value);
            return true;
        }
        if let Some(parent) = &self.parent {
            if let Ok(mut p) = parent.write() {
                return p.assign(name, value);
            }
        }
        false
    }

    pub fn assign_local(&mut self, name: &str, value: Value) -> bool {
        if self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), value);
            true
        } else {
            false
        }
    }
}