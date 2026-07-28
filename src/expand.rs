use std::collections::HashMap;

use crate::token::{Token, TokenPart};

impl Token {
    pub(crate) fn expand(&self, vars: &HashMap<String, String>) -> String {
        let mut result = String::with_capacity(self.parts.iter().map(|s| s.content().len()).sum());

        for part in &self.parts {
            match part {
                TokenPart::Unquoted(s) => {
                    let Some((prefix, remaining)) = s.split_once("$") else {
                        result.push_str(s);
                        continue;
                    };

                    result.push_str(prefix);
                    for part in remaining.split("$") {
                        let mut var_name = part;
                        let mut end_index = part.len();
                        if part.starts_with("{") {
                            end_index = part.find('}').unwrap_or(part.len());
                            var_name = &part[1..end_index];
                            end_index += 1;
                        }

                        if let Some(value) = vars.get(var_name) {
                            result.push_str(value);
                        }

                        if end_index < part.len() {
                            result.push_str(&part[end_index..]);
                        }
                    }
                }
                TokenPart::SingleQuoted(s) | TokenPart::DoubleQuoted(s) => {
                    result.push_str(s);
                }
            }
        }

        result
    }
}
