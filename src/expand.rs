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
                    for var_name in remaining.split("$") {
                        if let Some(value) = vars.get(var_name) {
                            result.push_str(value);
                        } else {
                            result.push_str(var_name);
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
