use nom::{
    branch::alt,
    character::complete::{char, one_of},
    combinator::{opt, recognize},
    multi::many1,
    sequence::{preceded, tuple},
    IResult,
};

pub mod msi_parser;

pub fn decimal(input: &str) -> IResult<&str, &str> {
    recognize(many1(one_of("0123456789")))(input)
}
pub fn float(input: &str) -> IResult<&str, &str> {
    alt((
        // Case one: .42
        recognize(tuple((
            char('.'),
            decimal,
            opt(tuple((one_of("eE"), opt(one_of("+-")), decimal))),
        ))), // Case two: 42e42 and 42.42e42
        recognize(tuple((
            decimal,
            opt(preceded(char('.'), decimal)),
            one_of("eE"),
            opt(one_of("+-")),
            decimal,
        ))), // Case three: 42. and 42.42
        // Case four: 42., +42., 42.42, and -42.e-05
        recognize(tuple((
            opt(one_of("+-")),
            decimal,
            char('.'),
            opt(decimal),
            opt(one_of("eE")),
            opt(one_of("+-")),
            opt(decimal),
        ))),
    ))(input)
}

#[test]
fn test_float() {
    let number = "-2.865153883599e-05";
    let number_2 = "-2.";
    let parse_float = float(number);
    match parse_float {
        Ok((_, num)) => println!("{num}"),
        Err(e) => println!("{e}"),
    }
    let parse_float_2 = float(number_2);
    match parse_float_2 {
        Ok((_, num)) => println!("{num}"),
        Err(e) => println!("{e}"),
    }
}
