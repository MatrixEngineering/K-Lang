#![allow(dead_code)]
use std::{fs, io};

use crate::token::{Token, TokenErrorKind, TokenKind};

#[derive(Debug)]
pub struct Lexer {
    input: std::iter::Peekable<std::vec::IntoIter<char>>,
    line: u32,
}

#[derive(Debug)]
pub enum ParserError {
    IoError(std::io::Error),
    Other,
}

impl std::error::Error for ParserError {}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "Hello {:?}", e),
            _ => {
                todo!()
            }
        }
    }
}

impl From<io::Error> for ParserError {
    fn from(value: io::Error) -> Self {
        ParserError::IoError(value)
    }
}

impl Lexer {
    pub fn from_file(file_name: impl ToString) -> Result<Self, ParserError> {
        let input = fs::read_to_string(file_name.to_string())?;
        Ok(Self::new(input))
    }

    pub fn new(input: impl Into<String>) -> Self {
        let owned_string: String = input.into();
        let chars_vec: Vec<char> = owned_string.chars().collect();

        Lexer {
            input: chars_vec.into_iter().peekable(),
            line: 1,
        }
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.next()?;
        if ch == '\n' {
            self.line += 1;
        }
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.input.peek() {
            if ch.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let ch = self.advance()?;
        let start_line = self.line;
        let mut literal: Option<String> = None;
        let token_kind = match ch {
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '/' => {
                if self.input.next_if_eq(&'/').is_some() {
                    self.next_line();
                    return self.next_token();
                } else {
                    TokenKind::Slash
                }
            }
            '*' => TokenKind::Star,
            '>' => {
                if self.input.next_if_eq(&'=').is_some() {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            '<' => {
                if self.input.next_if_eq(&'=').is_some() {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }
            '!' => {
                if self.input.next_if_eq(&'=').is_some() {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                }
            }
            '-' => TokenKind::Minus,
            '+' => TokenKind::Plus,

            '=' => {
                if self.input.next_if_eq(&'=').is_some() {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                }
            }
            '"' => {
                let mut string = String::new();
                let mut found_closing_qoute = false;
                while let Some(c) = self.advance() {
                    if c == '"' {
                        found_closing_qoute = true;
                        break;
                    } else {
                        string.push(c);
                    }
                }
                if !found_closing_qoute {
                    TokenKind::Error(TokenErrorKind::UnterminatedString)
                } else {
                    literal = Some(string);
                    TokenKind::String
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut string: String = ch.to_string();
                while let Some(&next) = self.input.peek() {
                    if !next.is_whitespace() || next.is_ascii_digit() || next == '_' {
                        if next.is_ascii_punctuation() && next != '_' {
                            break;
                        }
                        string.push(next);
                        self.advance();
                    } else {
                        break;
                    }
                }

                literal = Some(string);
                TokenKind::Identifier
            }
            '0'..='9' => {
                let mut number = String::from(ch);
                while let Some(&c) = self.input.peek() {
                    if c.is_ascii_digit() {
                        number.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                let mut temp_input = self.input.clone();
                if temp_input.next_if_eq(&'.').is_some() {
                    if let Some(c) = temp_input.next()
                        && c.is_ascii_digit()
                    {
                        self.advance();
                    }
                    let mut next_number = String::new();
                    while let Some(&c) = self.input.peek() {
                        if c.is_ascii_digit() {
                            next_number.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    if !next_number.is_empty() {
                        number.push('.');
                        number.push_str(&next_number);
                    }
                }
                let num: f64 = number.parse().unwrap();
                TokenKind::Number(num)
            }
            c if is_khmer_digit(c) => {
                let khmer_to_number_char = |next| match next {
                    '០' => '0',
                    '១' => '1',
                    '២' => '2',
                    '៣' => '3',
                    '៤' => '4',
                    '៥' => '5',
                    '៦' => '6',
                    '៧' => '7',
                    '៨' => '8',
                    '៩' => '9',
                    '.' => '.',
                    _ => unreachable!(),
                };

                let mut string_digit: String = khmer_to_number_char(c).to_string();
                let mut dot_count = 0;
                let mut is_error = false;
                while let Some(&next) = self.input.peek() {
                    if is_khmer_char(next) || next == '.' {
                        if next == '.' {
                            dot_count += 1;
                        }
                        if dot_count > 1 {
                            is_error = true;
                        }
                        let next = khmer_to_number_char(next);
                        string_digit.push(next);
                        self.advance();
                    } else {
                        break;
                    }
                }

                if is_error {
                    TokenKind::Error(TokenErrorKind::Unexpected)
                } else {
                    let number: f64 = string_digit.parse().unwrap();
                    TokenKind::Number(number)
                }
            }
            c if is_khmer_char(c) => {
                let mut literal: String = c.to_string();
                while let Some(&next) = self.input.peek() {
                    if !next.is_whitespace() || next.is_ascii_digit() || next != '_' {
                        if next.is_ascii_punctuation() && next != '_' {
                            break;
                        }
                        literal.push(next);
                        self.advance();
                    } else {
                        break;
                    }
                }

                match literal.as_str() {
                    "តាង" => TokenKind::Var,
                    "បើ" => TokenKind::If,
                    "បើពុំនោះទេ" => TokenKind::Else,
                    "បោះពុម្ព" => TokenKind::Print,
                    "អនុគមន៍" => TokenKind::Fun,
                    "ពុម្ពគំរូ" => TokenKind::Class,
                    "គ្មានតម្លៃ" => TokenKind::Null,
                    "ឬ" => TokenKind::Or,
                    "និង" => TokenKind::And,
                    "ពិត" => TokenKind::True,
                    "មិនពិត" => TokenKind::False,
                    _ => {
                        todo!()
                    }
                }
            }
            _ => todo!(),
        };

        Some(Token {
            kind: token_kind,
            line: start_line,
            literal,
        })
    }

    fn next_line(&mut self) {
        while let Some(&c) = self.input.peek() {
            if c != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }
}

fn is_khmer_char(c: char) -> bool {
    matches!(c, '\u{1780}'..='\u{17FF}')
}

fn is_khmer_digit(c: char) -> bool {
    matches!(c, '\u{17E0}'..='\u{17E9}')
}

fn khmer_digit_to_int(c: char) -> Option<u32> {
    if is_khmer_digit(c) {
        Some((c as u32) - 0x17E0)
    } else {
        None
    }
}

impl Iterator for Lexer {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, token::TokenKind};

    #[test]
    fn keyword() {
        let buffer = "១.២ 1";
        let mut lexer = Lexer::new(buffer);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Number(1.2));
    }
}
