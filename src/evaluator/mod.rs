//! Runtime state and AST evaluation for the Connect language.
//!
//! The evaluator keeps global state separate from the current local scope, while
//! reference-counted locks allow arrays, maps, and shared runtime registries to
//! survive function calls and evaluator forks.

pub mod value;
pub mod environment;
pub mod operators;
pub mod control;
pub mod functions;
pub mod builtins;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::parser::Node;
use environment::Environment;
pub use value::{StoredFunc, Value};
use operators::apply_op;

#[derive(Debug, Clone)]
pub enum UndoRecord {
    VarUpdate {
        name: String,
        old_value: Option<Value>,
        is_global: bool,
    },
    IndexUpdate {
        name: String,
        index: usize,
        old_value: Value,
    },
}

pub struct Evaluator {
    pub globals: Arc<RwLock<Environment>>,
    pub locals: Arc<RwLock<Environment>>,
    pub constants: Arc<RwLock<HashMap<String, Value>>>,
    pub functions: Arc<RwLock<HashMap<String, StoredFunc>>>,
    pub protected: Arc<RwLock<HashSet<String>>>,
    pub when_triggers: Arc<RwLock<Vec<(String, Node, bool)>>>,
    pub history: Arc<RwLock<Vec<UndoRecord>>>,
}

