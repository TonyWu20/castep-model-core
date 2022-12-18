use nalgebra::Point3;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, space1},
    combinator::opt,
    multi::separated_list1,
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};

use crate::parser::{decimal, float};

pub fn parse_acl(input: &str) -> IResult<&str, (u8, &str)> {
    let (rest, (num, symbol)) = preceded(
        tuple((tag("C"), space1, tag("ACL"), space1)),
        delimited(
            char('"'),
            separated_pair(decimal, space1, alpha1),
            char('"'),
        ),
    )(input)?;
    Ok((rest, (num.parse::<u8>().unwrap(), symbol)))
}

pub fn parse_label(input: &str) -> IResult<&str, Option<&str>> {
    opt(preceded(
        tuple((tag("C"), space1, tag("Label"), space1)),
        delimited(char('"'), alphanumeric1, char('"')),
    ))(input)
}

pub fn parse_xyz(input: &str) -> IResult<&str, Point3<f64>> {
    let (rest, xyz_str) = preceded(
        tuple((tag("D"), space1, tag("XYZ"), space1)),
        delimited(
            tag("("),
            separated_list1(space1, alt((float, decimal))),
            tag(")"),
        ),
    )(input)?;
    let xyz_vec: Vec<f64> = xyz_str
        .iter()
        .map(|num| num.parse::<f64>().unwrap())
        .collect();
    Ok((rest, Point3::from_slice(&xyz_vec)))
}

pub fn parse_id(input: &str) -> IResult<&str, u32> {
    let (rest, id_str) = preceded(tuple((tag("I"), space1, tag("Id"), space1)), decimal)(input)?;
    Ok((rest, id_str.parse::<u32>().unwrap()))
}
