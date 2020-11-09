use nom::{
    error::Error,
    IResult,
};
use nom::sequence::{delimited, preceded, tuple};
use nom::bytes::complete::{take_till1, take_while_m_n, tag};
use nom::character::complete::{char, digit1};
use nom::combinator::{map_opt, map_res, map, value, all_consuming};
use nom::error::{ParseError, FromExternalError};
use nom::branch::alt;
use crate::parser::Fragment::SingleField;
use std::num::ParseIntError;
use nom::error::ErrorKind::Digit;

#[derive(PartialEq, Debug)]
enum Fragment<'a> {
    // "a value" - a literal string
    Literal(&'a [u8]),
    // {2} - field 2
    SingleField(usize),
    // {2,4} - fields 2, 3, 4
    FieldRange(usize, usize),
    // {2,} - fields 2, 3, 4, ....
    UnboundedFieldRange(usize),
}


// TODO make this do the FromString thing from the docs: https://docs.rs/nom/6.0.0/nom/recipes/index.html#implementing-fromstr

#[derive(Debug)]
pub struct Template<'a> {
    raw_template: &'a str,
    fragments: Vec<Fragment<'a>>,
}

impl PartialEq for Template<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.raw_template == other.raw_template && self.fragments == other.fragments
    }
}

fn parse_literal(input: &str) -> nom::IResult<&str, &str> {
    // parse until we get to something escaped (a '\') or the start of a field (an unescaped '{')
    nom::bytes::complete::is_not("{\\")(input)
}

fn till_closing_bracket(s: &str) -> IResult<&str, &str> {
    take_till1(|c| c == '}')(s)
}

fn inside_brackets(input: &str) -> IResult<&str, &str> {
    delimited(char('{'), till_closing_bracket, char('}'))(input)
}


// fn parse_field(input: &str) -> nom::IResult<&str, Fragment> {
//     let (remaining, inside) = inside_brackets(input)?;
//
//     let sf  = parse_single_field(input)?;
//
//     let field_result = alt((
//         map(parse_single_field, |s: &str| SingleField(s.parse::<usize>().unwrap())),
//         // map(parse_escaped_char, StringFragment::EscapedChar),
//         // value(StringFragment::EscapedWS, parse_escaped_whitespace),
//     ))(input);
//     Ok((remaining, field_result.unwrap().1))
// }

fn parse_num(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |digit_str: &str| {
        digit_str.parse::<usize>()
    })(input)
}


fn parse_single_field(input: &str) -> nom::IResult<&str, Fragment> {
    let (remainder, field_number) = all_consuming(parse_num)(input)?;
    Ok((remainder, Fragment::SingleField(field_number)))
}

fn parse_field_range(input: &str) -> nom::IResult<&str, Fragment> {
    let (_, (start, _, end)) = all_consuming(tuple((
        parse_num,
        char(','),
        parse_num,
    )))(input)?;

    Ok(("", Fragment::FieldRange(start, end)))
}


// fn parse_unbounded_field_range(input: &str) -> nom::IResult<&str, Fragment::UnboundedFieldRange> {
// parse until we get to something escaped (a '\') or the start of a field (an unescaped '{')
// nom::bytes::complete::is_not("{\\")(i)
// TODO get this to parse out the various fields
// }

// fn parse_field_range(input: &str) -> nom::IResult<&str, Fragment::FieldRange> {
// parse until we get to something escaped (a '\') or the start of a field (an unescaped '{')
// nom::bytes::complete::is_not("{\\")(i)
// TODO get this to parse out the various fields
// }

