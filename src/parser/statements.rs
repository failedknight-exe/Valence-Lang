use super::Parser;
use super::ast::Node;
use crate::lexer::Token;

impl Parser {
    pub fn parse_statement(&mut self) -> Option<Node> {
        match self.peek().clone() {
            Token::VarL    => self.parse_varl(),
            Token::VarG    => self.parse_varg(),
            Token::Const   => self.parse_const(),
            Token::Update  => self.parse_update(),
            Token::Print   => self.parse_print(),
            Token::Circle  => self.parse_circle(),
            Token::Check   => self.parse_check(),
            Token::Func    => self.parse_func(),
            Token::Reply   => self.parse_reply(),
            Token::Rest    => self.parse_rest(),
            Token::Wait    => self.parse_wait(),
            Token::Trigger => self.parse_trigger(),
            Token::Guard   => self.parse_guard(),
            Token::Use     => self.parse_use(),
            Token::Attempt => self.parse_attempt(),
            Token::Protect => self.parse_protect(),
            Token::Async   => self.parse_async(),
            Token::Shatter => { self.advance(); Some(Node::Shatter) }
            Token::Skip    => { self.advance(); Some(Node::Skip) }

            Token::Summon => {
                self.advance();
                let name = match self.advance().clone() {
                    Token::Ident(n) => n,
                    _ => { println!("[PARSE ERROR] Expected identifier after 'summon'."); return None; }
                };
                Some(Node::Summon(name))
            }

            Token::Ident(_) => {
                match self.peek_ahead() {
                    Token::LParen => self.parse_func_call_statement(),
                    Token::Dot    => self.parse_method_call_statement(),
                    _ => {
                        println!("[PARSE ERROR] Unexpected token: {:?}", self.peek());
                        self.advance();
                        None
                    }
                }
            }

            Token::Math   => self.parse_builtin_stmt("math"),
            Token::File   => self.parse_builtin_stmt("file"),
            Token::Json   => self.parse_builtin_stmt("json"),
            Token::Date   => self.parse_builtin_stmt("date"),
            Token::System => self.parse_builtin_stmt("system"),
            Token::Http   => self.parse_builtin_stmt("http"),
            Token::Crypto => self.parse_builtin_stmt("crypto"),

            Token::Rewind => {
                self.advance();
                self.expect(&Token::LParen);
                let count = self.parse_expression()?;
                self.expect(&Token::RParen);
                Some(Node::Rewind(Box::new(count)))
            }

            Token::Db => self.parse_builtin_stmt("db"),

            _ => {
                println!("[PARSE ERROR] Unexpected token: {:?}", self.peek());
                self.advance();
                None
            }
        }
    }

    fn parse_varl(&mut self) -> Option<Node> {
        self.advance();
        let name = match self.advance().clone() {
            Token::Ident(n) => n,
            _ => return None,
        };
        if self.peek() == &Token::Comma {
            return self.parse_multi_varl(name);
        }
        if !self.expect(&Token::Equals) { return None; }
        let value = self.parse_expression()?;
        Some(Node::VarLDecl { name, value: Box::new(value) })
    }

    fn parse_multi_varl(&mut self, first_name: String) -> Option<Node> {
        let mut names = vec![first_name];
        while self.peek() == &Token::Comma {
            self.advance();
            match self.advance().clone() {
                Token::Ident(n) => names.push(n),
                _ => return None,
            }
        }
        self.expect(&Token::Equals);
        let mut values = Vec::new();
        values.push(self.parse_expression()?);
        while self.peek() == &Token::Comma {
            self.advance();
            values.push(self.parse_expression()?);
        }
        if names.len() != values.len() {
            println!("[PARSE ERROR] Multi-assign mismatch.");
            return None;
        }
        Some(Node::MultiVarL {
            names,
            values: values.into_iter().map(|v| Box::new(v)).collect(),
        })
    }

    fn parse_varg(&mut self) -> Option<Node> {
        self.advance();
        let name = match self.advance().clone() {
            Token::Ident(n) => n,
            _ => return None,
        };
        self.expect(&Token::Equals);
        let value = self.parse_expression()?;
        Some(Node::VarGDecl { name, value: Box::new(value) })
    }

    fn parse_const(&mut self) -> Option<Node> {
        self.advance();
        let name = match self.advance().clone() {
            Token::Ident(n) => n,
            _ => return None,
        };
        if !self.expect(&Token::Equals) { return None; }
        let value = self.parse_expression()?;
        Some(Node::ConstDecl { name, value: Box::new(value) })
    }

    fn parse_update(&mut self) -> Option<Node> {
        self.advance();
        let name = match self.advance().clone() {
            Token::Ident(n) => n,
            _ => return None,
        };
        if self.peek() == &Token::LBracket {
            self.advance();
            let index = self.parse_expression()?;
            self.expect(&Token::RBracket);
            self.expect(&Token::Equals);
            let value = self.parse_expression()?;
            return Some(Node::UpdateIndex { name, index: Box::new(index), value: Box::new(value) });
        }
        self.expect(&Token::Equals);
        let value = self.parse_expression()?;
        Some(Node::UpdateDecl { name, value: Box::new(value) })
    }

