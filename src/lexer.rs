//! Lexer implementation for the Connect language.
//!
//! Converts raw source text into a stream of token values.

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    VarL,
    VarG,
    Summon,
    Update,
    Print,
    Check,
    OrCheck,
    Else,
    Circle,
    Shatter,
    Skip,
    Func,
    Callable,
    Auto,
    OneTime,
    Forever,
    Reply,
    Trigger,
    Rest,
    Guard,
    Wait,
    True,
    False,
    Use,
    When,
    Every,
    And,
    Or,
    Not,
    Const,
    Input,
    Type,
    ToInt,
    ToFloat,
    ToString,
    ToBool,
    Math,
    File,

    // Literals
    Integer(i64),
    Float(f64),
    StringLit(String),
    Null,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    DoubleStar,
    Equals,
    TripleEquals,
    EqualEqual,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Pipe,
    Dot,

    // Special
    Newline,
    EOF,

    // Identifier
    Ident(String),

    //Error Handling
    Attempt,
    Rescue,
    Always,
}

/// Lexer state for scanning source text.
///
/// `source` is stored as chars, `pos` tracks the current reading index,
/// and `line` is used for diagnostics.
pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied();
        self.pos += 1;
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        while self.pos < self.source.len() {
            if self.peek() == Some('*') && self.peek_next() == Some('/') {
                self.advance();
                self.advance();
                break;
            }
            if self.peek() == Some('\n') {
                self.line += 1;
            }
            self.advance();
        }
    }

    fn read_number(&mut self) -> Token {
        let mut num_str = String::new();
        let mut is_float = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                is_float = true;
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            Token::Float(num_str.parse::<f64>().unwrap_or(0.0))
        } else {
            Token::Integer(num_str.parse::<i64>().unwrap_or(0))
        }
    }

    fn read_string(&mut self) -> Token {
        self.advance();
        let mut result = String::new();

        while let Some(ch) = self.peek() {
            if ch == '\'' {
                self.advance();
                break;
            }
            if ch == '\\' {
                self.advance();
                if let Some(next) = self.peek() {
                    match next {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '\\' => result.push('\\'),
                        '\'' => result.push('\''),
                        other => {
                            result.push('\\');
                            result.push(other);
                        }
                    }
                    self.advance();
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }

        Token::StringLit(result)
    }

    fn read_template_string(&mut self) -> Token {
        self.advance();
        let mut result = String::new();

        while let Some(ch) = self.peek() {
            if ch == '`' {
                self.advance();
                break;
            }
            if ch == '\\' {
                self.advance();
                if let Some(next) = self.peek() {
                    match next {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '\\' => result.push('\\'),
                        '`' => result.push('`'),
                        other => {
                            result.push('\\');
                            result.push(other);
                        }
                    }
                    self.advance();
                }
            } else {
                if ch == '\n' {
                    self.line += 1;
                }
                result.push(ch);
                self.advance();
            }
        }

        Token::StringLit(result)
    }

    fn read_ident(&mut self) -> Token {
        let mut ident = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        match ident.as_str() {
            "varL"     => Token::VarL,
            "varG"     => Token::VarG,
            "summon"   => Token::Summon,
            "update"   => Token::Update,
            "print"    => Token::Print,
            "check"    => Token::Check,
            "orCheck"  => Token::OrCheck,
            "else"     => Token::Else,
            "circle"   => Token::Circle,
            "shatter"  => Token::Shatter,
            "skip"     => Token::Skip,
            "func"     => Token::Func,
            "callable" => Token::Callable,
            "auto"     => Token::Auto,
            "onetime"  => Token::OneTime,
            "forever"  => Token::Forever,
            "reply"    => Token::Reply,
            "trigger"  => Token::Trigger,
            "rest"     => Token::Rest,
            "guard"    => Token::Guard,
            "wait"     => Token::Wait,
            "true"     => Token::True,
            "false"    => Token::False,
            "use"      => Token::Use,
            "when"     => Token::When,
            "every"    => Token::Every,
            "and"      => Token::And,
            "or"       => Token::Or,
            "not"      => Token::Not,
            "const"    => Token::Const,
            "input"    => Token::Input,
            "type"     => Token::Type,
            "toInt"    => Token::ToInt,
            "toFloat"  => Token::ToFloat,
            "toString" => Token::ToString,
            "toBool"   => Token::ToBool,
            "math"     => Token::Math,
            "file"     => Token::File,
            "attempt"  => Token::Attempt,
            "rescue"   => Token::Rescue,
            "always"   => Token::Always,
            _          => Token::Ident(ident),
        }
    }

    /// Return the next token from the source, skipping whitespace and comments.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.peek() {
            None => Token::EOF,

            Some('\n') => {
                self.advance();
                self.line += 1;
                Token::Newline
            }

            Some('/') if self.peek_next() == Some('/') => {
                self.skip_comment();
                self.next_token()
            }

            Some('/') if self.peek_next() == Some('*') => {
                self.advance();
                self.advance();
                self.skip_block_comment();
                self.next_token()
            }

            Some(ch) if ch.is_ascii_digit() => self.read_number(),

            Some('\'') => self.read_string(),

            Some('`') => self.read_template_string(),

            Some(ch) if ch.is_alphabetic() || ch == '_' => self.read_ident(),

            Some('|') => {
                self.advance();
                self.skip_whitespace();
                if self.peek() == Some('|') {
                    self.advance();
                    Token::Null
                } else {
                    Token::Pipe
                }
            }

            Some('+') => { self.advance(); Token::Plus }
            Some('-') => { self.advance(); Token::Minus }
            Some('%') => { self.advance(); Token::Percent }
            Some(',') => { self.advance(); Token::Comma }
            Some(':') => { self.advance(); Token::Colon }
            Some('(') => { self.advance(); Token::LParen }
            Some(')') => { self.advance(); Token::RParen }
            Some('{') => { self.advance(); Token::LBrace }
            Some('}') => { self.advance(); Token::RBrace }
            Some('[') => { self.advance(); Token::LBracket }
            Some(']') => { self.advance(); Token::RBracket }
            Some('.') => { self.advance(); Token::Dot }

            Some('*') => {
                self.advance();
                if self.peek() == Some('*') {
                    self.advance();
                    Token::DoubleStar
                } else {
                    Token::Star
                }
            }

            Some('/') => { self.advance(); Token::Slash }

            Some('=') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::TripleEquals
                    } else {
                        Token::EqualEqual
                    }
                } else {
                    Token::Equals
                }
            }

            Some('!') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::NotEqual
                } else {
                    self.next_token()
                }
            }

            Some('>') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }

            Some('<') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::LessEqual
                } else {
                    Token::Less
                }
            }

            Some(ch) => {
                println!("[LEXER WARNING] Unknown character '{}' on line {}", ch, self.line);
                self.advance();
                self.next_token()
            }
        }
    }

    /// Scan the entire source into a token vector.
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();
            let is_eof = token == Token::EOF;
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        tokens
    }
}