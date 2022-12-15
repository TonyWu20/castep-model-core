use std::fmt::Debug;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, line_ending, space1},
    combinator::recognize,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::parser::decimal;

mod atom_parser;
mod helper;

trait ParserState: Debug {}

#[derive(Debug)]
/// A parser changes its state line by line.
struct MsiParser<'a, S: ParserState> {
    to_parse: &'a str,
    num_atom: usize,
    num_bond: usize,
    state: S,
}

#[derive(Debug)]
/// A zero-sized struct marking the parser received the file content.
/// It will transits to `Model<ModelStates::Init>` with taking until
/// the beginning of a model
struct Loaded;

impl ParserState for Loaded {}

impl<'a> MsiParser<'a, Loaded> {
    fn new(input: &'a str) -> Self {
        Self {
            to_parse: input,
            num_atom: 0,
            num_bond: 0,
            state: Loaded,
        }
    }
    fn get_to_model(input: &str) -> IResult<&str, &str> {
        take_until("(1 Model")(input)
    }
    fn starts(self) -> MsiParser<'a, Model> {
        let (rest, _): (&'a str, &'a str) = Self::get_to_model(self.to_parse).unwrap();
        MsiParser {
            to_parse: rest,
            num_atom: self.num_atom,
            num_bond: self.num_bond,
            state: Model {
                current: ModelStates::Init,
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
struct Model {
    current: ModelStates,
}

#[derive(Debug)]
enum ModelStates {
    Init,
    Attribute,
    Atom,
    Bond,
    Failure,
}
impl ParserState for Model {}
impl ParserState for ModelStates {}

impl<'a> MsiParser<'a, Model> {
    fn move_from_model(input: &str) -> IResult<&str, &str> {
        recognize(tuple((tag("(1 Model"), line_ending)))(input)
    }
    /// Go to next field, consuming the opening left parenthesis "("
    /// # Example:
    /// \s\s(A I CRY/DISPLAY (192 256))\r\n -> A I CRY/DISPLAY (192 256))\r\n
    /// \s\s)\r\n -> next line
    fn next_field(input: &str) -> IResult<&str, &str> {
        alt((
            preceded(space1, tag("(")),
            delimited(space1, tag(")"), line_ending),
        ))(input)
    }
    fn attribute_starts(input: &str) -> IResult<&str, &str> {
        terminated(tag("A"), space1)(input)
    }
    fn atom_starts(input: &str) -> IResult<&str, &str> {
        preceded(decimal, preceded(space1, tag("Atom")))(input)
    }
    fn bond_starts(input: &str) -> IResult<&str, &str> {
        preceded(decimal, preceded(space1, tag("Bond")))(input)
    }
    fn branch_states(self) -> Self {
        if let Ok((rest, result)) =
            alt((Self::attribute_starts, Self::atom_starts, Self::bond_starts))(self.to_parse)
        {
            if result == "A" {
                Self {
                    to_parse: rest,
                    state: Model {
                        current: ModelStates::Attribute,
                    },
                    ..self
                }
            } else if result == "Atom" {
                Self {
                    to_parse: rest,
                    state: Model {
                        current: ModelStates::Atom,
                    },
                    num_atom: self.num_atom + 1,
                    ..self
                }
            } else {
                Self {
                    to_parse: rest,
                    state: Model {
                        current: ModelStates::Bond,
                    },
                    num_bond: self.num_bond + 1,
                    ..self
                }
            }
        } else {
            Self {
                state: Model {
                    current: ModelStates::Failure,
                },
                ..self
            }
        }
    }
    fn next(mut self) -> Self {
        match self.state.current {
            ModelStates::Init => {
                let (rest, _) = Self::move_from_model(self.to_parse).unwrap();
                self.to_parse = rest;
                let (rest, _) = Self::next_field(self.to_parse).unwrap();
                self.to_parse = rest;
                Self::branch_states(self)
            }
            ModelStates::Atom => {
                todo!();
            }
            ModelStates::Bond => {
                todo!()
            }
            ModelStates::Attribute => {
                todo!()
            }
            ModelStates::Failure => {
                panic!("Failed at parsing: {}", self.to_parse)
            }
        }
    }
}

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
