pub(crate) trait Parser<'a, Output> {
    fn parse(&self, input: &'a str) -> Result<(&'a str, Output), &'a str>;

    fn or<P2>(self, parser: P2) -> impl Parser<'a, Output>
    where
        Self: Sized,
        P2: Parser<'a, Output>,
    {
        move |input| match self.parse(input) {
            result @ Ok(_) => result,
            Err(_) => parser.parse(input),
        }
    }
}

impl<'a, F, Output> Parser<'a, Output> for F
where
    F: Fn(&'a str) -> Result<(&'a str, Output), &'a str>,
{
    fn parse(&self, input: &'a str) -> Result<(&'a str, Output), &'a str> {
        self(input)
    }
}
