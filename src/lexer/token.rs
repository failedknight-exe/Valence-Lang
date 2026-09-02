#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    VarL, VarG, Summon, Update, Print,
    Check, OrCheck, Else,
    Circle, Shatter, Skip,
    Func, Callable, Auto, Forever,
    Reply, Trigger, Rest, Guard, Wait,
    True, False, Use, When, Every,
    And, Or, Not, Const, Input, Type,
    ToInt, ToFloat, ToString, ToBool,
    Math, File,

    // Literals
    Integer(i64),
    Float(f64),
    StringLit(String),
    Null,

    // Operators
    Plus, Minus, Star, Slash, Percent, DoubleStar,
    Equals, TripleEquals, EqualEqual, NotEqual,
    Greater, Less, GreaterEqual, LessEqual,

    // Delimiters
    LParen, RParen, LBrace, RBrace,
    LBracket, RBracket, Comma, Colon, Pipe, Dot,

    // Special
    Newline, EOF,

    // Identifier
    Ident(String),

    // Error Handling
    Attempt, Rescue, Always,

    // Backend
    Json, Date, System, Http, Crypto, Async, Protect,
}