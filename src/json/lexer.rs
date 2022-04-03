use std::fs;
use std::io::{Bytes, Read};

use crate::utf8;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum JsonTokenType {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    String,
    Number,
    Boolean,
    Null,
    EOF,
    None,
}

#[derive(Debug)]
pub struct Token(pub String, pub JsonTokenType);

impl Token {
    pub fn new(s: String, t: JsonTokenType) -> Self {
        Token(s, t)
    }
}

enum ContentType<'a> {
    File(Bytes<fs::File>),
    String(std::str::Bytes<'a>),
}

pub struct JsonLexer<'a> {
    file: ContentType<'a>,
    putback: char,
    has_putback: bool,
}

impl<'a> JsonLexer<'a> {
    pub fn new(filename: String) -> Self {
        JsonLexer {
            file: ContentType::File(fs::File::open(filename).unwrap().bytes()),
            putback: '\0',
            has_putback: false,
        }
    }

    pub fn new_from_string(s: &'a String) -> Self {
        JsonLexer {
            file: ContentType::<'a>::String(s.bytes()),
            putback: '\0',
            has_putback: false,
        }
    }

    fn next_char(&mut self) -> Result<char, &'static str> {
        if self.has_putback {
            self.has_putback = false;
            Ok(self.putback)
        } else {
            match self.file {
                ContentType::File(ref mut f) => match f.next() {
                    Some(Ok(c)) => {
                        match utf8::next_codepoint_head(f, c, |file| file.next().unwrap().unwrap())
                        {
                            Some(h) => Ok(h),
                            None => Err("Invalid UTF-8"),
                        }
                    }
                    Some(Err(_)) => Err("Error reading file"),
                    None => Ok('\0'),
                },
                ContentType::String(ref mut s) => match s.next() {
                    Some(c) => match utf8::next_codepoint_head(s, c, |str| str.next().unwrap()) {
                        Some(h) => Ok(h),
                        None => Err("Invalid UTF-8"),
                    },
                    None => Ok('\0'),
                },
            }
        }
    }

    fn try_match_char(&mut self, ch: char) -> bool {
        if self.has_putback {
            if self.putback == ch {
                self.has_putback = false;
                true
            } else {
                false
            }
        } else {
            let next = self.next_char();
            match next {
                Ok(ch2) => {
                    if ch == ch2 {
                        true
                    } else {
                        self.putback = ch2;
                        self.has_putback = true;
                        false
                    }
                }
                Err(_) => false,
            }
        }
    }

    fn putback(&mut self, ch: char) {
        if self.has_putback {
            panic!("putback called twice");
        }
        self.putback = ch;
        self.has_putback = true;
    }

    pub fn next_token(&mut self) -> Result<Token, &'static str> {
        let mut ch = self.next_char()?;
        // while ch.is_whitespace() {
        //     ch = self.next_char()?;
        // }
        match ch {
            '{' => Ok(Token(String::from("{"), JsonTokenType::LeftBrace)),
            '}' => Ok(Token(String::from("}"), JsonTokenType::RightBrace)),
            '[' => Ok(Token(String::from("["), JsonTokenType::LeftBracket)),
            ']' => Ok(Token(String::from("]"), JsonTokenType::RightBracket)),
            ',' => Ok(Token(String::from(","), JsonTokenType::Comma)),
            ':' => Ok(Token(String::from(":"), JsonTokenType::Colon)),
            '"' => {
                let mut s = String::new();
                loop {
                    ch = self.next_char()?;
                    if ch == '"' {
                        break;
                    }
                    s.push(ch);
                }
                Ok(Token(s, JsonTokenType::String))
            }
            '0'..='9' => {
                let mut s = String::from(ch.to_string());
                loop {
                    s.push(ch);
                    ch = self.next_char()?;
                    if !ch.is_ascii_digit() {
                        break;
                    }
                }
                self.putback(ch);
                Ok(Token(s, JsonTokenType::Number))
            }
            't' => {
                if self.try_match_char('r') && self.try_match_char('u') && self.try_match_char('e')
                {
                    Ok(Token(String::from("true"), JsonTokenType::Boolean))
                } else {
                    Err("expected true")
                }
            }
            'f' => {
                if self.try_match_char('a')
                    && self.try_match_char('l')
                    && self.try_match_char('s')
                    && self.try_match_char('e')
                {
                    Ok(Token(String::from("false"), JsonTokenType::Boolean))
                } else {
                    Err("expected false")
                }
            }
            'n' => {
                if self.try_match_char('u') && self.try_match_char('l') && self.try_match_char('l')
                {
                    Ok(Token(String::from("null"), JsonTokenType::Null))
                } else {
                    Err("expected null")
                }
            }
            '\0' => Ok(Token(String::from(""), JsonTokenType::EOF)),
            _ if ch.is_whitespace() => self.next_token(),
            _ => {
                if ch.is_ascii_punctuation() {
                    Err("unexpected punctuation")
                } else {
                    Err("unexpected character")
                }
            }
        }
    }
}
