#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: u32,
    pub literal: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Star,
    Dot,
    Comma,
    Plus,
    Minus,
    Semi,
    Slash,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    String,
    Number(f64),
    Identifier,
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Null,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Error(TokenErrorKind),
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TokenErrorKind {
    UnterminatedString,
    Unexpected,
}