impl Evaluator {
    pub fn new() -> Self {
        let globals = Environment::new();
        let locals = Environment::child(globals.clone());
        Self {
            globals,
            locals,
            constants: Arc::new(RwLock::new(HashMap::new())),
            functions: Arc::new(RwLock::new(HashMap::new())),
            protected: Arc::new(RwLock::new(HashSet::new())),
            when_triggers: Arc::new(RwLock::new(Vec::new())),
            // Each evaluator starts with an empty transaction log. Forks share
            // this log so mutations made by asynchronous work remain undoable.
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn fork(&self) -> Self {
    let locals = Environment::child(self.locals.clone());
    Self {
        globals: self.globals.clone(),
        locals,
        constants: self.constants.clone(),
        functions: self.functions.clone(),
        protected: self.protected.clone(),
        when_triggers: self.when_triggers.clone(),
        history: self.history.clone(),
    }
}

    pub fn run(&mut self, nodes: Vec<Node>) {
        for n in nodes {
            let v = self.eval(n);
            if let Value::Error(e) = v {
                println!("[RUNTIME ERROR] {e}");
            }
        }
    }

    pub fn lookup(&self, name: &str) -> Value {
        if let Ok(env) = self.locals.read() {
            if let Some(v) = env.get(name) {
                return v;
            }
        }
        if let Ok(c) = self.constants.read() {
            if let Some(v) = c.get(name) {
                return v.clone();
            }
        }
        Value::Error(format!("'{name}' was never declared."))
    }

    /// Restores up to `count` successful mutations in reverse chronological order.
    ///
    /// A missing previous value means the mutation created the variable, so rewind
    /// removes it. Array entries store their previous value directly because the
    /// array itself is a shared handle rather than a copied local binding.
    pub fn eval_rewind(&mut self, count_node: Node) -> Value {
        let count = match self.eval(count_node) {
            Value::Integer(n) if n > 0 => n as usize,
            _ => return Value::Error("rewind() expects a positive integer. Zero or negative rewind values are not history; they're just denial.".into()),
        };

        let mut hist = self.history.write().unwrap();
        let mut rewound_steps = 0;

        for _ in 0..count {
            if let Some(record) = hist.pop() {
                match record {
                    UndoRecord::VarUpdate { name, old_value, is_global } => {
                        if is_global {
                            if let Ok(mut env) = self.globals.write() {
                                if let Some(old) = old_value {
                                    env.define(name, old);
                                } else {
                                    env.vars.remove(&name);
                                }
                            }
                        } else {
                            if let Ok(mut env) = self.locals.write() {
                                if let Some(old) = old_value {
                                    env.assign(&name, old);
                                } else {
                                    env.vars.remove(&name);
                                }
                            }
                        }
                    }
                    UndoRecord::IndexUpdate { name, index, old_value } => {
                        if let Value::Array(arr) = self.lookup(&name) {
                            if let Ok(mut a) = arr.write() {
                                if index < a.len() {
                                    a[index] = old_value;
                                }
                            }
                        }
                    }
                }
                rewound_steps += 1;
            } else {
                break;
            }
        }

        Value::Integer(rewound_steps as i64)
    }

    fn eval_update(&mut self, name: String, value: Node) -> Value {
        if let Ok(p) = self.protected.read() {
            if p.contains(&name) {
                return Value::Error(format!("Variable '{name}' is protected. That's not a mutation; that's a locked door with a bad attitude."));
            }
        }
        if let Ok(c) = self.constants.read() {
            if c.contains_key(&name) {
                return Value::Error(format!("'{name}' is a constant. Cannot mutate. Constants don't change; they just sit there like a smug statue."));
            }
        }

        // Capture the value before evaluation changes the binding. `None` marks a
        // newly created binding and lets rewind remove it instead of restoring it.
        let old_val = match self.lookup(&name) {
            Value::Error(_) => None,
            val => Some(val),
        };

        let v = self.eval(value);

        let is_global = if let Ok(env) = self.locals.read() {
            !env.vars.contains_key(&name)
        } else {
            false
        };

        let ok = if let Ok(mut env) = self.locals.write() {
            env.assign(&name, v.clone())
        } else {
            false
        };

        if ok {
            // Only successful assignments enter the log; failed evaluations must
            // not make a later rewind undo an operation that never happened.
            if let Ok(mut h) = self.history.write() {
                h.push(UndoRecord::VarUpdate {
                    name,
                    old_value: old_val,
                    is_global,
                });
            }
            self.check_triggers();
            v
        } else {
            Value::Error(format!("'{name}' was never declared. That's not a variable; that's an imaginary friend with no references."))
        }
    }

    pub fn eval_judge(
    &mut self,
    expr: Node,
    cases: Vec<(Node, Node)>,
    default_case: Option<Box<Node>>,
) -> Value {
    let target = self.eval(expr);

    for (pattern_node, body_node) in cases {
        let pattern_val = self.eval(pattern_node);

        if target == pattern_val {
            return self.eval(body_node);
        }
    }

    if let Some(default_node) = default_case {
        return self.eval(*default_node);
    }

    Value::Null
}

    fn eval_update_index(&mut self, name: String, index: Node, value: Node) -> Value {
    let idx_val = self.eval(index);
    let new_v = self.eval(value);

    match self.lookup(&name) {
        // 1. Array Index Update: arr[0] = 99
        Value::Array(arr) => {
            let idx = match idx_val {
                Value::Integer(i) if i >= 0 => i as usize,
                _ => return Value::Error("Array index must be a non-negative integer.".into()),
            };
            if let Ok(mut a) = arr.write() {
                if idx >= a.len() {
                    return Value::Error("Array index out of bounds.".into());
                }
                let old_val = a[idx].clone();
                a[idx] = new_v.clone();

                if let Ok(mut h) = self.history.write() {
                    h.push(UndoRecord::IndexUpdate { name, index: idx, old_value: old_val });
                }
                new_v
            } else {
                Value::Error("Array handle locked.".into())
            }
        }

        // 2. Map Property Update: user.role = 'Master Architect'
        Value::Map(map) => {
            let key = match idx_val {
                Value::StringVal(s) => s,
                other => other.to_string(),
            };
            if let Ok(mut m) = map.write() {
                m.insert(key, new_v.clone());
                new_v
            } else {
                Value::Error("Map handle locked.".into())
            }
        }

        Value::Error(e) => Value::Error(e),
        _ => Value::Error(format!("'{name}' is not an array or map.")),
    }
}

    pub fn eval(&mut self, node: Node) -> Value {
        match node {
            Node::Rewind(count) => self.eval_rewind(*count),
            Node::Integer(n) => Value::Integer(n),
            Node::Float(f) => Value::Float(f),
            // Inside eval() in src/evaluator/mod.rs:

// Inside eval() match arm for Node::StringLit(s) in src/evaluator/mod.rs:

// Inside eval() match arm for Node::StringLit(s) in src/evaluator/mod.rs:

Node::StringLit(s) => {
    if s.contains('$') {
        let mut result = String::new();
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '$' && i + 1 < chars.len() && (chars[i + 1].is_alphabetic() || chars[i + 1] == '_') {
                let start_idx = i;
                i += 1; // skip '$'
                
                let mut var_name = String::new();
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    var_name.push(chars[i]);
                    i += 1;
                }

                let val = self.lookup(&var_name);
                if !matches!(val, Value::Error(_)) {
                    result.push_str(&format!("{}", val));
                } else {
                    // Variable doesn't exist: leave literal '$var_name' as text!
                    result.push_str(&s[start_idx..i]);
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        Value::StringVal(result)
    } else {
        Value::StringVal(s)
    }
}
            Node::Boolean(b) => Value::Boolean(b),
            Node::Null => Value::Null,
            Node::Array(items) => {
                let mut out = Vec::new();
                for i in items { out.push(self.eval(i)); }
                Value::array(out)
            }
            Node::MapLit { keys, values } => self.eval_map_lit(keys, values),
            Node::VarLDecl { name, value } => {
                let v = self.eval(*value);
                if let Ok(mut env) = self.locals.write() {
                    env.define(name, v.clone());
                }
                v
            }
            Node::VarGDecl { name, value } => {
                let v = self.eval(*value);
                if let Ok(mut env) = self.globals.write() {
                    env.define(name, v.clone());
                }
                self.check_triggers();
                v
            }
            Node::ConstDecl { name, value } => {
                if self.constants.read().unwrap().contains_key(&name) {
                    return Value::Error(format!("Constant '{name}' already exists."));
                }
                let v = self.eval(*value);
                let mut c = self.constants.write().unwrap();
                if c.contains_key(&name) {
                    return Value::Error(format!("Constant '{name}' already exists. This constant is more popular than a surprise outage."));
                }
                c.insert(name, v.clone());
                v
            }
            Node::VarAccess(name) => self.lookup(&name),
            Node::Summon(name) => {
                let v = self.lookup(&name);
                if matches!(v, Value::Error(_)) {
                    Value::Error(format!("'{}' could not be summoned. That variable is hiding from the code like a coward in a thunderstorm.", name))
                } else {
                    v
                }
            }
            Node::MultiVarL { names, values } => {
                for (i, name) in names.into_iter().enumerate() {
                    let v = self.eval(*values[i].clone());
                    if let Ok(mut env) = self.locals.write() {
                        env.define(name, v);
                    }
                }
                Value::Null
            }
            Node::UpdateDecl { name, value } => self.eval_update(name, *value),
            Node::UpdateIndex { name, index, value } => self.eval_update_index(name, *index, *value),
            Node::BinaryOp { left, op, right } => {
                let l = self.eval(*left);
                let r = self.eval(*right);
                apply_op(l, &op, r)
            }
            Node::UnaryOp { op, operand } => {
                let val = self.eval(*operand);
                match op.as_str() {
                    "Not" => Value::Boolean(!val.is_truthy()),
                    "Negate" => match val {
                        Value::Integer(n) => Value::Integer(-n),
                        Value::Float(f) => Value::Float(-f),
                        _ => Value::Null,
                    },
                    _ => Value::Null,
                }
            }
            Node::Check { condition, body, or_checks, else_body } => self.eval_check(*condition, body, or_checks, else_body),
            Node::Circle { name, count, body } => self.eval_circle(name, *count, body),
            Node::Shatter => Value::Break,
            Node::Skip => Value::Continue,
            Node::Block(nodes) => {
            let mut last = Value::Null;
            for n in nodes {
                last = self.eval(n);
                if matches!(last, Value::Return(_) | Value::Break | Value::Continue) {
                    return last;
                }
            }
            last
        },
            Node::Guard { condition, body } => self.eval_guard(*condition, body),
            Node::Attempt { body, rescue_param, rescue_body, always_body } => self.eval_attempt(body, rescue_param, rescue_body, always_body),
            Node::Protect { vars, body } => self.eval_protect(vars, body),
            Node::FuncDecl { name, func_type, params, body } => self.eval_func_decl(name, func_type, params, body),
            Node::FuncCall { name, args } => self.eval_func_call(name, args),
            Node::Reply(expr) => Value::Return(Box::new(self.eval(*expr))),
            Node::AsyncBlock { body } => self.eval_async_block(body),
            Node::TriggerCall { name, trigger_type, value } => self.eval_trigger_call(name, trigger_type, *value),
            Node::Rest(duration) | Node::Wait(duration) => {
                let ms = match self.eval(*duration) {
                    Value::Integer(n) if n >= 0 => n as u64,
                    _ => 0,
                };
                std::thread::sleep(std::time::Duration::from_millis(ms));
                Value::Null
            }
            Node::Print(expr) => self.eval_print(*expr),
            Node::InputExpr(prompt) => self.eval_input(*prompt),
            Node::TypeOf(expr) => self.eval_typeof(*expr),
            Node::ToInt(expr) => self.eval_to_int(*expr),
            Node::ToFloat(expr) => self.eval_to_float_expr(*expr),
            Node::ToString(expr) => self.eval_to_string(*expr),
            Node::ToBool(expr) => self.eval_to_bool(*expr),
            Node::IndexAccess { name, index } => self.eval_index_access(name, *index),
            Node::MethodCall { object, method, args } => self.eval_method_call(object, method, args),
            Node::MathCall { method, args } => self.eval_math_call(method, args),
            Node::FileCall { method, args } => self.eval_file_call(method, args),
            Node::JsonCall { method, args } => self.eval_json_call(method, args),
            Node::DateCall { method, args } => self.eval_date_call(method, args),
            Node::SystemCall { method, args } => self.eval_system_call(method, args),
            Node::HttpCall { method, args } => self.eval_http_call(method, args),
            Node::CryptoCall { method, args } => self.eval_crypto_call(method, args),
            Node::UseModule(path) => self.eval_use_module(path),
            Node::DbCall { method, args } => self.eval_db_builtin(&method, args),
            Node::Judge { expr, cases, default_case } => self.eval_judge(*expr, cases, default_case),
            Node::PaintCall { method, args } => self.eval_paint_builtin(&method, args),
            Node::Bond { target, alias } => self.eval_bond(target, alias),
            Node::VbpCall { method, args } => self.eval_vbp_builtin(&method, args),
            Node::VaultCall { method, args } => self.eval_vault_builtin(&method, args),
            Node::Weave { count, body } => self.eval_weave(*count, body),
            Node::MeshCall { mode, method, args } => {
                self.eval_mesh_builtin(&mode, &method, args)
            }
        }
    }
}