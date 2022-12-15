use std::fmt::Debug;

use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{char, space1},
    sequence::{delimited, separated_pair, tuple},
    IResult,
};

use crate::parser::decimal;

trait ParserState: Debug {}

#[derive(Debug)]
/// A parser changes its state line by line.
struct MsiParser<'a, S: ParserState> {
    to_parse: &'a str,
    state: S,
}

#[derive(Debug)]
struct Loaded;

impl ParserState for Loaded {}

impl<'a> MsiParser<'a, Loaded> {
    fn new(input: &'a str) -> Self {
        Self {
            to_parse: input,
            state: Loaded,
        }
    }
    fn get_to_model(input: &str) -> IResult<&str, &str> {
        take_until("(1 Model")(input)
    }
    fn starts(self) -> MsiParser<'a, Model<ModelStates>> {
        let (rest, _): (&'a str, &'a str) = Self::get_to_model(self.to_parse).unwrap();
        MsiParser {
            to_parse: rest,
            state: Model {
                state: ModelStates::Init,
            },
        }
    }
}

#[derive(Debug)]
/// A zero-sized struct, marking the parser is parsing a model.
/// At this state, the parser could enter the following state:
///    1. Parsing attributes of the model.
///    2. Parsing an atom object.
///    3. Parsing a bond object.
///    4. ... more if future needs.
/// Parsing directions will be limited if it enters `Atom` or `Bond`
/// state, until it reaches the end of the object.
struct Model<MS: ParserState> {
    state: MS,
}

#[derive(Debug)]
enum ModelStates {
    Init,
    Attribute,
    Atom,
    Bond,
}
impl<MS: ParserState> ParserState for Model<MS> {}
impl ParserState for ModelStates {}

#[derive(Debug)]
struct Atom;
#[derive(Debug)]
struct Bond;

mod error;

#[derive(Debug)]
struct CryDisplay((u32, u32));

fn detect_cry_display(input: &str) -> IResult<&str, &str> {
    delimited(tag("(A "), tag("I CRY/DISPLAY"), space1)(input)
}
fn parse_cry_display(input: &str) -> IResult<&str, CryDisplay> {
    let (rest, (num_1, num_2)) = delimited(
        char('('),
        separated_pair(decimal, space1, decimal),
        char(')'),
    )(input)?;
    let (num_1, num_2) = (num_1.parse::<u32>().unwrap(), num_2.parse::<u32>().unwrap());
    Ok((rest, CryDisplay((num_1, num_2))))
}
fn next_object(input: &str) -> IResult<&str, &str> {
    take_until("(")(input)
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use super::MsiParser;

    #[test]
    fn parsing() {
        let file_content = read_to_string("SAC_GDY_V.msi").unwrap();
        let parser = MsiParser::new(&file_content);
        let parser = parser.starts();
        println!("{:?}", parser);
    }
}
