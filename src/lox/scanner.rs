use std::{collections::HashMap, process};

use crate::lox::tokens::{Token, TokenType};
use crate::lox::error::report_error;

pub struct Scanner {
    pub source: String,
    pub tokens: Vec<Token>,
}

struct EnumeratedString {
    pub soken: String,
    pub offset: usize,
}

/// State for the scanner. This is used to help identify any bad character.
/// `Next` is the starting state, it may take any character
/// `MaybeTwo` represents we've gotten !, =, > or <.
/// This means it will become a token after the next character is read.
/// `LiteralOrKeyword` represents we've started a possible literal or keyword.
/// This means it will concatenate all characters until we get (, ), {, }, Comma, ., -, +, ;, /, *, !, =, >, < or "end of word".
//
// digraph {
//     IdentifierOrKeyword -> Next [ label="new" ]
//     IdentifierOrKeyword -> IdentifierOrKeyword [ label="buf" ]
//     Next -> InString [ label="buf" ]
//     InString -> InString [ label="buf" ]
//     Next -> MaybeTwo [ label="buf" ]
//     Next -> Next [ label="new" ]
//     Next -> IdentifierOrKeyword [ label="buf" ]
//     Next -> Number [ label="buf" ]
//     Next -> SoloDot [ label="buf" ]
//     MaybeTwo -> Next [ label="new" ]
//     MaybeTwo -> Comment [ label="nothing" ]
//     InString -> Next [ label="new" ]
//     Number -> Next [ label="new" ]
// 	Number -> Number [ label="buf" ]
//     Number -> NumberWithDot [ label="buf" ]
//     NumberWithDot -> NumberWithDot [ label="buf" ]
//     NumberWithDot -> Next [ label="new" ]
//     SoloDot -> NumberWithDot [ label="buf" ]
//     SoloDot -> Next [ label="new" ]
//     Comment -> Next [ label="nothing" ]
//     IdentifierOrKeyword -> SoloDot [ label="new/buf" ]
//     IdentifierOrKeyword -> MaybeTwo [ label="new/buf" ]
// }
//                                   buf
//                ┌────────────────────────────────────────────────────┐
//                ▼                                                    │
//              ┌───────────────┐                                      │
// ┌─────────── │    SoloDot    │ ─────────────────────────┐           │
// │            └───────────────┘                          │           │
// │              ▲                                        │           │
// │              │ new/buf                                │           │
// │              │                                        │           │
// │      buf   ┌───────────────────────────────┐          │           │
// │    ┌────── │                               │          │           │
// │    │       │      IdentifierOrKeyword      │          │           │
// │    └─────▶ │                               │ ◀┐       │           │
// │            └───────────────────────────────┘  │       │           │
// │              │                │               │       │           │
// │              │ new/buf        │               │       │           │
// │              ▼                │               │       │           │
// │            ┌───────────────┐  │               │       │           │
// │    ┌────── │   MaybeTwo    │ ◀┼───────┐       │       │           │
// │    │       └───────────────┘  │       │       │       │           │
// │    │         │                │       │       │ buf   │           │
// │    │         │ nothing        │       │       │       │           │
// │    │         ▼                │       │       │       │           │
// │    │       ┌───────────────┐  │       │ buf   │       │           │
// │    │ new   │    Comment    │  │       │       │       │           │
// │    │       └───────────────┘  │       │       │       │           │
// │    │         │                │       │       │       │           │
// │    │         │ nothing        │ new   │       │       │ new       │
// │    │         ▼                ▼       │       │       ▼           │
// │    │       ┌─────────────────────────────────────────────────────────────┐   new
// │    │       │                                                             │ ──────┐
// │    │       │                            Next                             │       │
// │    └─────▶ │                                                             │ ◀─────┘
// │            └─────────────────────────────────────────────────────────────┘
// │              │                ▲       ▲       ▲       │
// │              │ buf            │ new   │ new   │ new   │ buf
// │              ▼                │       │       │       ▼
// │      buf   ┌───────────────┐  │       │       │     ┌──────────┐   buf
// │    ┌────── │               │  │       │       │     │          │ ──────┐
// │    │       │    Number     │  │       │       │     │ InString │       │
// │    └─────▶ │               │ ─┘       │       └──── │          │ ◀─────┘
// │            └───────────────┘          │             └──────────┘
// │              │                        │
// │              │ buf                    │
// │              ▼                        │
// │      buf   ┌───────────────┐          │
// │    ┌────── │               │          │
// │    │       │ NumberWithDot │          │
// │    └─────▶ │               │ ─────────┘
// │            └───────────────┘
// │   buf        ▲
// └──────────────┘
enum ScannerState {
    Comment,
    Next,
    MaybeTwo,
    IdentifierOrKeyword,
    InString,
    Number,
    NumberWithDot,
    SoloDot,
}

