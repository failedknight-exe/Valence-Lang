pub mod value;
pub mod operators;
pub mod scope;
pub mod control;
pub mod functions;
pub mod builtins;

use crate::parser::Node;
pub use value::{Value, StoredFunc};
use operators::apply_op;

use std::collections::{HashMap, HashSet};

pub struct Evaluator {
    pub local_vars: HashMap<String, Value>,
    pub global_vars: HashMap<String, Value>,
    pub constants: HashMap<String, Value>,
    pub functions: HashMap<String, StoredFunc>,
    pub parent_locals: Vec<HashMap<String, Value>>,
    pub when_triggers: Vec<(String, Node, bool)>,
    pub protected_vars: HashSet<String>,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            local_vars: HashMap::new(),
            global_vars: HashMap::new(),
            constants: HashMap::new(),
            functions: HashMap::new(),
            parent_locals: Vec::new(),
            when_triggers: Vec::new(),
            protected_vars: HashSet::new(),
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
            protected_vars: self.protected_vars.clone(),
        }
    }

    pub fn run(&mut self, nodes: Vec<Node>) {
        for node in nodes {
            let result = self.eval(node);
            if let Value::Error(msg) = result {
                println!("[RUNTIME ERROR] {}", msg);
            }
        }
    }

    pub fn eval(&mut self, node: Node) -> Value {
        match node {
            Node::Integer(n) => Value::Integer(n),
            Node::Float(f) => Value::Float(f),
            Node::StringLit(s) => Value::StringVal(s),
            Node::Boolean(b) => Value::Boolean(b),
            Node::Null => Value::Null,

            Node::VarLDecl { name, value } => self.eval_var_l(name, *value),
            Node::VarGDecl { name, value } => self.eval_var_g(name, *value),
            Node::ConstDecl { name, value } => self.eval_const(name, *value),
            Node::UpdateDecl { name, value } => self.eval_update(name, *value),
            Node::UpdateIndex { name, index, value } => self.eval_update_index(name, *index, *value),
            Node::VarAccess(name) => self.eval_var_access(&name),
            Node::Summon(name) => self.eval_summon(name),
            Node::MultiVarL { names, values } => self.eval_multi_var_l(names, values),

            Node::BinaryOp { left, op, right } => {
                let l = self.eval(*left);
                let r = self.eval(*right);
                apply_op(l, &op, r)
            }
            Node::UnaryOp { op, operand } => {
                let val = self.eval(*operand);
                match op.as_str() {
                    "Not" => match val { Value::Boolean(b) => Value::Boolean(!b), _ => Value::Null },
                    "Negate" => match val {
                        Value::Integer(n) => Value::Integer(-n),
                        Value::Float(n) => Value::Float(-n),
                        _ => Value::Null,
                    },
                    _ => Value::Null,
                }
            }

            Node::Check { condition, body, or_checks, else_body } => {
                self.eval_check(*condition, body, or_checks, else_body)
            }
            Node::Circle { name, count, body } => self.eval_circle(name, *count, body),
            Node::Shatter => Value::Break,
            Node::Skip => Value::Continue,

            Node::FuncDecl { name, func_type, params, body } => self.eval_func_decl(name, func_type, params, body),
            Node::FuncCall { name, args } => self.eval_func_call(name, args),
            Node::Reply(expr) => Value::Return(Box::new(self.eval(*expr))),

            Node::Rest(duration) => self.eval_rest(*duration),
            Node::Wait(duration) => self.eval_wait(*duration),
            Node::TriggerCall { name, trigger_type, value } => self.eval_trigger_call(name, trigger_type, *value),
            Node::Guard { condition, body } => self.eval_guard(*condition, body),
            Node::Attempt { body, rescue_param, rescue_body, always_body } => {
                self.eval_attempt(body, rescue_param, rescue_body, always_body)
            }
            Node::Protect { vars, body } => self.eval_protect(vars, body),
            Node::AsyncBlock { body } => self.eval_async_block(body),

            Node::Print(expr) => self.eval_print(*expr),
            Node::InputExpr(prompt) => self.eval_input(*prompt),
            Node::TypeOf(expr) => self.eval_typeof(*expr),
            Node::ToInt(expr) => self.eval_to_int(*expr),
            Node::ToFloat(expr) => self.eval_to_float_expr(*expr),
            Node::ToString(expr) => self.eval_to_string(*expr),
            Node::ToBool(expr) => self.eval_to_bool(*expr),

            Node::Array(elements) => {
                let mut arr = Vec::new();
                for el in elements { arr.push(self.eval(el)); }
                Value::Array(arr)
            }
            Node::IndexAccess { name, index } => self.eval_index_access(name, *index),
            Node::MapLit { keys, values } => self.eval_map_lit(keys, values),

            Node::MethodCall { object, method, args } => self.eval_method_call(object, method, args),
            Node::MathCall { method, args } => self.eval_math_call(method, args),
            Node::FileCall { method, args } => self.eval_file_call(method, args),
            Node::JsonCall { method, args } => self.eval_json_call(method, args),
            Node::DateCall { method, args } => self.eval_date_call(method, args),
            Node::SystemCall { method, args } => self.eval_system_call(method, args),
            Node::HttpCall { method, args } => self.eval_http_call(method, args),
            Node::CryptoCall { method, args } => self.eval_crypto_call(method, args),
            Node::UseModule(path) => self.eval_use_module(path),
        }
    }
}