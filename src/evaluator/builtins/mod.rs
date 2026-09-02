pub mod math;
pub mod strings;
pub mod arrays;
pub mod maps;
pub mod file_io;
pub mod http;
pub mod json;
pub mod crypto;
pub mod date;
pub mod system;
pub mod conversion;
pub mod io;

use super::Evaluator;
use super::value::Value;
use crate::parser::Node;

impl Evaluator {
    pub fn eval_method_call(&mut self, object: String, method: String, args: Vec<Node>) -> Value {
        if self.constants.contains_key(&object) {
            let muts = ["push", "pop", "reverse", "sort", "clear", "delete"];
            if muts.contains(&method.as_str()) {
                return Value::Error(format!("Cannot call mutating method '{}' on constant '{}'", method, object));
            }
        }

        let val = if let Some(v) = self.local_vars.get(&object) { v.clone() }
            else if let Some(v) = self.global_vars.get(&object) { v.clone() }
            else if let Some(v) = self.constants.get(&object) { v.clone() }
            else { return Value::Error(format!("'{}' was never declared", object)); };

        match val {
            Value::Array(elements) => self.eval_array_method(&object, &method, &elements, args),
            Value::StringVal(s) => self.eval_string_method(&object, &method, &s, args),
            Value::Map(map) => self.eval_map_method(&object, &method, &map, args),
            _ => Value::Null,
        }
    }

    pub fn eval_math_call(&mut self, method: String, args: Vec<Node>) -> Value {
        self.eval_math_builtin(&method, args)
    }

    pub fn eval_file_call(&mut self, method: String, args: Vec<Node>) -> Value {
        self.eval_file_builtin(&method, args)
    }

    pub fn eval_json_call(&mut self, method: String, args: Vec<Node>) -> Value {
        self.eval_json_builtin(&method, args)
    }

    pub fn eval_date_call(&mut self, method: String, args: Vec<Node>) -> Value {
        self.eval_date_builtin(&method, args)
    }

    pub fn eval_system_call(&mut self, method: String, args: Vec<Node>) -> Value {
        self.eval_system_builtin(&method, args)
    }

    pub fn eval_http_call(&mut self, method: String, args: Vec<Node>) -> Value {
        self.eval_http_builtin(&method, args)
    }

    pub fn eval_crypto_call(&mut self, method: String, args: Vec<Node>) -> Value {
        self.eval_crypto_builtin(&method, args)
    }

    pub fn eval_use_module(&mut self, path: String) -> Value {
        match std::fs::read_to_string(&path) {
            Ok(source) => {
                let mut lexer = crate::lexer::Lexer::new(&source);
                let tokens = lexer.tokenize();
                let mut parser = crate::parser::Parser::new(tokens);
                let ast = parser.parse();
                for node in ast { self.eval(node); }
                Value::Null
            }
            Err(_) => Value::Error(format!("Could not load module '{}'.", path)),
        }
    }
}