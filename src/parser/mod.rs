pub mod ast;
pub mod expressions;
pub mod statements;

pub use ast::Node;
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn peek(&self) -> &Token {
        let mut i = self.pos;
        while i < self.tokens.len() {
            match &self.tokens[i] {
                Token::Newline => i += 1,
                _ => return &self.tokens[i],
            }
        }
        &Token::EOF
    }

    pub fn peek_ahead(&self) -> &Token {
        let mut i = self.pos;
        let mut count = 0;
        while i < self.tokens.len() {
            match &self.tokens[i] {
                Token::Newline => i += 1,
                _ => {
                    if count == 1 { return &self.tokens[i]; }
                    count += 1;
                    i += 1;
                }
            }
        }
        &Token::EOF
    }

    pub fn advance(&mut self) -> &Token {
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

    pub fn expect(&mut self, expected: &Token) -> bool {
        if self.peek() == expected {
            self.advance();
            true
        } else {
            println!("[PARSE ERROR] Expected {:?} but got {:?}", expected, self.peek());
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
}