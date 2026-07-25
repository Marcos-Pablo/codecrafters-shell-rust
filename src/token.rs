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
}

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
