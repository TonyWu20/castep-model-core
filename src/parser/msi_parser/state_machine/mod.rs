use std::{fmt::Debug, marker::PhantomData};

use nalgebra::{Matrix3, Point3};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alpha1, alphanumeric1, line_ending, space0, space1},
    combinator::{peek, recognize},
    multi::{many0, many1},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

use crate::{
    atom::{AtomCollection, AtomCollectionBuilder},
    builder_typestate::No,
    lattice::LatticeVectors,
    model_type::Settings,
    parser::{
        decimal,
        msi_parser::state_machine::model_attributes_parser::{
            parse_cry_tolerance, parse_space_group,
        },
    },
    LatticeModel, MsiModel,
};

use self::{
    atom_parser::{parse_acl, parse_id, parse_xyz},
    model_attributes_parser::{hashmap_attrs, parse_periodic_type, parse_vector},
};

mod atom_parser;
mod helper;
mod model_attributes_parser;

pub trait ParserState: Debug {}

#[derive(Debug)]
/// A parser changes its state line by line.
pub struct MsiParser<'a, S: ParserState> {
    // To denote the state that the input
    // has been completly consumed, use `None`
    to_parse: Option<&'a str>,
    // Vec storing string slices of model attributes.
    model_attributes: Vec<&'a str>,
    // Vec storing string slices of atom objects.
    atoms: Vec<&'a str>,
    // Vec storing string slices of bonds
    bonds: Vec<&'a str>,
    // Counter of the attributes.
    num_attr: usize,
    // Counter of the atoms.
    num_atom: usize,
    // Counter of the bonds.
    num_bond: usize,
    // State marker.
    state: PhantomData<S>,
}

/// Methods common for all states.
impl<'a, S: ParserState> MsiParser<'a, S> {
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
    /// Parser to extract the whole content of an attribute tag
    /// within the parenthsis.
    /// It matches when "A" immediately follows a left parenthesis.
    /// Both unix-style (`\n`) and Windows/DOS-style (`\r\n`) line endings
    /// are supported.
    fn take_attribute(input: &str) -> IResult<&str, &str> {
        delimited(
            tuple((space0, tag("("), tag("A"), space1)),
            alt((take_until(")\r\n"), take_until(")\n"))),
            tuple((tag(")"), line_ending)),
        )(input)
    }
    /// Parser to extract the whole content of an object.
    /// It matches when an decimal integral number immediately
    /// follows a left parenthesis, and will take everything
    /// until it finds spaces following with a single `)` and line ending.
    /// Both unix-style (`\n`) and Windows/DOS-style (`\r\n`) line endings
    /// are supported.
    fn take_object(input: &str) -> IResult<&str, &str> {
        delimited(
            tuple((space0, tag("("), decimal, space1)),
            take_until("  )"),
            tuple((space0, tag(")"), line_ending)),
        )(input)
    }
    /// Parser to extract the type tag of an object.
    /// # Note:
    /// **It will consume the line of the tag and move
    /// on to the attributes lines!**
    fn get_object_type(object_input: &str) -> IResult<&str, &str> {
        terminated(alpha1, line_ending)(object_input)
    }
    /// Parser to extract the type tag of an attribute.
    /// # Note:
    /// **It will not consume the type annotation and the spaces before the
    /// value fields. The returned remaining input is untouched.**
    fn get_attribute_type(attr_input: &str) -> IResult<&str, &str> {
        peek(delimited(
            tuple((alpha1, space1)),
            recognize(many1(alt((alphanumeric1, tag("/"))))),
            space1,
        ))(attr_input)
    }
    /// Extract either an attribute or an object.
    /// Only works when the input has been pushed into a `Model`.
    fn get_field(inside_model_input: &str) -> IResult<&str, &str> {
        alt((Self::take_attribute, Self::take_object))(inside_model_input)
    }
    /// Sign of the end of a model.
    fn model_end(input: &str) -> IResult<&str, &str> {
        tag(")")(input)
    }
}

#[derive(Debug)]
/// A zero-sized struct marking the parser received the file content.
/// It will transits to `Model<ModelStates::Init>` with taking until
/// the beginning of a model
pub(crate) struct Loaded;
impl ParserState for Loaded {}

