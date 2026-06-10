use super::core::Parser;

pub(crate) fn match_literal<'a>(expected: &'a str) -> impl Parser<'a, ()> {
    move |input: &'a str| match input.starts_with(expected) {
        true => Ok((&input[expected.len()..], ())),
        false => Err(input),
    }
}

pub(crate) fn match_single_quote<'a>() -> impl Parser<'a, ()> {
    match_literal("'")
}

pub(crate) fn match_double_quote<'a>() -> impl Parser<'a, ()> {
    match_literal("\"")
}

pub(crate) fn whitespace<'a>() -> impl Parser<'a, ()> {
    move |input: &'a str| match input.chars().next() {
        Some(c) if c.is_whitespace() => Ok((&input[c.len_utf8()..], ())),
        _ => Err(input),
    }
}

pub(crate) fn pair<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, (R1, R2)>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    move |input| {
        parser1.parse(input).and_then(|(next_input, result1)| {
            parser2
                .parse(next_input)
                .map(|(last_input, result2)| (last_input, (result1, result2)))
        })
    }
}

pub(crate) fn map<'a, P, F, A, B>(parser: P, map_fn: F) -> impl Parser<'a, B>
where
    P: Parser<'a, A>,
    F: Fn(A) -> B,
{
    move |input| {
        parser
            .parse(input)
            .map(|(next_input, result)| (next_input, map_fn(result)))
    }
}

pub(crate) fn left<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R1>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    map(pair(parser1, parser2), |(left, _right)| left)
}

pub(crate) fn right<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R2>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    map(pair(parser1, parser2), |(_left, right)| right)
}

pub(crate) fn match_until_char<'a>(ch: char) -> impl Parser<'a, String> {
    match_until_pred(move |c| c == ch)
}

pub(crate) fn match_until_pred<'a, F>(pred: F) -> impl Parser<'a, String>
where
    F: Fn(char) -> bool,
{
    move |input: &'a str| {
        if input.is_empty() {
            return Err(input);
        }

        let mut chars = input.chars();
        let mut matched = String::new();

        while let Some(next) = chars.next() {
            if pred(next) {
                break;
            }

            matched.push(next);
        }

        let next_index = matched.len();
        Ok((&input[next_index..], matched))
    }
}

pub(crate) fn match_until_pred_with_escape<'a, F1, F2>(
    pred: F1,
    should_escape: F2,
) -> impl Parser<'a, String>
where
    F1: Fn(char) -> bool,
    F2: Fn(char) -> bool,
{
    move |input: &'a str| {
        if input.is_empty() {
            return Err(input);
        }

        let mut chars = input.chars();
        let mut matched = String::new();

        while let Some(next) = chars.next() {
            if next == '\\' {
                match chars.next() {
                    Some(escaped) if should_escape(escaped) => {
                        matched.push(escaped);
                    }
                    Some(escaped) => {
                        matched.push(next);
                        matched.push(escaped);
                    }
                    None => {
                        matched.push(next);
                    }
                }
                continue;
            }

            if pred(next) {
                let next_index = input.len() - chars.as_str().len() - 1;
                return Ok((&input[next_index..], matched));
            }

            matched.push(next);
        }
        let next_index = input.len() - chars.as_str().len();
        Ok((&input[next_index..], matched))
    }
}

pub(crate) fn zero_or_more<'a, P, R>(parser: P) -> impl Parser<'a, Vec<R>>
where
    P: Parser<'a, R>,
{
    move |mut input| {
        let mut result = vec![];
        while let Ok((next_input, next_result)) = parser.parse(input) {
            result.push(next_result);
            input = next_input;
        }

        Ok((input, result))
    }
}

pub(crate) fn one_or_more<'a, P, R>(parser: P) -> impl Parser<'a, Vec<R>>
where
    P: Parser<'a, R>,
{
    move |mut input| {
        let mut result = vec![];

        let Ok((next_input, next_result)) = parser.parse(input) else {
            return Err(input);
        };

        result.push(next_result);
        input = next_input;

        while let Ok((next_input, next_result)) = parser.parse(input) {
            result.push(next_result);
            input = next_input;
        }

        Ok((input, result))
    }
}
