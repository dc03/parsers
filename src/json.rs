use std::collections::HashMap;

use self::lexer::JsonLexer;

mod lexer;

pub type JsonStringType = String;
pub type JsonNumberType = f32;
pub type JsonObjectType = HashMap<String, JsonValue>;
pub type JsonArrayType = Vec<JsonValue>;
pub type JsonBooleanType = bool;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    String(JsonStringType),
    Number(JsonNumberType),
    Object(JsonObjectType),
    Array(JsonArrayType),
    Boolean(JsonBooleanType),
    Nil,
}

pub struct JsonParser<'a> {
    lexer: JsonLexer<'a>,
    current: lexer::Token,
    next: lexer::Token,
}

impl<'a> JsonParser<'a> {
    pub fn new(filename: String) -> Self {
        JsonParser {
            lexer: JsonLexer::new(filename),
            current: lexer::Token(String::new(), lexer::JsonTokenType::None),
            next: lexer::Token(String::new(), lexer::JsonTokenType::None),
        }
    }

    pub fn new_from_string(s: &'a String) -> Self {
        JsonParser {
            lexer: JsonLexer::new_from_string(s),
            current: lexer::Token(String::new(), lexer::JsonTokenType::None),
            next: lexer::Token(String::new(), lexer::JsonTokenType::None),
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        match self.peek().1 {
            lexer::JsonTokenType::String => self.parse_string(),
            lexer::JsonTokenType::Number => self.parse_number(),
            lexer::JsonTokenType::LeftBrace => {
                self.advance()?;
                self.parse_object()
            }
            lexer::JsonTokenType::LeftBracket => {
                self.advance()?;
                self.parse_array()
            }
            lexer::JsonTokenType::Boolean => self.parse_boolean(),
            lexer::JsonTokenType::Null => Ok(JsonValue::Nil),
            _ => Err("Unexpected input".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        let mut obj = JsonObjectType::new();

        if self.peek().1 == lexer::JsonTokenType::RightBrace {
            self.advance()?;
            return Ok(JsonValue::Object(obj));
        }

        loop {
            let key = self.parse_string()?;
            self.consume(
                lexer::JsonTokenType::Colon,
                "Expected ':' after object key".to_string(),
            )?;

            let value = self.parse_value()?;
            if let JsonValue::String(key) = key {
                obj.insert(key, value);
            }

            if self.try_match(lexer::JsonTokenType::Comma) {
                continue;
            } else if self.try_match(lexer::JsonTokenType::RightBrace) {
                break;
            } else {
                return Err("Expected ',' or '}'".to_string());
            }
        }

        Ok(JsonValue::Object(obj))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        let mut arr = JsonArrayType::new();

        if self.peek().1 == lexer::JsonTokenType::RightBracket {
            self.advance()?;
            return Ok(JsonValue::Array(arr));
        }

        loop {
            let value = self.parse_value()?;
            arr.push(value);

            if self.try_match(lexer::JsonTokenType::Comma) {
                continue;
            } else if self.try_match(lexer::JsonTokenType::RightBracket) {
                break;
            } else {
                return Err("Expected ',' or ']'".to_string());
            }
        }

        Ok(JsonValue::Array(arr))
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        Ok(JsonValue::String(
            self.consume(lexer::JsonTokenType::String, "Expected string".to_string())?
                .0
                .clone(),
        ))
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        Ok(JsonValue::Number(
            self.consume(lexer::JsonTokenType::Number, "Expected number".to_string())?
                .0
                .parse()
                .unwrap(),
        ))
    }

    fn parse_boolean(&mut self) -> Result<JsonValue, String> {
        Ok(JsonValue::Boolean(
            self.consume(
                lexer::JsonTokenType::Boolean,
                "Expected boolean".to_string(),
            )?
            .0
            .parse()
            .unwrap(),
        ))
    }

    fn peek(&self) -> &lexer::Token {
        &self.next
    }

    fn advance(&mut self) -> Result<&lexer::Token, String> {
        if self.current.1 == lexer::JsonTokenType::EOF {
            Err("unexpected EOF".to_string())
        } else {
            self.current = lexer::Token::new(self.next.0.clone(), self.next.1);
            self.next = self.lexer.next_token().unwrap();
            Ok(&self.current)
        }
    }

    fn try_match(&mut self, token_type: lexer::JsonTokenType) -> bool {
        if self.next.1 == token_type {
            self.advance().unwrap();
            true
        } else {
            false
        }
    }

    fn consume(
        &mut self,
        expected: lexer::JsonTokenType,
        msg: String,
    ) -> Result<&lexer::Token, String> {
        if self.next.1 == expected {
            self.advance()
        } else {
            Err(msg)
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        self.advance()?;
        self.consume(
            lexer::JsonTokenType::LeftBrace,
            "Expected '{' at start of object".to_string(),
        )?;

        self.parse_object()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_string() {
        let string = "{\"foo\":\"bar\",\"baz\":\"qux\"}".to_string();
        let mut parser = super::JsonParser::new_from_string(&string);
        let result = parser.parse();
        assert!(result.is_ok());
        let result = result.unwrap();
        println!("{:?}", result);
    }

    #[test]
    fn test_file() {
        let file = "src/test.json".to_string();
        let mut parser = super::JsonParser::new(file);
        let result = parser.parse();
        assert!(result.is_ok());
        let result = result.unwrap();
        println!("{:?}", result);
    }
}
