use crate::parser::primitives::{
    left, map, match_double_quote, match_single_quote, match_until_char, match_until_pred,
    match_until_pred_with_escape, one_or_more, whitespace, zero_or_more,
};
use crate::parser::{Token, TokenPart};

use super::core::Parser;
use super::primitives::right;

pub fn tokens<'a>() -> impl Parser<'a, Vec<Token>> {
    one_or_more(token())
}

fn token<'a>() -> impl Parser<'a, Token> {
    right(
        zero_or_more(whitespace()),
        map(
            one_or_more(
                single_quote_part()
                    .or(double_quote_part())
                    .or(unquoted_token_part()),
            ),
            |parts| Token { parts },
        ),
    )
}

fn unquoted_token_part<'a>() -> impl Parser<'a, TokenPart> {
    move |input| {
        let parser =
            match_until_pred_with_escape(|c| c.is_whitespace() || c == '"' || c == '\'', |_| true);

        match parser.parse(input) {
            Ok((next_input, result)) if !result.is_empty() => {
                Ok((next_input, TokenPart::Unquoted(result)))
            }
            Ok(_) => Err(input),
            Err(err) => Err(err),
        }
    }
}

fn single_quote_part<'a>() -> impl Parser<'a, TokenPart> {
    map(
        right(
            match_single_quote(),
            left(match_until_char('\''), match_single_quote()),
        ),
        TokenPart::SingleQuoted,
    )
}

fn double_quote_part<'a>() -> impl Parser<'a, TokenPart> {
    map(
        right(
            match_double_quote(),
            left(match_until_char('"'), match_double_quote()),
        ),
        TokenPart::DoubleQuoted,
    )
}
