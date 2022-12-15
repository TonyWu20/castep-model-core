use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, space1},
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};

use crate::parser::decimal;

fn parse_acl(input: &str) -> IResult<&str, (u8, &str)> {
    let (rest, (num, symbol)) = preceded(
        tuple((tag("A"), space1, tag("C"), space1, tag("ACL"), space1)),
        delimited(
            char('"'),
            separated_pair(decimal, space1, alpha1),
            char('"'),
        ),
    )(input)?;
    Ok((rest, (num.parse::<u8>().unwrap(), symbol)))
}

fn parse_label(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((tag("A"), space1, tag("C"), space1, tag("Label"), space1)),
        delimited(char('"'), alphanumeric1, char('"')),
    )(input)
}
