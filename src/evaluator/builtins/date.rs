// src/evaluator/builtins/date.rs

use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;
use std::time::{SystemTime, UNIX_EPOCH};

impl Evaluator {
    pub fn eval_date_builtin(&mut self, method: &str, args: Vec<Node>) -> Value {
        match method {
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
            "micro_time" => {
                let start = SystemTime::now();
                let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
                Value::Integer(since_the_epoch.as_micros() as i64)
            }
            "nano_time" => {
                let start = SystemTime::now();
                let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
                Value::Integer(since_the_epoch.as_nanos() as i64)
            }
            _ => Value::Error(format!("'{}' is not a valid date method. That call has less calendar sense than a weekend without coffee.", method)),
        }
    }
}