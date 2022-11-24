use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct InvalidIndex;

impl Display for InvalidIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid index")
    }
}

impl Error for InvalidIndex {}

#[derive(Debug)]
/// Error type when determining plane normal.
pub struct CollinearPoints;

impl Display for CollinearPoints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The given points are on one line!")
    }
}

impl Error for CollinearPoints {}

#[derive(Debug)]
/// Error type when coordinate becomes NaN
pub struct InvalidCoord();

impl Display for InvalidCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NaN value exists in coordinates!")
    }
}

impl Error for InvalidCoord {}
