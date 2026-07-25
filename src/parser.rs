mod core;
mod primitives;
mod tokenizer;

pub use tokenizer::tokenize;

use crate::{
    command::{ExecutionMode, Redirect, ShellCommand},
    token::{Token, TokenPart},
};

pub fn parse_command(tokens: &[Token]) -> Result<ShellCommand, String> {
    let mut cmd_name = String::new();
    let mut args = vec![];
    let mut redirect: Option<Redirect> = None;
    let mut has_parsed_cmd_name = false;

    let exec_mode = match tokens.last() {
        Some(token) => match token {
            Token { parts } if parts.len() == 1 => match parts[0] {
                TokenPart::Unquoted(ref s) if s == "&" => ExecutionMode::Background,
                _ => ExecutionMode::Foreground,
            },
            _ => ExecutionMode::Foreground,
        },
        _ => ExecutionMode::Foreground,
    };

    let exec_mode_offset = match exec_mode {
        ExecutionMode::Background => 1,
        ExecutionMode::Foreground => 0,
    };

    let mut tokens_iter = tokens[..tokens.len() - exec_mode_offset].iter();

    while let Some(token) = tokens_iter.next() {
        let content = token.to_string();
        if is_redirect_token(&content) {
            let file_name = match tokens_iter.next() {
                Some(token) => token.to_string(),
                None => {
                    return Err(String::from(
                        "Error parsing command: trying to redirect stdout to an empty file",
                    ));
                }
            };

            redirect = match content.as_str() {
                ">" | "1>" => Some(Redirect::Stdout(file_name)),
                "2>" => Some(Redirect::Stderr(file_name)),
                ">>" | "1>>" => Some(Redirect::AppendStdout(file_name)),
                "2>>" => Some(Redirect::AppendStderr(file_name)),
                _ => None,
            };
            continue;
        }

        if !has_parsed_cmd_name {
            has_parsed_cmd_name = true;
            cmd_name = content;
            continue;
        }

        args.push(content);
    }

    let command = ShellCommand {
        name: cmd_name,
        args,
        redirect,
        exec_mode,
    };

    Ok(command)
}

fn is_redirect_token(token: &str) -> bool {
    matches!(token, ">" | "1>" | "2>" | ">>" | "1>>" | "2>>")
}
