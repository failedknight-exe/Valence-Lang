use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;
use std::io::Read;

impl Evaluator {
    pub fn eval_http_builtin(&mut self, method: &str, args: Vec<Node>) -> Value {
        match method {
            "get" => {
                if args.is_empty() { return Value::Error("http.get needs a URL".to_string()); }
                let url = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("URL must be a string".to_string()),
                };
                match ureq::get(&url).call() {
                    Ok(res) => match res.into_string() {
                        Ok(body) => Value::StringVal(body),
                        Err(e) => Value::Error(format!("Failed to parse response body: {}", e)),
                    },
                    Err(e) => Value::Error(format!("HTTP GET failed: {}", e)),
                }
            }
            "post" => {
                if args.len() < 2 { return Value::Error("http.post needs URL and body string".to_string()); }
                let url = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("URL must be a string".to_string()),
                };
                let body = match self.eval(args[1].clone()) {
                    Value::StringVal(s) => s,
                    other => format!("{}", other),
                };
                match ureq::post(&url).set("Content-Type", "application/json").send_string(&body) {
                    Ok(res) => match res.into_string() {
                        Ok(body) => Value::StringVal(body),
                        Err(e) => Value::Error(format!("Failed to read response: {}", e)),
                    },
                    Err(e) => Value::Error(format!("HTTP POST failed: {}", e)),
                }
            }
            "put" => {
                if args.len() < 2 { return Value::Error("http.put needs URL and body".to_string()); }
                let url = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("URL must be a string".to_string()),
                };
                let body = match self.eval(args[1].clone()) {
                    Value::StringVal(s) => s,
                    other => format!("{}", other),
                };
                match ureq::put(&url).set("Content-Type", "application/json").send_string(&body) {
                    Ok(res) => match res.into_string() {
                        Ok(body) => Value::StringVal(body),
                        Err(e) => Value::Error(format!("Failed to read response: {}", e)),
                    },
                    Err(e) => Value::Error(format!("HTTP PUT failed: {}", e)),
                }
            }
            "delete" => {
                if args.is_empty() { return Value::Error("http.delete needs URL".to_string()); }
                let url = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("URL must be a string".to_string()),
                };
                match ureq::delete(&url).call() {
                    Ok(res) => match res.into_string() {
                        Ok(body) => Value::StringVal(body),
                        Err(e) => Value::Error(format!("Failed to read response: {}", e)),
                    },
                    Err(e) => Value::Error(format!("HTTP DELETE failed: {}", e)),
                }
            }
            "status" => {
                if args.is_empty() { return Value::Error("http.status needs a URL".to_string()); }
                let url = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("URL must be a string".to_string()),
                };
                match ureq::get(&url).call() {
                    Ok(res) => Value::Integer(res.status() as i64),
                    Err(ureq::Error::Status(code, _)) => Value::Integer(code as i64),
                    Err(e) => Value::Error(format!("HTTP request failed: {}", e)),
                }
            }
            "download" => {
                if args.len() < 2 { return Value::Error("http.download needs URL and destination filename".to_string()); }
                let url = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("URL must be a string".to_string()),
                };
                let filename = match self.eval(args[1].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("Filename must be a string".to_string()),
                };
                match ureq::get(&url).call() {
                    Ok(res) => {
                        let mut bytes = Vec::new();
                        match res.into_reader().read_to_end(&mut bytes) {
                            Ok(_) => match std::fs::write(&filename, &bytes) {
                                Ok(_) => Value::Boolean(true),
                                Err(e) => Value::Error(format!("Failed to save download: {}", e)),
                            },
                            Err(e) => Value::Error(format!("Failed to read payload: {}", e)),
                        }
                    }
                    Err(e) => Value::Error(format!("Download failed: {}", e)),
                }
            }
            "getJson" => {
                if args.is_empty() { return Value::Error("http.getJson needs URL".to_string()); }
                let url = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("URL must be a string".to_string()),
                };
                match ureq::get(&url).set("Accept", "application/json").call() {
                    Ok(res) => match res.into_string() {
                        Ok(body) => {
                            let trimmed = body.trim();
                            if trimmed.starts_with('{') {
                                let mut map = std::collections::HashMap::new();
                                let inner = &trimmed[1..trimmed.len() - 1];
                                for pair in inner.split(',') {
                                    let kv: Vec<&str> = pair.splitn(2, ':').collect();
                                    if kv.len() == 2 {
                                        let key = kv[0].trim().trim_matches('"').to_string();
                                        let val_str = kv[1].trim();
                                        let value = if val_str.starts_with('"') {
                                            Value::StringVal(val_str.trim_matches('"').to_string())
                                        } else if let Ok(n) = val_str.parse::<i64>() {
                                            Value::Integer(n)
                                        } else if let Ok(f) = val_str.parse::<f64>() {
                                            Value::Float(f)
                                        } else {
                                            Value::StringVal(val_str.to_string())
                                        };
                                        map.insert(key, value);
                                    }
                                }
                                Value::Map(map)
                            } else {
                                Value::StringVal(body)
                            }
                        }
                        Err(e) => Value::Error(format!("Failed to decode response: {}", e)),
                    },
                    Err(e) => Value::Error(format!("HTTP request failed: {}", e)),
                }
            }
            "getH" => {
                if args.len() < 2 { return Value::Error("http.getH needs URL and headers map".to_string()); }
                let url = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("URL must be a string".to_string()),
                };
                let headers = match self.eval(args[1].clone()) {
                    Value::Map(m) => m,
                    _ => return Value::Error("Headers must be a Map".to_string()),
                };
                let mut req = ureq::get(&url);
                for (k, v) in headers.iter() {
                    req = req.set(k, &format!("{}", v));
                }
                match req.call() {
                    Ok(res) => match res.into_string() {
                        Ok(body) => Value::StringVal(body),
                        Err(e) => Value::Error(format!("Failed to read: {}", e)),
                    },
                    Err(e) => Value::Error(format!("HTTP request failed: {}", e)),
                }
            }
            "postH" => {
                if args.len() < 3 { return Value::Error("http.postH needs URL, body, and headers map".to_string()); }
                let url = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("URL must be a string".to_string()),
                };
                let body = match self.eval(args[1].clone()) {
                    Value::StringVal(s) => s,
                    other => format!("{}", other),
                };
                let headers = match self.eval(args[2].clone()) {
                    Value::Map(m) => m,
                    _ => return Value::Error("Headers must be a Map".to_string()),
                };
                let mut req = ureq::post(&url);
                for (k, v) in headers.iter() {
                    req = req.set(k, &format!("{}", v));
                }
                match req.send_string(&body) {
                    Ok(res) => match res.into_string() {
                        Ok(body) => Value::StringVal(body),
                        Err(e) => Value::Error(format!("Failed to read: {}", e)),
                    },
                    Err(e) => Value::Error(format!("HTTP POST failed: {}", e)),
                }
            }
            _ => Value::Error(format!("'{}' is not a valid http method", method)),
        }
    }
}