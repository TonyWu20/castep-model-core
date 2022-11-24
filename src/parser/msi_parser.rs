use na::Point3;
use nom::character::complete::{char, line_ending, space0, space1};
use nom::combinator::recognize;
use nom::multi::many1;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::alpha1,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::atom::Atom;
use crate::lattice::{LatticeModel, LatticeVectors};
use crate::model_type::msi::MsiModel;

use super::{decimal, float};
extern crate nom;

impl<'a> TryFrom<&'a str> for LatticeModel<MsiModel> {
    type Error = nom::Err<nom::error::Error<&'a str>>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (rest, _) = msi_model_start(value)?;
        let parse_lattice_vec = lattice_vector(rest);
        if let Ok(res) = parse_lattice_vec {
            let (rest, lattice_vectors) = res;
            let lattice_vector_flatten: Vec<f64> = lattice_vectors
                .iter()
                .flat_map(|slice| -> Vec<f64> { slice.to_vec() })
                .collect();
            let lattice_vector_matrix = na::Matrix3::from_vec(lattice_vector_flatten);
            let lattice_vectors: LatticeVectors<MsiModel> =
                LatticeVectors::new(lattice_vector_matrix, MsiModel::default());
            let (rest, _) = skip_to_atoms(rest)?;
            let (_, atoms) = many1(parse_atom)(rest)?;
            Ok(LatticeModel::new(
                Some(lattice_vectors),
                atoms,
                MsiModel::default(),
            ))
        } else {
            let (_, atoms) = many1(parse_atom)(rest)?;
            Ok(LatticeModel::new(None, atoms, MsiModel::default()))
        }
    }
}

/// Skip the header of the file.
fn msi_model_start(input: &str) -> IResult<&str, &str> {
    alt((
        preceded(take_until("(1 Model\r\n"), tag("(1 Model\r\n")),
        preceded(take_until("(1 Model\n"), tag("(1 Model\n")),
    ))(input)
}
fn skip_to_atoms(input: &str) -> IResult<&str, &str> {
    preceded(alt((take_until("\r\n"), take_until("\n"))), line_ending)(input)
}

/// Parse XYZ in `msi` file. Since it possibly write `0` instead of `0.0`, we have to parse with `alt((float, decimal))`
fn parse_xyz(input: &str) -> IResult<&str, [f64; 3]> {
    let (rest, res) = terminated(
        tuple((
            terminated(alt((float, decimal)), space0),
            terminated(alt((float, decimal)), space0),
            alt((float, decimal)),
        )),
        alt((tag("))\r\n"), tag("))\n"))),
    )(input)?;
    let (x, y, z) = res;
    Ok((
        rest,
        [
            x.parse::<f64>().unwrap(),
            y.parse::<f64>().unwrap(),
            z.parse::<f64>().unwrap(),
        ],
    ))
}
/// Parse the lattice vector block in `msi` file format.
fn lattice_vector(input: &str) -> IResult<&str, [[f64; 3]; 3]> {
    let (rest, _) = preceded(take_until("A D A3 ("), tag("A D A3 ("))(input)?;
    let (rest, vector_a) = parse_xyz(rest)?;
    let (rest, _) = preceded(take_until("A D B3 ("), tag("A D B3 ("))(rest)?;
    let (rest, vector_b) = parse_xyz(rest)?;
    let (rest, _) = preceded(take_until("A D C3 ("), tag("A D C3 ("))(rest)?;
    let (rest, vector_c) = parse_xyz(rest)?;
    Ok((rest, [vector_a, vector_b, vector_c]))
}

/// Parse atom blocks in `msi` file format.
/// Use space1 to handle 2/4 spaces cases. Use `line_ending` to handle `\n` (in unix-format) or `\r\n` (in dos-format)
fn parse_atom<'b, 'a: 'b>(input: &'a str) -> IResult<&'a str, Atom<MsiModel>> {
    // This gives the nth of `Atom` blocks.
    let (rest, _) = tuple((
        tuple((space1, tag("("))),
        decimal,
        tag(" Atom"),
        line_ending,
    ))(input)?;
    // Parser to recognize and consume `")\r\n` or `")\n`
    let quoted_ending_block = |input: &'a str| -> IResult<&'a str, &'b str> {
        recognize(tuple((tag("\")"), line_ending)))(input)
    };
    // Parser to recognize and consume `)\r\n` or `)\n`
    let ending_block = |input: &'a str| -> IResult<&'a str, &'b str> {
        recognize(tuple((tag(")"), line_ending)))(input)
    };
    // This will parse the atomic number and element name.
    let (rest, element_line) = delimited(
        tuple((space1, tag("(A C ACL \""))),
        separated_pair(decimal, char(' '), alpha1), // Example: (A C ACL "6 C")
        quoted_ending_block,
    )(rest)?;
    let (element_id, element_symbol) = element_line;
    // Alternative cases: with a line of `Label` before `XYZ`, or without
    let (rest, xyz) = alt((
        preceded(
            tuple((
                tuple((space1, tag("(A C Label \""))),
                alpha1,
                quoted_ending_block,
                tuple((space1, tag("(A D XYZ ("))),
            )),
            parse_xyz,
        ),
        preceded(tuple((space1, tag("(A D XYZ ("))), parse_xyz),
    ))(rest)?;
    // Parse `atom_id`
    let (rest, atom_id) = preceded(tuple((space1, tag("(A I Id "))), decimal)(rest)?;
    // Travel out the block
    let (rest, _) = tuple((ending_block, space1, ending_block))(rest)?;
    let element_id = element_id.parse::<u32>().unwrap();
    let atom_id = atom_id.parse::<u32>().unwrap();
    let xyz = Point3::from_slice(&xyz);
    Ok((
        rest,
        Atom::new(
            element_symbol.to_string(),
            element_id,
            xyz,
            atom_id,
            MsiModel::default(),
        ),
    ))
}

#[cfg(test)]
#[test]
fn test_msi() {
    use std::fs::{read_to_string, write};

    use crate::model_type::{cell::CellModel, ModelInfo};
    #[derive(Debug)]
    struct GDYLattice<T: ModelInfo + Clone> {
        lattice: LatticeModel<T>,
        name: String,
    }

    impl<T: ModelInfo + Clone> GDYLattice<T> {
        fn lattice(&self) -> &LatticeModel<T> {
            &self.lattice
        }
    }

    let test_lat = read_to_string("SAC_GDY_Ag.msi").unwrap();
    let lat = LatticeModel::<MsiModel>::try_from(test_lat.as_str()).unwrap();
    let cell: LatticeModel<CellModel> = LatticeModel::from(lat.clone());
    let gdy_ag_lat = GDYLattice {
        lattice: lat,
        name: "SAC_GDY_Ag.msi".to_string(),
    };
    let msi_export = gdy_ag_lat.lattice().msi_export();
    println!("{}", msi_export);
    write("SAC_GDY_Ag.cell", cell.cell_export()).unwrap();
}