impl<'a> MsiParser<'a, Loaded> {
    /// Init a new parser by feeding the `msi` file,
    /// which has been read into string.
    pub fn new(input: &'a str) -> Self {
        Self {
            to_parse: Some(input),
            num_atom: 0,
            num_bond: 0,
            state: PhantomData,
            model_attributes: Vec::new(),
            atoms: Vec::new(),
            bonds: Vec::new(),
            num_attr: 0,
        }
    }
    /// The file may have one to many comment lines.
    /// Skip to the beginning of the actual content.
    fn get_to_model(input: &str) -> IResult<&str, &str> {
        take_until("(1 Model")(input)
    }
    /// Enter the scope of the model.
    fn enter_model(input: &str) -> IResult<&str, &str> {
        recognize(tuple((tag("(1 Model"), line_ending)))(input)
    }
    /// Transits state into `Start` by entering the scope of model.
    pub fn starts(self) -> MsiParser<'a, Start> {
        let (rest, _): (&'a str, &'a str) = Self::get_to_model(self.to_parse.unwrap()).unwrap();
        let (rest, _) = Self::enter_model(rest).unwrap();
        MsiParser {
            to_parse: Some(rest),
            model_attributes: self.model_attributes,
            atoms: self.atoms,
            bonds: self.bonds,
            num_attr: 0,
            num_atom: 0,
            num_bond: 0,
            state: PhantomData,
        }
    }
}

/// A zero-sized struct, marking the parser is parsing a model.
/// At this state, the parser could be doing:
///    1. Parsing attributes of the model.
///    2. Parsing an atom object.
///    3. Parsing a bond object.
///    4. ... more if future needs.
/// The input will be looped over with the `get_field` function,
/// and push the identified fields to the corresponding vectors
/// to store them in the struct, until the end of model is reached.
/// # Note:
/// An annoying fact is that the `Materials Studio` generated `msi` files
/// may not emit the model objects (atom, bond, etc.) in a strict order,
/// and they will even interlace with each other (e.g., a bond appears
/// among the list of atoms). In other words, it does not commit a
/// strict serialization. So rather than carefully matching the types
/// of the parsed contents one by one, I decide to go through and parse
/// all sorts of things first, while pushing them to their corresponding
/// vectors. Then I can invoke parsing workflow for each vec of field I am
/// interested in.
#[derive(Debug)]
pub(crate) struct Start {}
impl ParserState for Start {}

