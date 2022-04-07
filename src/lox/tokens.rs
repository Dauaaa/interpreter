#[derive(Debug, Clone)]
pub struct Token {
    ttype: TokenType,
    literal: Option<String>,
    line: usize,
    offset: usize,
}

impl Token {
    pub fn new(ttype: TokenType, literal: Option<String>, line: usize, offset: usize) -> Self {
        Self {
            ttype,
            literal,
            line,
            offset,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Slash,
    // SlashSlash (no need though since we ignore comments [for now?])

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}
