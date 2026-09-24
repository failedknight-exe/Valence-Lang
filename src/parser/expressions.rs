//! Recursive-descent expression parsing ordered from lowest to highest precedence.

use super::Parser;
use super::ast::Node;
use crate::lexer::Token;

impl Parser {
    pub fn parse_expression(&mut self) -> Option<Node> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Option<Node> {
        let mut left = self.parse_and()?;
        while self.peek() == &Token::Or {
            self.advance();
            let right = self.parse_and()?;
            left = Node::BinaryOp {
                left: Box::new(left),
                op: "Or".to_string(),
                right: Box::new(right),
            };
        }
        Some(left)
    }

    fn parse_and(&mut self) -> Option<Node> {
        let mut left = self.parse_comparison()?;
        while self.peek() == &Token::And {
            self.advance();
            let right = self.parse_comparison()?;
            left = Node::BinaryOp {
                left: Box::new(left),
                op: "And".to_string(),
                right: Box::new(right),
            };
        }
        Some(left)
    }

    fn parse_comparison(&mut self) -> Option<Node> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Token::EqualEqual | Token::TripleEquals => "EqualEqual",
                Token::NotEqual     => "NotEqual",
                Token::Greater      => "Greater",
                Token::Less         => "Less",
                Token::GreaterEqual => "GreaterEqual",
                Token::LessEqual    => "LessEqual",
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Node::BinaryOp {
                left: Box::new(left),
                op: op.to_string(),
                right: Box::new(right),
            };
        }
        Some(left)
    }

    fn parse_additive(&mut self) -> Option<Node> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Token::Plus  => "Plus",
                Token::Minus => "Minus",
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Node::BinaryOp {
                left: Box::new(left),
                op: op.to_string(),
                right: Box::new(right),
            };
        }
        Some(left)
    }

    fn parse_multiplicative(&mut self) -> Option<Node> {
        let mut left = self.parse_power()?;
        loop {
            let op = match self.peek() {
                Token::Star    => "Star",
                Token::Slash   => "Slash",
                Token::Percent => "Percent",
                _ => break,
            };
            self.advance();
            let right = self.parse_power()?;
            left = Node::BinaryOp {
                left: Box::new(left),
                op: op.to_string(),
                right: Box::new(right),
            };
        }
        Some(left)
    }

    fn parse_power(&mut self) -> Option<Node> {
        let mut left = self.parse_unary()?;
        while self.peek() == &Token::DoubleStar {
            self.advance();
            let right = self.parse_unary()?;
            left = Node::BinaryOp {
                left: Box::new(left),
                op: "DoubleStar".to_string(),
                right: Box::new(right),
            };
        }
        Some(left)
    }

    fn parse_unary(&mut self) -> Option<Node> {
        if self.peek() == &Token::Not {
            self.advance();
            let operand = self.parse_unary()?;
            return Some(Node::UnaryOp { op: "Not".to_string(), operand: Box::new(operand) });
        }
        if self.peek() == &Token::Minus {
            self.advance();
            let operand = self.parse_unary()?;
            return Some(Node::UnaryOp { op: "Negate".to_string(), operand: Box::new(operand) });
        }
        self.parse_primary()
    }

    pub fn parse_primary(&mut self) -> Option<Node> {
        match self.advance().clone() {
            Token::Integer(n)   => Some(Node::Integer(n)),
            Token::Float(f)     => Some(Node::Float(f)),
            Token::StringLit(s) => Some(Node::StringLit(s)),
            Token::True         => Some(Node::Boolean(true)),
            Token::False        => Some(Node::Boolean(false)),
            Token::Null         => Some(Node::Null),

            Token::LBrace => {
                let mut keys = Vec::new();
                let mut values = Vec::new();
                if self.peek() == &Token::RBrace {
                    self.advance();
                    return Some(Node::MapLit { keys, values });
                }
                loop {
                    let key = match self.advance().clone() {
                        Token::Ident(k) => k,
                        other => {
                            println!("[PARSE ERROR] Expected map key, got {:?}", other);
                            return None;
                        }
                    };
                    keys.push(key);
                    self.expect(&Token::Colon);
                    let val = self.parse_expression()?;
                    values.push(Box::new(val));
                    if self.peek() == &Token::Comma { self.advance(); } else { break; }
                }
                self.expect(&Token::RBrace);
                Some(Node::MapLit { keys, values })
            }

            Token::LBracket => {
                let mut elements = Vec::new();
                if self.peek() != &Token::RBracket {
                    elements.push(self.parse_expression()?);
                    while self.peek() == &Token::Comma {
                        self.advance();
                        elements.push(self.parse_expression()?);
                    }
                }
                self.expect(&Token::RBracket);
                Some(Node::Array(elements))
            }

            Token::Summon => {
                match self.advance().clone() {
                    Token::Ident(name) => Some(Node::Summon(name)),
                    _ => None,
                }
            }

            Token::Input => {
                self.expect(&Token::LParen);
                let prompt = self.parse_expression()?;
                self.expect(&Token::RParen);
                Some(Node::InputExpr(Box::new(prompt)))
            }

            Token::Type => {
                self.expect(&Token::LParen);
                let expr = self.parse_expression()?;
                self.expect(&Token::RParen);
                Some(Node::TypeOf(Box::new(expr)))
            }

            Token::ToInt => {
                self.expect(&Token::LParen);
                let expr = self.parse_expression()?;
                self.expect(&Token::RParen);
                Some(Node::ToInt(Box::new(expr)))
            }

            Token::ToFloat => {
                self.expect(&Token::LParen);
                let expr = self.parse_expression()?;
                self.expect(&Token::RParen);
                Some(Node::ToFloat(Box::new(expr)))
            }

            Token::ToString => {
                self.expect(&Token::LParen);
                let expr = self.parse_expression()?;
                self.expect(&Token::RParen);
                Some(Node::ToString(Box::new(expr)))
            }

            Token::ToBool => {
                self.expect(&Token::LParen);
                let expr = self.parse_expression()?;
                self.expect(&Token::RParen);
                Some(Node::ToBool(Box::new(expr)))
            }

            Token::Math => self.parse_builtin_expr("math"),
            Token::File => self.parse_builtin_expr("file"),
            Token::Json => self.parse_builtin_expr("json"),
            Token::Date => self.parse_builtin_expr("date"),
            Token::System => self.parse_builtin_expr("system"),
            Token::Http => self.parse_builtin_expr("http"),
            Token::Crypto => self.parse_builtin_expr("crypto"),
            Token::Db => self.parse_builtin_expr("db"),
            Token::Paint => self.parse_builtin_expr("paint"),
            Token::Vbp => self.parse_builtin_expr("vbp"),
            Token::Vault => self.parse_builtin_expr("vault"),

            Token::Weave => {
        self.expect(&Token::LParen);
        let count = self.parse_expression()?;
        self.expect(&Token::RParen);
        self.expect(&Token::LBrace);
        
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
        self.expect(&Token::RBrace);
        Some(Node::Weave {
            count: Box::new(count),
            body,
        })
    }

            Token::Ident(name) => {
                if name == "mesh" && self.peek() == &Token::LBracket {
                    self.advance(); // [
                    let mode = match self.advance().clone() {
                        Token::Ident(m) => m,
                        _ => return None,
                    };
                    self.expect(&Token::RBracket);
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
                    return Some(Node::MeshCall {
                        mode,
                        method,
                        args,
                    });
                }

                if self.peek() == &Token::LParen {
                    self.advance();
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
                } else if self.peek() == &Token::LBracket {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(&Token::RBracket);
                    Some(Node::IndexAccess { name, index: Box::new(index) })
                } else if self.peek() == &Token::Dot {
                    self.advance();
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
                } else {
                    Some(Node::VarAccess(name))
                }
            }

            Token::LParen => {
                let expr = self.parse_expression()?;
                self.expect(&Token::RParen);
                Some(expr)
            }

            token => {
                println!("[PARSE ERROR] Unexpected token in expression: {:?}", token);
                None
            }
        }
    }

    fn parse_builtin_expr(&mut self, kind: &str) -> Option<Node> {
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
            "paint"  => Some(Node::PaintCall { method, args }),
            "vbp"    => Some(Node::VbpCall { method, args}),
            "vault"  => Some(Node::VaultCall { method, args }),
            _ => None,
        }
    }
}