    fn parse_print(&mut self) -> Option<Node> {
        self.advance();
        self.expect(&Token::LParen);
        let value = self.parse_expression()?;
        self.expect(&Token::RParen);
        Some(Node::Print(Box::new(value)))
    }

    fn parse_circle(&mut self) -> Option<Node> {
        self.advance();
        let name = match self.advance().clone() {
            Token::Ident(n) => n,
            _ => return None,
        };
        self.expect(&Token::LParen);
        let count = self.parse_expression()?;
        self.expect(&Token::RParen);
        self.expect(&Token::LBrace);
        let body = self.parse_block()?;
        self.expect(&Token::RBrace);
        Some(Node::Circle { name, count: Box::new(count), body })
    }

    fn parse_check(&mut self) -> Option<Node> {
        self.advance();
        self.expect(&Token::LParen);
        let condition = self.parse_expression()?;
        self.expect(&Token::RParen);
        self.expect(&Token::LBrace);
        let body = self.parse_block()?;
        self.expect(&Token::RBrace);

        let mut or_checks = Vec::new();
        while self.peek() == &Token::OrCheck {
            self.advance();
            self.expect(&Token::LParen);
            let or_cond = self.parse_expression()?;
            self.expect(&Token::RParen);
            self.expect(&Token::LBrace);
            let or_body = self.parse_block()?;
            self.expect(&Token::RBrace);
            or_checks.push((or_cond, or_body));
        }

        let else_body = if self.peek() == &Token::Else {
            self.advance();
            self.expect(&Token::LBrace);
            let else_nodes = self.parse_block()?;
            self.expect(&Token::RBrace);
            Some(else_nodes)
        } else { None };

        Some(Node::Check { condition: Box::new(condition), body, or_checks, else_body })
    }

    fn parse_func(&mut self) -> Option<Node> {
        self.advance();
        self.expect(&Token::LBracket);
        let func_type = match self.advance().clone() {
            Token::Callable => "callable".to_string(),
            Token::Auto     => "auto".to_string(),
            Token::Forever  => "forever".to_string(),
            _ => return None,
        };
        self.expect(&Token::RBracket);
        let name = match self.advance().clone() {
            Token::Ident(n) => n,
            _ => return None,
        };
        let mut params = Vec::new();
        self.expect(&Token::LParen);
        if self.peek() != &Token::RParen {
            match self.advance().clone() {
                Token::Ident(p) => params.push(p),
                _ => return None,
            }
            while self.peek() == &Token::Comma {
                self.advance();
                match self.advance().clone() {
                    Token::Ident(p) => params.push(p),
                    _ => return None,
                }
            }
        }
        self.expect(&Token::RParen);
        self.expect(&Token::LBrace);
        let body = self.parse_block()?;
        self.expect(&Token::RBrace);
        Some(Node::FuncDecl { name, func_type, params, body })
    }

    fn parse_reply(&mut self) -> Option<Node> {
        self.advance();
        let value = self.parse_expression()?;
        Some(Node::Reply(Box::new(value)))
    }

    fn parse_rest(&mut self) -> Option<Node> {
        self.advance();
        self.expect(&Token::LParen);
        let duration = self.parse_expression()?;
        self.expect(&Token::RParen);
        Some(Node::Rest(Box::new(duration)))
    }

    fn parse_wait(&mut self) -> Option<Node> {
        self.advance();
        self.expect(&Token::LParen);
        let duration = self.parse_expression()?;
        self.expect(&Token::RParen);
        Some(Node::Wait(Box::new(duration)))
    }

    fn parse_trigger(&mut self) -> Option<Node> {
        self.advance();
        let name = match self.advance().clone() {
            Token::Ident(n) => n,
            _ => { println!("[PARSE ERROR] Expected function name after trigger"); return None; }
        };
        self.expect(&Token::LBracket);
        let trigger_type = match self.advance().clone() {
            Token::Ident(t) => t,
            Token::When => "when".to_string(),
            _ => { println!("[PARSE ERROR] Expected trigger type"); return None; }
        };
        self.expect(&Token::RBracket);
        self.expect(&Token::LParen);
        let value = self.parse_expression()?;
        self.expect(&Token::RParen);
        Some(Node::TriggerCall { name, trigger_type, value: Box::new(value) })
    }

    fn parse_guard(&mut self) -> Option<Node> {
        self.advance();
        self.expect(&Token::LParen);
        let condition = self.parse_expression()?;
        self.expect(&Token::RParen);
        self.expect(&Token::LBrace);
        let body = self.parse_block()?;
        self.expect(&Token::RBrace);
        Some(Node::Guard { condition: Box::new(condition), body })
    }

