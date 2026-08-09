//! Parser implementation for the Connect language.
//!
//! Builds an AST from a token stream produced by the lexer.

use crate::lexer::Token;

/// AST node variants for the language.
#[derive(Debug, Clone)]
pub enum Node {
    VarLDecl { name: String, value: Box<Node> },
    VarGDecl { name: String, value: Box<Node> },
    ConstDecl { name: String, value: Box<Node> },
    UpdateDecl { name: String, value: Box<Node> },
    MultiVarL { names: Vec<String>, values: Vec<Box<Node>> },
    UpdateIndex { name: String, index: Box<Node>, value: Box<Node> },
    Summon(String),
    Print(Box<Node>),
    InputExpr(Box<Node>),
    TypeOf(Box<Node>),
    ToInt(Box<Node>),
    ToFloat(Box<Node>),
    ToString(Box<Node>),
    ToBool(Box<Node>),
    Integer(i64),
    Float(f64),
    StringLit(String),
    Boolean(bool),
    Null,
    Array(Vec<Node>),
    IndexAccess { name: String, index: Box<Node> },
    MethodCall { object: String, method: String, args: Vec<Node> },
    MathCall { method: String, args: Vec<Node> },
    FileCall { method: String, args: Vec<Node> },
    BinaryOp { left: Box<Node>, op: String, right: Box<Node> },
    UnaryOp { op: String, operand: Box<Node> },
    Check {
        condition: Box<Node>,
        body: Vec<Node>,
        or_checks: Vec<(Node, Vec<Node>)>,
        else_body: Option<Vec<Node>>,
    },
    Circle { name: String, count: Box<Node>, body: Vec<Node> },
    Shatter,
    Skip,
    FuncDecl { name: String, func_type: String, params: Vec<String>, body: Vec<Node> },
    FuncCall { name: String, args: Vec<Node> },
    Reply(Box<Node>),
    Rest(Box<Node>),
    Wait(Box<Node>),
    Trigger { trigger_type: String, value: Box<Node> },
    Guard { condition: Box<Node>, body: Vec<Node> },
    UseModule(String),
}