impl<'a> MsiParser<'a, Start> {
    /// Push the content to `self.atoms`, increment the counter by 1.
    fn push_atom(&mut self, atom_fields: &'a str) {
        self.atoms.push(atom_fields);
        self.num_atom += 1;
    }
    /// Push the content to `self.bonds`, increment the counter by 1.
    fn push_bond(&mut self, bond_fields: &'a str) {
        self.bonds.push(bond_fields);
        self.num_bond += 1;
    }
    /// Push the content to `self.model_attributes`, increment the counter by 1.
    fn push_model_attribute(&mut self, attribute_field: &'a str) {
        self.model_attributes.push(attribute_field);
        self.num_attr += 1;
    }
    /// Loop over the input to parse attributes or objects,
    /// store the parsed contents into corresponding fields,
    /// finished with state transisted to `Analyzed`
    pub fn analyze(mut self) -> MsiParser<'a, Analyzed> {
        // While we have fields
        while let Ok((rest, parsed_field)) = Self::get_field(self.to_parse.unwrap()) {
            // Check if it is an object.
            if let Ok((object_fields, object_type)) = Self::get_object_type(parsed_field) {
                if object_type == "Atom" {
                    self.push_atom(object_fields);
                } else {
                    self.push_bond(object_fields);
                }
            } else {
                // It is a model attribute.
                self.push_model_attribute(parsed_field);
            }
            // Update `self.to_parse` to continue the loop.
            self.to_parse = Some(rest);
        }
        // Fields have been consumed entirely.
        let (_, _model_end) = Self::model_end(self.to_parse.unwrap()).unwrap();
        // Assume the file has only one model...
        self.to_parse = None;
        let Self {
            to_parse,
            model_attributes: attributes,
            atoms,
            bonds,
            num_attr,
            num_atom,
            num_bond,
            state: _,
        } = self;
        // Let's go to the `Analyzed` state.
        MsiParser {
            to_parse,
            model_attributes: attributes,
            atoms,
            bonds,
            num_attr,
            num_atom,
            num_bond,
            state: PhantomData,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Analyzed {}
impl ParserState for Analyzed {}

impl<'a> MsiParser<'a, Analyzed> {
    fn parse_attributes(&self) -> Settings<MsiModel> {
        if self.model_attributes.is_empty() {
            Settings::default()
        } else {
            let attr_table = hashmap_attrs(self.model_attributes.as_ref());
            let (_, periodic_type) =
                parse_periodic_type(attr_table.get("PeriodicType").expect("No `PeriodicType`"))
                    .unwrap();
            let (_, cry_tolerance) =
                parse_cry_tolerance(attr_table.get("CRY/TOLERANCE").expect("No `CRY/TOLERANCE`"))
                    .unwrap();
            let (_, space_group) =
                parse_space_group(attr_table.get("SpaceGroup").expect("No `SpaceGroup`")).unwrap();
            Settings::new_msi_settings(periodic_type, space_group, cry_tolerance)
        }
    }
    fn parse_lattice_vectors(&self) -> Option<LatticeVectors<MsiModel>> {
        if self.model_attributes.is_empty() {
            None
        } else {
            let attr_table = hashmap_attrs(self.model_attributes.as_ref());
            let (_, vec_a) = parse_vector(attr_table.get("A3").unwrap()).unwrap();
            let (_, vec_b) = parse_vector(attr_table.get("B3").unwrap()).unwrap();
            let (_, vec_c) = parse_vector(attr_table.get("C3").unwrap()).unwrap();
            let lattice_vector = Matrix3::from_columns(&[vec_a, vec_b, vec_c]);
            Some(LatticeVectors::new(lattice_vector))
        }
    }
    fn parse_atoms(&self) -> AtomCollection<MsiModel> {
        let mut element_symbols: Vec<String> = Vec::with_capacity(self.num_atom);
        let mut atomic_numbers: Vec<u8> = Vec::with_capacity(self.num_atom);
        let mut xyz_coords: Vec<Point3<f64>> = Vec::with_capacity(self.num_atom);
        let mut atom_ids: Vec<u32> = Vec::with_capacity(self.num_atom);
        let frac_xyz: Vec<Option<Point3<f64>>> =
            (0..self.num_atom).into_iter().map(|_| None).collect();
        self.atoms.iter().for_each(|atom_fields| {
            let (_, atom_attrs) = many0(Self::take_attribute)(atom_fields).unwrap();
            atom_attrs.iter().for_each(|item| {
                if let Ok((_, acl)) = parse_acl(item) {
                    let (num, symbol) = acl;
                    atomic_numbers.push(num);
                    element_symbols.push(symbol.into());
                } else if let Ok((_, xyz)) = parse_xyz(item) {
                    xyz_coords.push(xyz);
                } else if let Ok((_, id)) = parse_id(item) {
                    atom_ids.push(id);
                } else {
                }
            })
        });
        let builder = AtomCollectionBuilder::<MsiModel, No>::new(self.num_atom);
        builder
            .with_atom_ids(&atom_ids)
            .unwrap()
            .with_element_symbols(&element_symbols)
            .unwrap()
            .with_atomic_nums(&atomic_numbers)
            .unwrap()
            .with_xyz_coords(&xyz_coords)
            .unwrap()
            .with_fractional_xyz(&frac_xyz)
            .unwrap()
            .finish()
            .unwrap()
            .build()
    }
    pub fn build_lattice_model(&self) -> LatticeModel<MsiModel> {
        let settings = self.parse_attributes();
        let lattice_vector = self.parse_lattice_vectors();
        let atoms = self.parse_atoms();
        LatticeModel::new(lattice_vector, atoms, settings)
    }
}

mod error;

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use crate::parser::msi_parser::state_machine::Analyzed;

    use super::MsiParser;

    #[test]
    fn parsing() {
        let file_content = read_to_string("SAC_GDY_V.msi").unwrap();
        let parser = MsiParser::new(&file_content);
        let mut parser = parser.starts().analyze();
        println!("{:?}", parser.parse_atoms());
        parser.model_attributes.sort_by_key(|item| {
            let (_, key) = MsiParser::<Analyzed>::get_attribute_type(item).unwrap();
            key
        });
        println!("{:?}", parser.parse_lattice_vectors());
        println!("{:?}", parser.build_lattice_model());
    }
}