    fn parse_use(&mut self) -> Option<Node> {
        self.advance();
        match self.advance().clone() {
            Token::StringLit(path) => Some(Node::UseModule(path)),
            _ => { println!("[PARSE ERROR] Expected file path after 'use'."); None }
        }
    }

    fn parse_attempt(&mut self) -> Option<Node> {
        self.advance();
        self.expect(&Token::LBrace);
        let body = self.parse_block()?;
        self.expect(&Token::RBrace);

        let mut rescue_param: Option<String> = None;
        let mut rescue_body: Option<Vec<Node>> = None;
        if self.peek() == &Token::Rescue {
            self.advance();
            if self.peek() == &Token::LParen {
                self.advance();
                rescue_param = match self.advance().clone() {
                    Token::Ident(name) => Some(name),
                    _ => None,
                };
                self.expect(&Token::RParen);
            }
            self.expect(&Token::LBrace);
            rescue_body = Some(self.parse_block()?);
            self.expect(&Token::RBrace);
        }

        let mut always_body: Option<Vec<Node>> = None;
        if self.peek() == &Token::Always {
            self.advance();
            self.expect(&Token::LBrace);
            always_body = Some(self.parse_block()?);
            self.expect(&Token::RBrace);
        }

        Some(Node::Attempt { body, rescue_param, rescue_body, always_body })
    }

    fn parse_protect(&mut self) -> Option<Node> {
        self.advance();
        let mut vars = Vec::new();
        if self.peek() == &Token::LParen {
            self.advance();
            if self.peek() != &Token::RParen {
                match self.advance().clone() {
                    Token::Ident(v) => vars.push(v),
                    _ => return None,
                }
                while self.peek() == &Token::Comma {
                    self.advance();
                    match self.advance().clone() {
                        Token::Ident(v) => vars.push(v),
                        _ => return None,
                    }
                }
            }
            self.expect(&Token::RParen);
        }
        self.expect(&Token::LBrace);
        let body = self.parse_block()?;
        self.expect(&Token::RBrace);
        Some(Node::Protect { vars, body })
    }

    fn parse_async(&mut self) -> Option<Node> {
        self.advance();
        self.expect(&Token::LBrace);
        let body = self.parse_block()?;
        self.expect(&Token::RBrace);
        Some(Node::AsyncBlock { body })
    }

    fn parse_func_call_statement(&mut self) -> Option<Node> {
        let name = match self.advance().clone() {
            Token::Ident(n) => n,
            _ => return None,
        };
        self.expect(&Token::LParen);
        let mut args = Vec::new();
        if self.peek() != &Token::RParen {
            args.push(self.parse_expression()?);
            while self.peek() == &Token::Comma {
                self.advance();
                args.push(self.parse_expression()?);
            }
        }
        self.expect(&Token::RParen);
        Some(Node::FuncCall { name, args })
    }

    fn parse_method_call_statement(&mut self) -> Option<Node> {
        let name = match self.advance().clone() {
            Token::Ident(n) => n,
            _ => return None,
        };
        self.advance(); // dot
        let method = match self.advance().clone() {
            Token::Ident(m) => m,
            _ => return None,
        };
        let mut args = Vec::new();
        if self.peek() == &Token::LParen {
            self.advance();
            if self.peek() != &Token::RParen {
                args.push(self.parse_expression()?);
                while self.peek() == &Token::Comma {
                    self.advance();
                    args.push(self.parse_expression()?);
                }
            }
            self.expect(&Token::RParen);
        }
        Some(Node::MethodCall { object: name, method, args })
    }

    fn parse_builtin_stmt(&mut self, kind: &str) -> Option<Node> {
        self.advance();
        self.expect(&Token::Dot);
        let method = match self.advance().clone() {
            Token::Ident(m) => m,
            _ => return None,
        };
        let mut args = Vec::new();
        if self.peek() == &Token::LParen {
            self.advance();
            if self.peek() != &Token::RParen {
                args.push(self.parse_expression()?);
                while self.peek() == &Token::Comma {
                    self.advance();
                    args.push(self.parse_expression()?);
                }
            }
            self.expect(&Token::RParen);
        }
        match kind {
            "math"   => Some(Node::MathCall { method, args }),
            "file"   => Some(Node::FileCall { method, args }),
            "json"   => Some(Node::JsonCall { method, args }),
            "date"   => Some(Node::DateCall { method, args }),
            "system" => Some(Node::SystemCall { method, args }),
            "http"   => Some(Node::HttpCall { method, args }),
            "crypto" => Some(Node::CryptoCall { method, args }),
            "db"     => Some(Node::DbCall { method, args }),
            _ => None,
        }
    }

    fn parse_block(&mut self) -> Option<Vec<Node>> {
        let mut body = Vec::new();
        loop {
            match self.peek() {
                Token::RBrace | Token::EOF => break,
                Token::Newline => { self.advance(); }
                _ => {
                    if let Some(node) = self.parse_statement() {
                        body.push(node);
                    }
                }
            }
        }
        Some(body)
    }
}