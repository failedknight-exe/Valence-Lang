use super::Evaluator;
use crate::evaluator::value::Value;
use crate::parser::Node;
use std::io::Read;

impl Evaluator {
    pub fn eval_http_builtin(&mut self, method: &str, args: Vec<Node>) -> Value {
        match method {
            "get" => {
                if args.is_empty() { return Value::Error("http.get needs a URL. Without a URL, this is just a staring contest with a dead network.".to_string()); }
                let url = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("URL must be a string".to_string()),
                };
                match ureq::get(&url).call() {
                    Ok(res) => match res.into_string() {
                        Ok(body) => Value::StringVal(body),
                        Err(e) => Value::Error(format!("Failed to parse response body: {}. The server sent a message, but the decoder waved the white flag.", e)),
                    },
                    Err(e) => Value::Error(format!("HTTP GET failed: {}. The endpoint said no, and your code heard it as a challenge.", e)),
                }
            }
            "post" => {
                if args.len() < 2 { return Value::Error("http.post needs URL and body string. Posting without a body is just shouting into the void.".to_string()); }
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
                if args.len() < 2 { return Value::Error("http.put needs URL and body. You can't update the internet with a shrug and a prayer.".to_string()); }
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
                    Err(e) => Value::Error(format!("HTTP PUT failed: {}. The server rejected your update like a bad review.", e)),
                }
            }
            "delete" => {
                if args.is_empty() { return Value::Error("http.delete needs URL. Deleting without a target is just digital vandalism without a victim.".to_string()); }
                let url = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("URL must be a string".to_string()),
                };
                match ureq::delete(&url).call() {
                    Ok(res) => match res.into_string() {
                        Ok(body) => Value::StringVal(body),
                        Err(e) => Value::Error(format!("Failed to read response: {}", e)),
                    },
                    Err(e) => Value::Error(format!("HTTP DELETE failed: {}. The service refused to erase your mistake on principle.", e)),
                }
            }
            "status" => {
                if args.is_empty() { return Value::Error("http.status needs a URL. Without a destination, even the status code is just a rumor.".to_string()); }
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
                if args.len() < 2 { return Value::Error("http.download needs URL and destination filename. You can't download a file to nowhere and call it a workflow.".to_string()); }
                let url = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("URL must be a string".to_string()),
                };
                let filename = match self.eval(args[1].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("Filename must be a string. A number is not a destination, and a boolean is not a plan.".to_string()),
                };
                match ureq::get(&url).call() {
                    Ok(res) => {
                        let mut bytes = Vec::new();
                        match res.into_reader().read_to_end(&mut bytes) {
                            Ok(_) => match std::fs::write(&filename, &bytes) {
                                Ok(_) => Value::Boolean(true),
                                Err(e) => Value::Error(format!("Failed to save download: {}. The file fought back and won.", e)),
                            },
                            Err(e) => Value::Error(format!("Failed to read payload: {}. The response arrived, then immediately ghosted your parser.", e)),
                        }
                    }
                    Err(e) => Value::Error(format!("Download failed: {}. The internet is giving you a cold shoulder.", e)),
                }
            }
            "getJson" => {
                if args.is_empty() { return Value::Error("http.getJson needs URL. JSON without a URL is just a sad dictionary.".to_string()); }
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
                                Value::map(map)
                            } else {
                                Value::StringVal(body)
                            }
                        }
                        Err(e) => Value::Error(format!("Failed to decode response: {}. The JSON was there; your decoder just wasn't ready for the drama.", e)),
                    },
                    Err(e) => Value::Error(format!("HTTP request failed: {}", e)),
                }
            }
            "getH" => {
                if args.len() < 2 { return Value::Error("http.getH needs URL and headers map. A request without headers is like texting with a broken keyboard.".to_string()); }
                let url = match self.eval(args[0].clone()) {
                    Value::StringVal(s) => s,
                    _ => return Value::Error("URL must be a string".to_string()),
                };
                let headers = match self.eval(args[1].clone()) {
                    Value::Map(m) => m,
                    _ => return Value::Error("Headers must be a Map".to_string()),
                };
                let guard = headers.read().unwrap();
                let mut req = ureq::get(&url);
                for (k, v) in guard.iter() {
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
                if args.len() < 3 { return Value::Error("http.postH needs URL, body, and headers map. Posting without headers is just a dramatic handshake with no rules.".to_string()); }
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
                let guard = headers.read().unwrap();
                let mut req = ureq::post(&url);
                for (k, v) in guard.iter() {
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
            _ => Value::Error(format!("'{}' is not a valid http method. That's not an HTTP verb; it's a typo with ambition.", method)),
        }
    }
}