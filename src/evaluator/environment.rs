//! Lexically nested variable scopes used by the evaluator.
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
            // Cloning preserves the value's shared-handle semantics for arrays and
            // maps while keeping scalar values independent of the environment.
            return Some(v.clone());
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