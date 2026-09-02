use super::Evaluator;
use super::value::Value;
use crate::parser::Node;

impl Evaluator {
    pub fn eval_check(
        &mut self,
        condition: Node,
        body: Vec<Node>,
        or_checks: Vec<(Node, Vec<Node>)>,
        else_body: Option<Vec<Node>>,
    ) -> Value {
        let cond = self.eval(condition);
        if let Value::Boolean(true) = cond {
            for node in body {
                let result = self.eval(node);
                if matches!(result, Value::Return(_) | Value::Break | Value::Continue) {
                    return result;
                }
            }
            return Value::Null;
        }
        for (or_cond, or_body) in or_checks {
            let result = self.eval(or_cond);
            if let Value::Boolean(true) = result {
                for node in or_body {
                    let result = self.eval(node);
                    if matches!(result, Value::Return(_) | Value::Break | Value::Continue) {
                        return result;
                    }
                }
                return Value::Null;
            }
        }
        if let Some(else_nodes) = else_body {
            for node in else_nodes {
                let result = self.eval(node);
                if matches!(result, Value::Return(_) | Value::Break | Value::Continue) {
                    return result;
                }
            }
        }
        Value::Null
    }

    pub fn eval_circle(&mut self, name: String, count: Node, body: Vec<Node>) -> Value {
        let count_val = self.eval(count);
        let times = match count_val { Value::Integer(n) => n, _ => return Value::Null };
        for i in 0..times {
            self.local_vars.insert(name.clone(), Value::Integer(i));
            let mut break_loop = false;
            for node in body.clone() {
                let result = self.eval(node);
                match result {
                    Value::Return(_) => return result,
                    Value::Break => { break_loop = true; break; }
                    Value::Continue => { break; }
                    _ => {}
                }
            }
            if break_loop { break; }
        }
        Value::Null
    }

    pub fn eval_guard(&mut self, condition: Node, body: Vec<Node>) -> Value {
        let cond = self.eval(condition);
        if let Value::Boolean(true) = cond {
            for statement in body {
                let result = self.eval(statement);
                if let Value::Break = result {
                    return Value::Boolean(true); // guard shatter returns true
                }
            }
        }
        Value::Null
    }

    pub fn eval_attempt(
        &mut self,
        body: Vec<Node>,
        rescue_param: Option<String>,
        rescue_body: Option<Vec<Node>>,
        always_body: Option<Vec<Node>>,
    ) -> Value {
        let mut error_occurred = false;
        let mut error_message = String::new();
        for node in body {
            let result = self.eval(node);
            if let Value::Error(msg) = result {
                error_occurred = true;
                error_message = msg;
                break;
            } else if matches!(result, Value::Return(_) | Value::Break | Value::Continue) {
                return result;
            }
        }
        if error_occurred {
            if let Some(rbody) = rescue_body {
                if let Some(param_name) = rescue_param {
                    self.local_vars.insert(param_name, Value::StringVal(error_message));
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

    pub fn eval_protect(&mut self, vars: Vec<String>, body: Vec<Node>) -> Value {
        let to_protect: Vec<String> = if vars.is_empty() {
            self.local_vars.keys().cloned()
                .chain(self.global_vars.keys().cloned())
                .chain(self.constants.keys().cloned())
                .collect()
        } else {
            vars
        };
        let mut newly_protected = Vec::new();
        for v in &to_protect {
            if !self.protected_vars.contains(v) {
                self.protected_vars.insert(v.clone());
                newly_protected.push(v.clone());
            }
        }
        let mut result = Value::Null;
        for statement in body {
            result = self.eval(statement);
            if matches!(result, Value::Return(_) | Value::Break | Value::Continue | Value::Error(_)) {
                break;
            }
        }
        for v in &newly_protected {
            self.protected_vars.remove(v);
        }
        result
    }
}