impl Scanner {
    pub fn new(code: String) -> Self {
        Scanner {
            source: code,
            tokens: Vec::new(),
        }
    }
    pub fn scan_tokens(&mut self) {
        let single_char: HashMap<char, TokenType> = HashMap::from([
            ('(', TokenType::LeftParen),
            (')', TokenType::RightParen),
            ('{', TokenType::LeftBrace),
            ('}', TokenType::RightBrace),
            (',', TokenType::Comma),
            ('.', TokenType::Dot),
            ('-', TokenType::Minus),
            ('+', TokenType::Plus),
            (';', TokenType::Semicolon),
            ('*', TokenType::Star),
        ]);

        let first_two_char: HashMap<char, TokenType> = HashMap::from([
            ('!', TokenType::Bang),
            ('=', TokenType::Equal),
            ('>', TokenType::Greater),
            ('<', TokenType::Less),
            ('/', TokenType::Slash),
        ]);

        let keywords: HashMap<String, TokenType> = HashMap::from([
            ("and".to_string(),     TokenType::And),
            ("class".to_string(),   TokenType::Class),
            ("else".to_string(),    TokenType::Else),
            ("false".to_string(),   TokenType::False),
            ("fun".to_string(),     TokenType::Fun),
            ("for".to_string(),     TokenType::For),
            ("if".to_string(),      TokenType::If),
            ("nil".to_string(),     TokenType::Nil),
            ("or".to_string(),      TokenType::Or),
            ("print".to_string(),   TokenType::Print),
            ("return".to_string(),  TokenType::Return),
            ("super".to_string(),   TokenType::Super),
            ("this".to_string(),    TokenType::This),
            ("true".to_string(),    TokenType::True),
            ("var".to_string(),     TokenType::Var),
            ("while".to_string(),   TokenType::While),
        ]);

        let mut state = ScannerState::Next;
        let mut buffer_vec: Vec<char> = Vec::with_capacity(128);
        let mut buffer_type = TokenType::Print;
        let mut line_count = 1usize;
        let mut since_last_line = 0usize;
        
        // Maybe good impl. Needs refac, but don't know how to bcs too much overhead (too many arguments)
        // and too many custom functions needed.
        for (i, c) in self.source.chars().enumerate() {
            match state {
                ScannerState::Comment => {
                    if c == '\n' {
                        line_count += 1;
                        since_last_line = i + 1;
                        state = ScannerState::Next;
                    }
                }
                ScannerState::SoloDot => {
                    if c.is_numeric() {
                        buffer_vec.clear();
                        buffer_vec.push('.');
                        buffer_vec.push(c);
                        state = ScannerState::NumberWithDot;
                        continue;
                    }
                    self.tokens.push(Token::new(TokenType::Dot, None, line_count, i - since_last_line - 1));
                    // same as ScannerState::Next without is_numeric() check
                    if c == '.' {
                        state = ScannerState::SoloDot;
                    } else if let Some(tt) = single_char.get(&c) {
                        self.tokens.push(Token::new(tt.clone(), None, line_count, i - since_last_line));
                    } else if let Some(tt) = first_two_char.get(&c) {
                        state = ScannerState::MaybeTwo;
                        buffer_type = tt.clone();
                    } else if c.is_whitespace() {
                        if c == '\n' {
                            line_count += 1;
                            since_last_line = i + 1;
                        }
                        state = ScannerState::Next;
                    } else if c == '"' {
                        state = ScannerState::InString;
                        buffer_vec.clear();
                        buffer_vec.push(c);
                    } else {
                        state = ScannerState::IdentifierOrKeyword;
                        buffer_vec.clear();
                        buffer_vec.push(c);
                    }
                    
                }
                ScannerState::NumberWithDot => {
                    if c == '.' {
                        println!("wtf");
                        report_error(line_count, i - since_last_line, &self.source, "Did not expect '.'".to_string());
                        process::exit(1);
                    }
                    if let Some(tt) = single_char.get(&c) {
                        let number = buffer_vec.iter().collect::<String>();
                        self.tokens.push(Token::new(TokenType::Number, Some(number), line_count, i - since_last_line - 1));
                        self.tokens.push(Token::new(tt.clone(), None, line_count, i - since_last_line));
                    } else if let Some(tt) = first_two_char.get(&c) {
                        let number = buffer_vec.iter().collect::<String>();
                        self.tokens.push(Token::new(TokenType::Number, Some(number), line_count, i - since_last_line - 1));
                        state = ScannerState::MaybeTwo;
                        buffer_type = tt.clone();
                    } else if c.is_whitespace() {
                        let number = buffer_vec.iter().collect::<String>();
                        self.tokens.push(Token::new(TokenType::Number, Some(number), line_count, i - since_last_line - 1));
                        if c == '\n' {
                            line_count += 1;
                            since_last_line = i + 1;
                        }
                        state = ScannerState::Next;
                    } else if c.is_numeric() {
                        buffer_vec.push(c);
                    }
                }
                ScannerState::Number => {
                    if c == '.' {
                        buffer_vec.push(c);
                        state = ScannerState::NumberWithDot;
                        continue;
                    }
                    if let Some(tt) = single_char.get(&c) {
                        let number = buffer_vec.iter().collect::<String>();
                        self.tokens.push(Token::new(TokenType::Number, Some(number), line_count, i - since_last_line - 1));
                        self.tokens.push(Token::new(tt.clone(), None, line_count, i - since_last_line));
                    } else if let Some(tt) = first_two_char.get(&c) {
                        let number = buffer_vec.iter().collect::<String>();
                        self.tokens.push(Token::new(TokenType::Number, Some(number), line_count, i - since_last_line - 1));
                        state = ScannerState::MaybeTwo;
                        buffer_type = tt.clone();
                    } else if c.is_whitespace() {
                        let number = buffer_vec.iter().collect::<String>();
                        self.tokens.push(Token::new(TokenType::Number, Some(number), line_count, i - since_last_line - 1));
                        if c == '\n' {
                            line_count += 1;
                            since_last_line = i + 1;
                        }
                        state = ScannerState::Next;
                    } else {
                        buffer_vec.push(c);
                    }
                }
                ScannerState::InString => {
                    buffer_vec.push(c);
                    if c == '"' {
                        let word = buffer_vec.iter().collect::<String>();
                        self.tokens.push(Token::new(TokenType::String, Some(word), line_count, i - since_last_line));
                        state = ScannerState::Next;
                    }
                }
                ScannerState::Next => {
                    if c == '.' {
                        state = ScannerState::SoloDot;
                    } else if let Some(tt) = single_char.get(&c) {
                        self.tokens.push(Token::new(tt.clone(), None, line_count, i - since_last_line));
                    } else if let Some(tt) = first_two_char.get(&c) {
                        state = ScannerState::MaybeTwo;
                        buffer_type = tt.clone();
                    } else if c.is_whitespace() {
                        if c == '\n' {
                            line_count += 1;
                            since_last_line = i + 1;
                        }
                    } else if c == '"' {
                        state = ScannerState::InString;
                        buffer_vec.clear();
                        buffer_vec.push(c);
                    } else if c.is_numeric() {
                        state = ScannerState::Number;
                        buffer_vec.clear();
                        buffer_vec.push(c);
                    } else {
                        state = ScannerState::IdentifierOrKeyword;
                        buffer_vec.clear();
                        buffer_vec.push(c);
                    }
                }
                ScannerState::MaybeTwo => {
                    if let Some(tt) = single_char.get(&c) {
                        self.tokens.push(Token::new(buffer_type, None, line_count, i - since_last_line - 1));
                        self.tokens.push(Token::new(tt.clone(), None, line_count, i - since_last_line));
                        state = ScannerState::Next;
                    } else if let Some(tt) = first_two_char.get(&c) {
                        if c == '=' && buffer_type != TokenType::Slash {
                            let tt = match buffer_type {
                                TokenType::Bang => TokenType::BangEqual,
                                TokenType::Equal => TokenType::EqualEqual,
                                TokenType::Greater => TokenType::GreaterEqual,
                                TokenType::Less => TokenType::LessEqual,
                                _ => {
                                    println!("Somehow a not possible two character token was considered as possible two character token");
                                    process::exit(1);
                                }
                            };
                            self.tokens.push(Token::new(tt, None, line_count, i - since_last_line));
                        } else if c == '/' && buffer_type == TokenType::Slash {
                            state = ScannerState::Comment;
                        } else {
                            self.tokens.push(Token::new(buffer_type, None, line_count, i - since_last_line - 1));
                            buffer_type = tt.clone();
                        }
                    } else if c.is_whitespace() {
                        self.tokens.push(Token::new(buffer_type, None, line_count, i - since_last_line - 1));
                        if c == '\n' {
                            line_count += 1;
                            since_last_line = i + 1;
                        }
                        state = ScannerState::Next;
                    } else if c == '"' { 
                        self.tokens.push(Token::new(buffer_type, None, line_count, i - since_last_line - 1));
                        state = ScannerState::InString;
                        buffer_vec.clear();
                        buffer_vec.push(c);
                    } else if c.is_numeric() {
                        self.tokens.push(Token::new(buffer_type, None, line_count, i - since_last_line - 1));
                        state = ScannerState::Number;
                        buffer_vec.clear();
                        buffer_vec.push(c);
                    } else {
                        self.tokens.push(Token::new(buffer_type, None, line_count, i - since_last_line - 1));
                        state = ScannerState::IdentifierOrKeyword;
                        buffer_vec.clear();
                        buffer_vec.push(c);
                    }
                }
                ScannerState::IdentifierOrKeyword => {
                    if let Some(tt) = single_char.get(&c) {
                        let word = buffer_vec.iter().collect::<String>();
                        if let Some(word_tt) = keywords.get(&word) {
                            self.tokens.push(Token::new(*word_tt, None, line_count, i - since_last_line - 1));
                        } else {
                            self.tokens.push(Token::new(TokenType::Identifier, Some(word), line_count, i - since_last_line - 1));
                        }
                        self.tokens.push(Token::new(tt.clone(), None, line_count, i - since_last_line));
                        state = ScannerState::Next;
                    } else if let Some(tt) = first_two_char.get(&c) {
                        let word = buffer_vec.iter().collect::<String>();
                        if let Some(word_tt) = keywords.get(&word) {
                            self.tokens.push(Token::new(*word_tt, None, line_count, i - since_last_line - 1));
                        } else {
                            self.tokens.push(Token::new(TokenType::Identifier, Some(word), line_count, i - since_last_line - 1));
                        }
                        state = ScannerState::MaybeTwo;
                        buffer_type = tt.clone();
                    } else if c.is_whitespace() {
                        let word = buffer_vec.iter().collect::<String>();
                        if let Some(word_tt) = keywords.get(&word) {
                            self.tokens.push(Token::new(*word_tt, None, line_count, i - since_last_line - 1));
                        } else {
                            self.tokens.push(Token::new(TokenType::Identifier, Some(word), line_count, i - since_last_line - 1));
                        }
                        state = ScannerState::Next;
                        if c == '\n' {
                            line_count += 1;
                            since_last_line = i + 1;
                        }
                    }  else if c == '"' { 
                        let word = buffer_vec.iter().collect::<String>();
                        if let Some(word_tt) = keywords.get(&word) {
                            self.tokens.push(Token::new(*word_tt, None, line_count, i - since_last_line - 1));
                        } else {
                            self.tokens.push(Token::new(TokenType::Identifier, Some(word), line_count, i - since_last_line - 1));
                        }
                        state = ScannerState::InString;
                        buffer_vec.clear();
                        buffer_vec.push(c);
                    } else {
                        buffer_vec.push(c);
                    }
                }
            }
        }

        let eof_token = Token::new(TokenType::Eof, None, line_count, 0);

        self.tokens.push(eof_token);
        println!("{:#?}", self.tokens);
    }
}
