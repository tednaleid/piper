use crate::parser::Fragment::{EscapedChar, FieldRange, SingleField};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till1, take_while_m_n};
use nom::character::complete::{anychar, char, digit1};
use nom::combinator::{all_consuming, map, map_opt, map_res, value};
use nom::error::ErrorKind::Digit;
use nom::error::{FromExternalError, ParseError};
use nom::sequence::{delimited, preceded, tuple};
use nom::{error::Error, IResult};
use std::num::ParseIntError;

#[derive(PartialEq, Debug)]
enum Fragment<'a> {
    // "a value" - a literal string
    Literal(&'a [u8]),
    // a backslash escaped character, ex: '\{' -> {   or '\\' -> \
    EscapedChar(char),
    // {2} - field 2
    SingleField(usize),
    // {2,4} - fields 2, 3, 4
    FieldRange(usize, usize),
    // {2,} - fields 2, 3, 4, 5, ....
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
    // parse until we get to the start of a field (an unescaped '{') or something escaped (a '\')
    nom::bytes::complete::is_not("{\\")(input)
}

fn parse_escaped_char(input: &str) -> IResult<&str, char> {
    preceded(char('\\'), anychar)(input)
}

fn till_closing_bracket(s: &str) -> IResult<&str, &str> {
    take_till1(|c| c == '}')(s)
}

fn inside_brackets(input: &str) -> IResult<&str, &str> {
    delimited(char('{'), till_closing_bracket, char('}'))(input)
}

/// parses a field enclosed in curly brackets into a Fragment, one of:
/// - SingleField: "{1}" -> SingleField(1)
/// - FieldRange: "{1,4}" -> FieldRange(1,4)
/// - UnboundedFieldRange: "{3,}" -> UnboundedFieldRange(3)
fn parse_field(input: &str) -> nom::IResult<&str, Fragment> {
    let (remaining, inside) = inside_brackets(input)?;

    let field_result = alt((
        parse_single_field,
        parse_field_range,
        parse_unbounded_field_range,
    ))(inside)?;

    Ok((remaining, field_result.1))
}

// fn parse_fragment(input: &str) -> nom::IResult<&str, Fragment> {
// // TODO parse field plus literal and escaped character
// }

fn parse_num(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |digit_str: &str| digit_str.parse::<usize>())(input)
}

fn parse_single_field(input: &str) -> nom::IResult<&str, Fragment> {
    let (remainder, field_number) = all_consuming(parse_num)(input)?;
    Ok((remainder, Fragment::SingleField(field_number)))
}

fn parse_field_range(input: &str) -> nom::IResult<&str, Fragment> {
    let (_, (start, _, end)) = all_consuming(tuple((parse_num, char(','), parse_num)))(input)?;

    Ok(("", Fragment::FieldRange(start, end)))
}

