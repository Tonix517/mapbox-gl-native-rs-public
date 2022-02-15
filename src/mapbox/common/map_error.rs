use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum MapErrorTag {
    //Unknown,
    //Style,
    //Manifest,
    Network,
    //DiskCache,
}

#[derive(Debug)]
pub struct MapError {
    pub tag: MapErrorTag,
    pub msg: String,
}

impl MapError {
    pub fn new(tag: MapErrorTag, msg: String) -> MapError {
        MapError { tag, msg }
    }
}

impl Display for MapError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", &self.msg)
    }
}

impl Error for MapError {
    fn description(&self) -> &str {
        "MapError"
    }
}
