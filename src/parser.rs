use crate::parser::{core::Parser, tokens::tokens};

mod core;
mod primitives;
mod tokens;

#[derive(Debug)]
pub enum TokenPart {
    Unquoted(String),
    SingleQuoted(String),
    DoubleQuoted(String),
}

impl TokenPart {
    pub fn content(&self) -> &str {
        match self {
            TokenPart::Unquoted(s) => s,
            TokenPart::SingleQuoted(s) => s,
            TokenPart::DoubleQuoted(s) => s,
        }
    }
}

#[derive(Debug)]
pub struct Token {
    pub parts: Vec<TokenPart>,
}

impl Token {
    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(self.parts.iter().map(|s| s.content().len()).sum());

        for part in &self.parts {
            result.push_str(part.content());
        }

        result
    }

    pub fn is_empty(&self) -> bool {
        self.parts.iter().all(|part| match part {
            TokenPart::Unquoted(s) => s.is_empty(),
            TokenPart::SingleQuoted(s) => s.is_empty(),
            TokenPart::DoubleQuoted(s) => s.is_empty(),
        })
    }
}

pub fn tokenize(input: &str) -> Result<(&str, Vec<Token>), &str> {
    let parser = tokens();

    parser.parse(input)
}
