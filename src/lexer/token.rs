#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Language words are represented separately so the parser can dispatch on
    // syntax without comparing identifier strings.
    VarL, VarG, Summon, Update, Print,
    Check, OrCheck, Else,
    Circle, Shatter, Skip,
    Func, Callable, Auto, Forever,
    Reply, Trigger, Rest, Guard, Wait,
    True, False, Use, When, Every,
    And, Or, Not, Const, Input, Type,
    ToInt, ToFloat, ToString, ToBool,
    Math, File, Db, Rewind,

    // Domain-specific syntax introduced by the language extensions.
    Judge,
    Paint,
    FatArrow,
    Bond,
    As,
    Vbp,

    Vault,
    Weave,

    // Literal values are converted from source text by the lexer.
    Integer(i64),
    Float(f64),
    StringLit(String),
    Null,

    // Operators retain their own token so precedence is handled by the parser.
    Plus, Minus, Star, Slash, Percent, DoubleStar,
    Equals, TripleEquals, EqualEqual, NotEqual,
    Greater, Less, GreaterEqual, LessEqual,

    // Structural punctuation used by expressions and blocks.
    LParen, RParen, LBrace, RBrace,
    LBracket, RBracket, Comma, Colon, Pipe, Dot,

    // Newline is retained for parser navigation, while EOF marks input exhaustion.
    Newline, EOF,

    // Identifier
    Ident(String),

    // Error Handling
    Attempt, Rescue, Always,

    // Backend
    Json, Date, System, Http, Crypto, Async, Protect,
}