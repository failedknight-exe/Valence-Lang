use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;

impl Evaluator {
    pub fn eval_string_method(&mut self, _object: &str, method: &str, s: &str, args: Vec<Node>) -> Value {
        match method {
            "len" => Value::Integer(s.chars().count() as i64),
            "upper" => Value::StringVal(s.to_uppercase()),
            "lower" => Value::StringVal(s.to_lowercase()),
            "trim" => Value::StringVal(s.trim().to_string()),
            "reverse" => Value::StringVal(s.chars().rev().collect()),
            "contains" => {
                if args.is_empty() { return Value::Null; }
                let check = self.eval(args[0].clone());
                if let Value::StringVal(search) = check { Value::Boolean(s.contains(&search)) } else { Value::Boolean(false) }
            }
            "startsWith" => {
                if args.is_empty() { return Value::Boolean(false); }
                let prefix = self.eval(args[0].clone());
                if let Value::StringVal(p) = prefix { Value::Boolean(s.starts_with(&p)) } else { Value::Boolean(false) }
            }
            "endsWith" => {
                if args.is_empty() { return Value::Boolean(false); }
                let suffix = self.eval(args[0].clone());
                if let Value::StringVal(suf) = suffix { Value::Boolean(s.ends_with(&suf)) } else { Value::Boolean(false) }
            }
            "replace" => {
                if args.len() < 2 { return Value::Null; }
                let find = self.eval(args[0].clone());
                let rep = self.eval(args[1].clone());
                if let (Value::StringVal(f), Value::StringVal(r)) = (find, rep) { Value::StringVal(s.replace(&f, &r)) } else { Value::Null }
            }
            "split" => {
                if args.is_empty() { return Value::Null; }
                let sep = self.eval(args[0].clone());
                if let Value::StringVal(separator) = sep {
                    let parts: Vec<Value> = s.split(&separator).map(|p| Value::StringVal(p.to_string())).collect();
                    Value::Array(parts)
                } else { Value::Null }
            }
            "slice" => {
                if args.len() < 2 { return Value::Null; }
                let start = match self.eval(args[0].clone()) { Value::Integer(i) => i as usize, _ => 0 };
                let end = match self.eval(args[1].clone()) { Value::Integer(i) => i as usize, _ => s.chars().count() };
                let count = s.chars().count();
                if start > end || end > count { return Value::StringVal("".to_string()); }
                let sliced: String = s.chars().skip(start).take(end - start).collect();
                Value::StringVal(sliced)
            }
            "charAt" => {
                if args.is_empty() { return Value::Null; }
                let idx = match self.eval(args[0].clone()) { Value::Integer(i) => i as usize, _ => return Value::Null };
                match s.chars().nth(idx) {
                    Some(c) => Value::StringVal(c.to_string()),
                    None => Value::Null,
                }
            }
            "repeat" => {
                if args.is_empty() { return Value::StringVal(s.to_string()); }
                let times = match self.eval(args[0].clone()) { Value::Integer(i) => i as usize, _ => 1 };
                Value::StringVal(s.repeat(times))
            }
            "capitalize" => {
                let words: Vec<String> = s.split(' ')
                    .map(|w| {
                        let mut chars = w.chars();
                        match chars.next() {
                            Some(first) => {
                                let upper = first.to_uppercase().to_string();
                                upper + &chars.collect::<String>()
                            }
                            None => String::new(),
                        }
                    })
                    .collect();
                Value::StringVal(words.join(" "))
            }
            "camelCase" => {
                let words: Vec<&str> = s.split(' ').collect();
                let mut result = String::new();
                for (i, word) in words.iter().enumerate() {
                    if i == 0 {
                        result.push_str(&word.to_lowercase());
                    } else {
                        let mut chars = word.chars();
                        if let Some(first) = chars.next() {
                            result.push_str(&first.to_uppercase().to_string());
                            result.push_str(&chars.collect::<String>().to_lowercase());
                        }
                    }
                }
                Value::StringVal(result)
            }
            "snakeCase" => {
                let mut result = String::new();
                for (i, c) in s.chars().enumerate() {
                    if c.is_uppercase() {
                        if i > 0 { result.push('_'); }
                        result.push(c.to_lowercase().next().unwrap());
                    } else if c == ' ' {
                        result.push('_');
                    } else {
                        result.push(c);
                    }
                }
                Value::StringVal(result)
            }
            "padLeft" => {
                if args.len() < 2 { return Value::Null; }
                let width = match self.eval(args[0].clone()) { Value::Integer(n) => n as usize, _ => return Value::Null };
                let pad_char = match self.eval(args[1].clone()) { Value::StringVal(c) => c.chars().next().unwrap_or(' '), _ => ' ' };
                if s.len() >= width { Value::StringVal(s.to_string()) } else {
                    let padding = std::iter::repeat(pad_char).take(width - s.len()).collect::<String>();
                    Value::StringVal(format!("{}{}", padding, s))
                }
            }
            "padRight" => {
                if args.len() < 2 { return Value::Null; }
                let width = match self.eval(args[0].clone()) { Value::Integer(n) => n as usize, _ => return Value::Null };
                let pad_char = match self.eval(args[1].clone()) { Value::StringVal(c) => c.chars().next().unwrap_or(' '), _ => ' ' };
                if s.len() >= width { Value::StringVal(s.to_string()) } else {
                    let padding = std::iter::repeat(pad_char).take(width - s.len()).collect::<String>();
                    Value::StringVal(format!("{}{}", s, padding))
                }
            }
            "isNumeric" => {
                Value::Boolean(!s.is_empty() && s.chars().all(|c| c.is_numeric()))
            }
            "isAlpha" => {
                Value::Boolean(!s.is_empty() && s.chars().all(|c| c.is_alphabetic()))
            }
            "isEmail" => {
                Value::Boolean(s.contains('@') && s.contains('.') && s.len() > 5)
            }
            "wordCount" => {
                Value::Integer(s.split_whitespace().count() as i64)
            }
            "truncate" => {
                if args.is_empty() { return Value::StringVal(s.to_string()); }
                let max_len = match self.eval(args[0].clone()) { Value::Integer(n) => n as usize, _ => return Value::StringVal(s.to_string()) };
                let char_count = s.chars().count();
                if char_count <= max_len { Value::StringVal(s.to_string()) } else {
                    let truncated: String = s.chars().take(max_len).collect();
                    Value::StringVal(format!("{}...", truncated))
                }
            }
            "slug" => {
                let result = s.to_lowercase()
                    .chars()
                    .map(|c| {
                        if c.is_alphanumeric() { c }
                        else if c == ' ' { '-' }
                        else { ' ' }
                    })
                    .collect::<String>();
                let cleaned = result.split_whitespace().collect::<Vec<&str>>().join("");
                Value::StringVal(cleaned)
            }
            "urlEncode" => {
                let mut result = String::new();
                for c in s.chars() {
                    match c {
                        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => { result.push(c); }
                        ' ' => result.push_str("%20"),
                        _ => {
                            for b in c.to_string().bytes() { result.push_str(&format!("%{:02X}", b)); }
                        }
                    }
                }
                Value::StringVal(result)
            }
            "urlDecode" => {
                let mut result = String::new();
                let mut chars = s.chars();
                while let Some(c) = chars.next() {
                    if c == '%' {
                        let hex: String = chars.by_ref().take(2).collect();
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) { result.push(byte as char); }
                    } else if c == '+' {
                        result.push(' ');
                    } else {
                        result.push(c);
                    }
                }
                Value::StringVal(result)
            }
            _ => Value::Null,
        }
    }
}