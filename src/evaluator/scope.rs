use super::Evaluator;
use super::value::Value;
use crate::parser::Node;

impl Evaluator {
    pub fn eval_var_l(&mut self, name: String, value: Node) -> Value {
        let val = self.eval(value);
        self.local_vars.insert(name, val.clone());
        val
    }

    pub fn eval_var_g(&mut self, name: String, value: Node) -> Value {
        let val = self.eval(value);
        self.global_vars.insert(name, val.clone());
        self.check_triggers();
        val
    }

    pub fn eval_const(&mut self, name: String, value: Node) -> Value {
        if self.constants.contains_key(&name) {
            return Value::Error(format!("Constant '{}' already exists.", name));
        }
        let val = self.eval(value);
        self.constants.insert(name, val.clone());
        val
    }

    pub fn eval_update(&mut self, name: String, value: Node) -> Value {
        if self.protected_vars.contains(&name) {
            return Value::Error(format!("Variable '{}' is protected and cannot be mutated.", name));
        }
        if self.constants.contains_key(&name) {
            return Value::Error(format!("'{}' is a constant. Cannot change constant state.", name));
        }
        let val = self.eval(value);
        if self.local_vars.contains_key(&name) {
            self.local_vars.insert(name, val.clone());
        } else if self.global_vars.contains_key(&name) {
            self.global_vars.insert(name, val.clone());
            self.check_triggers();
        } else {
            return Value::Error(format!("'{}' was never declared.", name));
        }
        val
    }

    pub fn eval_update_index(&mut self, name: String, index: Node, value: Node) -> Value {
        if self.protected_vars.contains(&name) {
            return Value::Error(format!("Variable '{}' is protected and cannot be mutated.", name));
        }
        if self.constants.contains_key(&name) {
            return Value::Error(format!("'{}' is a constant. Cannot mutate indexes of constants.", name));
        }
        let idx = match self.eval(index) {
            Value::Integer(i) => i as usize,
            _ => return Value::Error("Array index must be an integer.".to_string()),
        };
        let val = self.eval(value);
        if let Some(arr) = self.local_vars.get_mut(&name) {
            if let Value::Array(elements) = arr {
                if idx < elements.len() { elements[idx] = val.clone(); }
                else { return Value::Error("Index out of bounds.".to_string()); }
            }
        } else if let Some(arr) = self.global_vars.get_mut(&name) {
            if let Value::Array(elements) = arr {
                if idx < elements.len() { elements[idx] = val.clone(); }
                else { return Value::Error("Index out of bounds.".to_string()); }
            }
        } else {
            return Value::Error(format!("'{}' does not exist.", name));
        }
        val
    }

    pub fn eval_var_access(&self, name: &str) -> Value {
        if let Some(val) = self.local_vars.get(name) { return val.clone(); }
        if let Some(val) = self.global_vars.get(name) { return val.clone(); }
        if let Some(val) = self.constants.get(name) { return val.clone(); }
        Value::Error(format!("'{}' was never declared.", name))
    }

    pub fn eval_summon(&mut self, name: String) -> Value {
        if let Some(val) = self.local_vars.get(&name) { return val.clone(); }
        for parent in self.parent_locals.iter().rev() {
            if let Some(val) = parent.get(&name) {
                self.local_vars.insert(name.clone(), val.clone());
                return val.clone();
            }
        }
        if let Some(val) = self.global_vars.get(&name) { return val.clone(); }
        if let Some(val) = self.constants.get(&name) { return val.clone(); }
        Value::Error(format!("'{}' could not be summoned.", name))
    }

    pub fn eval_multi_var_l(&mut self, names: Vec<String>, values: Vec<Box<Node>>) -> Value {
        for (i, name) in names.iter().enumerate() {
            let val = self.eval(*values[i].clone());
            self.local_vars.insert(name.clone(), val);
        }
        Value::Null
    }
}