/// Recursive descent parser state.
///
/// `tokens` is the input stream and `pos` tracks the current parse position.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Create a parser for a prepared token stream.
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Return the next non-newline token without consuming it.
    fn peek(&self) -> &Token {
        let mut i = self.pos;
        while i < self.tokens.len() {
            match &self.tokens[i] {
                Token::Newline => i += 1,
                _ => return &self.tokens[i],
            }
        }
        &Token::EOF
    }

    /// Return the second non-newline token ahead of the current position.
    fn peek_ahead(&self) -> &Token {
        let mut i = self.pos;
        let mut count = 0;
        while i < self.tokens.len() {
            match &self.tokens[i] {
                Token::Newline => i += 1,
                _ => {
                    if count == 1 {
                        return &self.tokens[i];
                    }
                    count += 1;
                    i += 1;
                }
            }
        }
        &Token::EOF
    }

    /// Consume and return the next non-newline token.
    fn advance(&mut self) -> &Token {
        while self.pos < self.tokens.len() {
            match &self.tokens[self.pos] {
                Token::Newline => self.pos += 1,
                _ => break,
            }
        }
        let token = &self.tokens[self.pos];
        self.pos += 1;
        token
    }

    /// Ensure the next token matches `expected`, consuming it if so.
    fn expect(&mut self, expected: &Token) -> bool {
        if self.peek() == expected {
            self.advance();
            true
        } else {
            println!("[PARSE ERROR] Expected {:?} but got {:?}", expected, self.peek());
            println!("Your syntax is so wrong it physically hurt me to read.");
            false
        }
    }

    pub fn parse(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            match self.peek() {
                Token::EOF => break,
                Token::Newline => { self.advance(); }
                _ => {
                    if let Some(node) = self.parse_statement() {
                        nodes.push(node);
                    }
                }
            }
        }
        nodes
    }

    /// Parse a top-level statement or expression statement.
    fn parse_statement(&mut self) -> Option<Node> {
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
            Token::Shatter => { self.advance(); Some(Node::Shatter) }
            Token::Skip    => { self.advance(); Some(Node::Skip) }
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
            Token::Math => {
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
                Some(Node::MathCall { method, args })
            }
            Token::File => {
                self.advance();
                self.expect(&Token::Dot);
                let method = match self.advance().clone() {
                    Token::Ident(m) => m,
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
                Some(Node::FileCall { method, args })
            }
            _ => {
                println!("[PARSE ERROR] Unexpected token: {:?}", self.peek());
                self.advance();
                None
            }
        }
    }

    /// Parse a module import statement.
    fn parse_use(&mut self) -> Option<Node> {
        self.advance();
        match self.advance().clone() {
            Token::StringLit(path) => Some(Node::UseModule(path)),
            _ => {
                println!("[PARSE ERROR] Expected file path after 'use'.");
                None
            }
        }
    }

    /// Parse a method call on a variable or object.
    fn parse_method_call_statement(&mut self) -> Option<Node> {
        let name = match self.advance().clone() {
            Token::Ident(n) => n,
            _ => return None,
        };
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
        if !self.expect(&Token::Equals) {
            return None;
        }
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
        if !self.expect(&Token::Equals) {
            return None;
        }
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
            return Some(Node::UpdateIndex {
                name,
                index: Box::new(index),
                value: Box::new(value),
            });
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
        Some(Node::Circle { name, count: Box::new(count), body })
    }

    fn parse_check(&mut self) -> Option<Node> {
        self.advance();
        self.expect(&Token::LParen);
        let condition = self.parse_expression()?;
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

        let mut or_checks = Vec::new();
        while self.peek() == &Token::OrCheck {
            self.advance();
            self.expect(&Token::LParen);
            let or_condition = self.parse_expression()?;
            self.expect(&Token::RParen);
            self.expect(&Token::LBrace);
            let mut or_body = Vec::new();
            loop {
                match self.peek() {
                    Token::RBrace | Token::EOF => break,
                    Token::Newline => { self.advance(); }
                    _ => {
                        if let Some(node) = self.parse_statement() {
                            or_body.push(node);
                        }
                    }
                }
            }
            self.expect(&Token::RBrace);
            or_checks.push((or_condition, or_body));
        }

        let else_body = if self.peek() == &Token::Else {
            self.advance();
            self.expect(&Token::LBrace);
            let mut else_nodes = Vec::new();
            loop {
                match self.peek() {
                    Token::RBrace | Token::EOF => break,
                    Token::Newline => { self.advance(); }
                    _ => {
                        if let Some(node) = self.parse_statement() {
                            else_nodes.push(node);
                        }
                    }
                }
            }
            self.expect(&Token::RBrace);
            Some(else_nodes)
        } else { None };

        Some(Node::Check {
            condition: Box::new(condition),
            body,
            or_checks,
            else_body,
        })
    }

    fn parse_func(&mut self) -> Option<Node> {
        self.advance();
        self.expect(&Token::LBracket);
        let func_type = match self.advance().clone() {
            Token::Callable => "callable".to_string(),
            Token::Auto     => "auto".to_string(),
            Token::OneTime  => "onetime".to_string(),
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
        self.expect(&Token::LParen);
        let trigger_type = match self.advance().clone() {
            Token::When  => "when".to_string(),
            Token::Every => "every".to_string(),
            _ => return None,
        };
        self.expect(&Token::Colon);
        let value = self.parse_expression()?;
        self.expect(&Token::RParen);
        Some(Node::Trigger { trigger_type, value: Box::new(value) })
    }

    fn parse_guard(&mut self) -> Option<Node> {
        self.advance();
        self.expect(&Token::LParen);
        let condition = self.parse_expression()?;
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
        Some(Node::Guard { condition: Box::new(condition), body })
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

    /// Parse an expression with the lowest precedence operator first.
    fn parse_expression(&mut self) -> Option<Node> {
        self.parse_or()
    }

    /// Parse boolean OR expressions.
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
                Token::EqualEqual   => "EqualEqual",
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
            return Some(Node::UnaryOp {
                op: "Not".to_string(),
                operand: Box::new(operand),
            });
        }
        if self.peek() == &Token::Minus {
            self.advance();
            let operand = self.parse_unary()?;
            return Some(Node::UnaryOp {
                op: "Negate".to_string(),
                operand: Box::new(operand),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Option<Node> {
        match self.advance().clone() {
            Token::Integer(n)   => Some(Node::Integer(n)),
            Token::Float(f)     => Some(Node::Float(f)),
            Token::StringLit(s) => Some(Node::StringLit(s)),
            Token::True         => Some(Node::Boolean(true)),
            Token::False        => Some(Node::Boolean(false)),
            Token::Null         => Some(Node::Null),

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

            Token::Math => {
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
                Some(Node::MathCall { method, args })
            }

            Token::File => {
                self.expect(&Token::Dot);
                let method = match self.advance().clone() {
                    Token::Ident(m) => m,
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
                Some(Node::FileCall { method, args })
            }

            Token::Ident(name) => {
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
                }
                else if self.peek() == &Token::LBracket {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(&Token::RBracket);
                    Some(Node::IndexAccess { name, index: Box::new(index) })
                }
                else if self.peek() == &Token::Dot {
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
                }
                else {
                    Some(Node::Summon(name))
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
}