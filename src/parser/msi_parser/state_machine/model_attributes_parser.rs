use std::collections::HashMap;

use nalgebra::Vector3;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, space0, space1},
    combinator::recognize,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::parser::{decimal, float};

use super::{Analyzed, MsiParser};

pub fn hashmap_attrs<'a>(attributes: &[&'a str]) -> HashMap<String, &'a str> {
    let mut hashmap: HashMap<String, &str> = HashMap::new();
    attributes.iter().for_each(|item| {
        let (attr_str, key) = MsiParser::<Analyzed>::get_attribute_type(item).unwrap();
        hashmap.insert(key.into(), attr_str);
    });
    hashmap
}

pub fn parse_cry_display(input: &str) -> IResult<&str, (u32, u32)> {
    let (rest, value_str) = preceded(
        tuple((tag("I"), space1, tag("CRY/DISPLAY"), space1)),
        delimited(
            char('('),
            separated_pair(decimal, space1, decimal),
            char(')'),
        ),
    )(input)?;
    let value = (
        value_str.0.parse::<u32>().unwrap(),
        value_str.1.parse::<u32>().unwrap(),
    );
    Ok((rest, value))
}

pub fn parse_periodic_type(input: &str) -> IResult<&str, u8> {
    let (rest, value_str) = preceded(
        tuple((tag("I"), space1, tag("PeriodicType"), space1)),
        decimal,
    )(input)?;
    Ok((rest, value_str.parse::<u8>().unwrap()))
}

pub fn parse_space_group(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((tag("C"), space1, tag("SpaceGroup"), space1)),
        delimited(
            char('"'),
            recognize(separated_pair(decimal, space1, decimal)),
            char('"'),
        ),
    )(input)
}

pub fn parse_cry_tolerance(input: &str) -> IResult<&str, f64> {
    let (rest, value_str) = preceded(
        tuple((tag("D"), space1, tag("CRY/TOLERANCE"), space1)),
        float,
    )(input)?;
    Ok((rest, value_str.parse::<f64>().unwrap()))
}

pub fn parse_vector(input: &str) -> IResult<&str, Vector3<f64>> {
    let (rest, vec_str): (&str, (&str, &str, &str)) = preceded(
        tuple((
            tag("D"),
            space1,
            alt((tag("A3"), tag("B3"), tag("C3"))),
            space1,
        )),
        delimited(
            char('('),
            tuple((
                terminated(alt((float, decimal)), space0),
                terminated(alt((float, decimal)), space0),
                alt((float, decimal)),
            )),
            char(')'),
        ),
    )(input)?;
    let (x, y, z) = vec_str;
    let (x, y, z) = (
        x.parse::<f64>().unwrap(),
        y.parse::<f64>().unwrap(),
        z.parse::<f64>().unwrap(),
    );
    let vec: [f64; 3] = [x, y, z];
    Ok((rest, Vector3::from(vec)))
}
