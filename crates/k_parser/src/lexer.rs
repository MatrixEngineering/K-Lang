#![allow(dead_code)]
use std::{fs, io};

use k_ast::token::{Token, TokenErrorKind, TokenKind};

#[derive(Debug)]
pub struct Lexer {
    input: std::iter::Peekable<std::vec::IntoIter<char>>,
    line: u32,
}

#[derive(Debug)]
pub enum ParserError {
    IoError(std::io::Error),
    MSG(String),
    Other,
}

impl std::error::Error for ParserError {}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => e.fmt(f),
            Self::MSG(s) => s.fmt(f),
            _ => write!(f, "Unexpected Error"),
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
            ',' => TokenKind::Comma,
            '។' | ';' => TokenKind::Semi,
            '.' => TokenKind::Dot,
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
            '0'..='9' | '\u{17E0}'..='\u{17E9}' => {
                let get_ch = |next| match next {
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
                    c => c,
                };
                let mut number = String::from(get_ch(ch));
                while let Some(&c) = self.input.peek() {
                    if c.is_ascii_digit() || is_khmer_digit(c) {
                        number.push(get_ch(c));
                        self.advance();
                    } else {
                        break;
                    }
                }
                let mut temp_input = self.input.clone();
                if temp_input.next_if_eq(&'.').is_some() {
                    if let Some(c) = temp_input.next()
                        && (c.is_ascii_digit() || is_khmer_digit(c))
                    {
                        self.advance();
                    }
                    let mut next_number = String::new();
                    while let Some(&c) = self.input.peek() {
                        if c.is_ascii_digit() || is_khmer_digit(c) {
                            next_number.push(get_ch(c));
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

            c => {
                let mut string: String = c.to_string();
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

                match string.as_str() {
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
                    "សម្រាប់" => TokenKind::For,
                    "ខណៈ" => TokenKind::While,
                    "វត្ថុនេះ" => TokenKind::This,
                    "ថ្នាក់មេ" => TokenKind::Super,
                    "ត្រឡប់" => TokenKind::Return,
                    _ => {
                        literal = Some(string);
                        TokenKind::Identifier
                    }
                }
            }
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
    use crate::lexer::Lexer;
    use k_ast::token::{TokenErrorKind, TokenKind};

    fn kinds(input: &str) -> Vec<TokenKind> {
        Lexer::new(input).map(|t| t.kind).collect()
    }

    #[test]
    fn keyword() {
        let buffer = "១.២3១";
        let mut lexer = Lexer::new(buffer);
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Number(1.231));
    }

    #[test]
    fn empty_and_whitespace_input() {
        assert!(kinds("").is_empty());
        assert!(kinds("  \t\n\r\n ").is_empty());
    }

    #[test]
    fn single_char_tokens() {
        assert_eq!(
            kinds("(){}*/+-"),
            vec![
                TokenKind::LeftParen,
                TokenKind::RightParen,
                TokenKind::LeftBrace,
                TokenKind::RightBrace,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Plus,
                TokenKind::Minus,
            ]
        );
    }

    #[test]
    fn one_or_two_char_operators() {
        assert_eq!(
            kinds("! != = == < <= > >="),
            vec![
                TokenKind::Bang,
                TokenKind::BangEqual,
                TokenKind::Equal,
                TokenKind::EqualEqual,
                TokenKind::Less,
                TokenKind::LessEqual,
                TokenKind::Greater,
                TokenKind::GreaterEqual,
            ]
        );
    }

    #[test]
    fn adjacent_operators_without_spaces() {
        assert_eq!(
            kinds("===!"),
            vec![TokenKind::EqualEqual, TokenKind::Equal, TokenKind::Bang]
        );
    }

    #[test]
    fn comment_is_skipped() {
        assert_eq!(kinds("// hello\n+"), vec![TokenKind::Plus]);
        assert!(kinds("// only a comment").is_empty());
    }

    #[test]
    fn slash_is_not_comment() {
        assert_eq!(
            kinds("4 / 2"),
            vec![
                TokenKind::Number(4.0),
                TokenKind::Slash,
                TokenKind::Number(2.0)
            ]
        );
    }

    #[test]
    fn string_literal() {
        let token = Lexer::new("\"hello world\"").next_token().unwrap();
        assert_eq!(token.kind, TokenKind::String);
        assert_eq!(token.literal.as_deref(), Some("hello world"));
    }

    #[test]
    fn empty_string_literal() {
        let token = Lexer::new("\"\"").next_token().unwrap();
        assert_eq!(token.kind, TokenKind::String);
        assert_eq!(token.literal.as_deref(), Some(""));
    }

    #[test]
    fn khmer_string_literal() {
        let token = Lexer::new("\"សួស្តី\"").next_token().unwrap();
        assert_eq!(token.kind, TokenKind::String);
        assert_eq!(token.literal.as_deref(), Some("សួស្តី"));
    }

    #[test]
    fn unterminated_string() {
        let token = Lexer::new("\"abc").next_token().unwrap();
        assert_eq!(
            token.kind,
            TokenKind::Error(TokenErrorKind::UnterminatedString)
        );
        assert!(token.literal.is_none());
    }

    #[test]
    fn multiline_string_advances_line() {
        let mut lexer = Lexer::new("\"a\nb\" +");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::String);
        let plus = lexer.next_token().unwrap();
        assert_eq!(plus.kind, TokenKind::Plus);
        assert_eq!(plus.line, 2);
    }

    #[test]
    fn identifiers() {
        let tokens: Vec<_> = Lexer::new("foo _bar Baz9 a_b").collect();
        let literals: Vec<_> = tokens.iter().map(|t| t.literal.as_deref()).collect();
        assert!(tokens.iter().all(|t| t.kind == TokenKind::Identifier));
        assert_eq!(
            literals,
            vec![Some("foo"), Some("_bar"), Some("Baz9"), Some("a_b")]
        );
    }

    #[test]
    fn identifier_stops_at_punctuation() {
        let tokens: Vec<_> = Lexer::new("foo(bar)").collect();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].literal.as_deref(), Some("foo"));
        assert_eq!(tokens[1].kind, TokenKind::LeftParen);
        assert_eq!(tokens[2].literal.as_deref(), Some("bar"));
        assert_eq!(tokens[3].kind, TokenKind::RightParen);
    }

    #[test]
    fn integer_and_float_numbers() {
        assert_eq!(kinds("0"), vec![TokenKind::Number(0.0)]);
        assert_eq!(kinds("42"), vec![TokenKind::Number(42.0)]);
        assert_eq!(kinds("3.14"), vec![TokenKind::Number(3.14)]);
    }

    #[test]
    fn khmer_digits() {
        assert_eq!(kinds("០"), vec![TokenKind::Number(0.0)]);
        assert_eq!(kinds("១២៣"), vec![TokenKind::Number(123.0)]);
        assert_eq!(kinds("៩.៥"), vec![TokenKind::Number(9.5)]);
    }

    #[test]
    fn mixed_digit_scripts() {
        assert_eq!(kinds("1២3"), vec![TokenKind::Number(123.0)]);
    }

    #[test]
    fn number_followed_by_operator() {
        assert_eq!(
            kinds("1+2"),
            vec![
                TokenKind::Number(1.0),
                TokenKind::Plus,
                TokenKind::Number(2.0)
            ]
        );
    }

    #[test]
    fn khmer_keywords() {
        let cases = [
            ("តាង", TokenKind::Var),
            ("បើ", TokenKind::If),
            ("បើពុំនោះទេ", TokenKind::Else),
            ("បោះពុម្ព", TokenKind::Print),
            ("អនុគមន៍", TokenKind::Fun),
            ("ពុម្ពគំរូ", TokenKind::Class),
            ("គ្មានតម្លៃ", TokenKind::Null),
            ("ឬ", TokenKind::Or),
            ("និង", TokenKind::And),
            ("ពិត", TokenKind::True),
            ("មិនពិត", TokenKind::False),
            ("សម្រាប់", TokenKind::For),
            ("ខណៈ", TokenKind::While),
            ("ត្រឡប់", TokenKind::Return),
            ("វត្ថុនេះ", TokenKind::This),
            ("ថ្នាក់មេ", TokenKind::Super),
        ];
        for (src, expected) in cases {
            assert_eq!(kinds(src), vec![expected], "keyword {src}");
        }
    }

    #[test]
    fn khmer_keyword_before_punctuation() {
        assert_eq!(
            kinds("បើ(ពិត)"),
            vec![
                TokenKind::If,
                TokenKind::LeftParen,
                TokenKind::True,
                TokenKind::RightParen,
            ]
        );
    }

    #[test]
    fn variable_declaration_statement() {
        assert_eq!(
            kinds("តាង x = ១២ + 3"),
            vec![
                TokenKind::Var,
                TokenKind::Identifier,
                TokenKind::Equal,
                TokenKind::Number(12.0),
                TokenKind::Plus,
                TokenKind::Number(3.0),
            ]
        );
    }

    #[test]
    fn line_numbers_are_tracked() {
        let tokens: Vec<_> = Lexer::new("a\n\nb // c\n  d").collect();
        let lines: Vec<_> = tokens.iter().map(|t| t.line).collect();
        assert_eq!(lines, vec![1, 3, 4]);
    }

    #[test]
    fn dot_comma_semi_tokens() {
        assert_eq!(
            kinds(". , ;"),
            vec![TokenKind::Dot, TokenKind::Comma, TokenKind::Semi]
        );
    }

    #[test]
    fn call_with_arguments() {
        assert_eq!(
            kinds("f(a, b);"),
            vec![
                TokenKind::Identifier,
                TokenKind::LeftParen,
                TokenKind::Identifier,
                TokenKind::Comma,
                TokenKind::Identifier,
                TokenKind::RightParen,
                TokenKind::Semi,
            ]
        );
    }

    #[test]
    fn property_access() {
        assert_eq!(
            kinds("a.b"),
            vec![TokenKind::Identifier, TokenKind::Dot, TokenKind::Identifier]
        );
    }

    #[test]
    fn number_with_trailing_dot() {
        assert_eq!(kinds("1."), vec![TokenKind::Number(1.0), TokenKind::Dot]);
    }

    #[test]
    fn unknown_khmer_word_is_identifier() {
        let token = Lexer::new("សួស្តី").next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.literal.as_deref(), Some("សួស្តី"));
    }

    #[test]
    fn from_file_missing_returns_error() {
        assert!(Lexer::from_file("definitely/not/a/real/file.k").is_err());
    }
}