// get a single fragment
// TODO see parse_fragment in string.rs example
// fn parse_fragment(input: &str) -> nom::IResult<&str, Fragment> {
//     Ok((input, Fragment::Literal(input.as_bytes())))
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_literal;
    use nom::error::ErrorKind::{IsNot, TakeTill1, Eof, Char};
    use nom::bytes::complete::take_while;
    use crate::parser::Fragment::FieldRange;

    #[test]
    fn test_parse_literal() {
        assert_eq!(
            parse_literal(""),
            Err(nom::Err::Error(Error::new("", IsNot)))
        );
        assert_eq!(parse_literal("abcd"), Ok(("", "abcd")));
        assert_eq!(parse_literal("the fox"), Ok(("", "the fox")));
        assert_eq!(parse_literal("the }} fox"), Ok(("", "the }} fox")));

// should stop when it hits the start of a possible token
        assert_eq!(parse_literal("the {} fox"), Ok(("{} fox", "the ")));

// should stop when it hits an escaped character
        assert_eq!(parse_literal("the \\n fox"), Ok(("\\n fox", "the ")));


// TODO start here parse an escaped character sequence

// assert_eq!(parse_literal("the \\{\\} fox"), Ok(("", "the {} fox")));
    }

    #[test]
    fn test_inside_brackets() {
        assert_eq!(inside_brackets("{}"), Err(nom::Err::Error(Error::new("}", TakeTill1))));
        assert_eq!(inside_brackets("{1}"), Ok(("", "1")));
        assert_eq!(inside_brackets("{100}"), Ok(("", "100")));
        assert_eq!(inside_brackets("{2,}"), Ok(("", "2,")));
        assert_eq!(inside_brackets("{200,}"), Ok(("", "200,")));
        assert_eq!(inside_brackets("{2,4}"), Ok(("", "2,4")));
        assert_eq!(inside_brackets("{200,400}"), Ok(("", "200,400")));

        // inside brackets doesn't only look for numbers, that's for another combinator to decide
        assert_eq!(inside_brackets("{foobar}"), Ok(("", "foobar")));
        assert_eq!(inside_brackets("{foo bar}"), Ok(("", "foo bar")));

        // it leaves the things alone after the brackets
        assert_eq!(inside_brackets("{1} after"), Ok((" after", "1")));
    }

    #[test]
    fn test_parse_single_field() {
        assert_eq!(parse_single_field("1"), Ok(("", SingleField(1))));
        assert_eq!(parse_single_field("100"), Ok(("", SingleField(100))));

        assert_eq!(parse_single_field("100,"), Err(nom::Err::Error(Error::new(",", Eof))));
        assert_eq!(parse_single_field("100,200"), Err(nom::Err::Error(Error::new(",200", Eof))));
    }

    #[test]
    fn test_parse_field_range() {
        assert_eq!(parse_field_range("1,2"), Ok(("", FieldRange(1, 2))));
        assert_eq!(parse_field_range("100,200"), Ok(("", FieldRange(100, 200))));

        assert_eq!(parse_field_range("100,"), Err(nom::Err::Error(Error::new("", Digit))));
        assert_eq!(parse_field_range("100"), Err(nom::Err::Error(Error::new("", Char))));
    }

    #[test]
    fn test_parse_field() {
        // extract different kinds of fields without static text
        // assert_eq!(parse_field("{1}"), Ok(("", SingleField(1))));
        // assert_eq!(parse_field("{100}"), Ok(("", SingleField(100))));
        // assert_eq!(parse_field("{2,}"), Ok(("", UnboundedFieldRange(2))));
        // assert_eq!(parse_field("{200,}"), Ok(("", UnboundedFieldRange(200))));
        // assert_eq!(parse_field("{2,4}"), Ok(("", FieldRange(2, 4))));
        // assert_eq!(parse_field("{200,400}"), Ok(("", FieldRange(200, 400))));

        // extract different kinds of fields followed by other text
        // assert_eq!(parse_field("{1} after"), Ok(("after", SingleField(1))));
        // assert_eq!(parse_field("{2} after"), Ok(("after", SingleField(2))));
        // assert_eq!(parse_field("{2,} after"), Ok(("after", UnboundedFieldRange(2))));
        // assert_eq!(parse_field("{2,4} after"), Ok(("after", FieldRange(2, 4))));


        // error if we don't close our field appropriately


        // should only allow numerics and commas (and ignore whitespace?) before the closing "}"?
    }

    #[test]
    fn test_extract_template() {
// combine the two
    }
}