fn parse_unbounded_field_range(input: &str) -> nom::IResult<&str, Fragment> {
    let (_, (start, _)) = all_consuming(tuple((parse_num, char(','))))(input)?;

    Ok(("", Fragment::UnboundedFieldRange(start)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_literal;
    use crate::parser::Fragment::{FieldRange, Literal, UnboundedFieldRange};
    use nom::bytes::complete::take_while;
    use nom::error::ErrorKind::{Char, Eof, IsNot, TakeTill1};

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

        assert_eq!(parse_literal("the \\{\\} fox"), Ok(("\\{\\} fox", "the ")));
    }

    #[test]
    fn test_parse_escaped_character() {
        assert_eq!(parse_escaped_char("\\\\ fox"), Ok((" fox", '\\')));
        assert_eq!(parse_escaped_char("\\{ fox"), Ok((" fox", '{')));
        assert_eq!(parse_escaped_char("\\} fox"), Ok((" fox", '}')));
        assert_eq!(parse_escaped_char("\\n fox"), Ok((" fox", 'n')));
        assert_eq!(
            parse_escaped_char("no slash at start \\ fox"),
            Err(nom::Err::Error(Error::new(
                "no slash at start \\ fox",
                Char
            )))
        );
    }

    #[test]
    fn test_inside_brackets() {
        assert_eq!(
            inside_brackets("{}"),
            Err(nom::Err::Error(Error::new("}", TakeTill1)))
        );
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

        assert_eq!(
            parse_single_field("100,"),
            Err(nom::Err::Error(Error::new(",", Eof)))
        );
        assert_eq!(
            parse_single_field("100,200"),
            Err(nom::Err::Error(Error::new(",200", Eof)))
        );
    }

    #[test]
    fn test_parse_field_range() {
        assert_eq!(parse_field_range("1,2"), Ok(("", FieldRange(1, 2))));
        assert_eq!(parse_field_range("100,200"), Ok(("", FieldRange(100, 200))));

        assert_eq!(
            parse_field_range("100,"),
            Err(nom::Err::Error(Error::new("", Digit)))
        );
        assert_eq!(
            parse_field_range("100"),
            Err(nom::Err::Error(Error::new("", Char)))
        );
    }

    #[test]
    fn test_parse_unbounded_field_range() {
        assert_eq!(
            parse_unbounded_field_range("1,"),
            Ok(("", UnboundedFieldRange(1)))
        );
        assert_eq!(
            parse_unbounded_field_range("100,"),
            Ok(("", UnboundedFieldRange(100)))
        );

        assert_eq!(
            parse_unbounded_field_range("100,200"),
            Err(nom::Err::Error(Error::new("200", Eof)))
        );
        assert_eq!(
            parse_unbounded_field_range("100"),
            Err(nom::Err::Error(Error::new("", Char)))
        );
    }

    #[test]
    fn test_parse_field() {
        // extract different kinds of fields without static text
        assert_eq!(parse_field("{1}"), Ok(("", SingleField(1))));
        assert_eq!(parse_field("{100}"), Ok(("", SingleField(100))));
        assert_eq!(parse_field("{2,}"), Ok(("", UnboundedFieldRange(2))));
        assert_eq!(parse_field("{200,}"), Ok(("", UnboundedFieldRange(200))));
        assert_eq!(parse_field("{2,4}"), Ok(("", FieldRange(2, 4))));
        assert_eq!(parse_field("{200,400}"), Ok(("", FieldRange(200, 400))));

        // extract different kinds of fields followed by other text
        assert_eq!(parse_field("{1} after"), Ok((" after", SingleField(1))));
        assert_eq!(parse_field("{2} after"), Ok((" after", SingleField(2))));
        assert_eq!(
            parse_field("{2,} after"),
            Ok((" after", UnboundedFieldRange(2)))
        );
        assert_eq!(parse_field("{2,4} after"), Ok((" after", FieldRange(2, 4))));

        // error if we don't close our field appropriately
        assert_eq!(
            parse_field("{1"),
            Err(nom::Err::Error(Error::new("", Char)))
        );

        // error if we have some invalid text inside
        assert_eq!(
            parse_field("{f}"),
            Err(nom::Err::Error(Error::new("f", Digit)))
        );
    }

    #[test]
    fn test_parse_fragment() {
        // TODO test above plus getting literal and escaped character out
        // TODO next make these tests pass
        // assert_eq!(parse_fragment("before {1} after"), Ok(("{1} after", Literal("before ".as_bytes()))));
        // assert_eq!(parse_fragment("before \\{1\\} after"), Ok(("\\{1\\} after", Literal("before ".as_bytes()))));
        //
        // assert_eq!(parse_fragment("\\after"), Ok(("after", EscapedChar('\\'))));
        //
        // assert_eq!(parse_fragment("{1} after"), Ok((" after", SingleField(1))));
        // assert_eq!(parse_fragment("{2,} after"), Ok((" after", UnboundedFieldRange(2))));
        // assert_eq!(parse_fragment("{2,4} after"), Ok((" after", FieldRange(2, 4))));
        //
        // // TODO error conditions? this might not be the right error
        // assert_eq!(parse_fragment("{2,4\\} after"), Err(nom::Err::Error(Error::new("\\", Digit))));
    }

    #[test]
    fn test_extract_template() {
        // combine the two
    }
}
