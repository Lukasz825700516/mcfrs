use std::path::PathBuf;

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum McVersion {
    V1_17_4,
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Datapack {
    pub name: String,

    pub version: McVersion,
    pub description: String,

    // Path to directory in witch datapack is stored.
    pub path: PathBuf,
}

impl Datapack {
    pub fn is_char_valid(c: char) -> bool {
        for legal_char in 'a'..='z' {
            if c == legal_char { return true }
        }
        for legal_char in '0'..='9' {
            if c == legal_char { return true }
        }
        for legal_char in "-_".chars() {
            if c == legal_char { return true }
        }
        return false
    }

    pub fn is_name_valid(name: &str) -> bool {
        for c in name.chars() {
            if !Self::is_char_valid(c) { return false }
        }
        return true

    }

    pub fn try_new(name: String) -> Result<Self, std::io::Error> {
        match Self::is_name_valid(&name) {
            true => Ok(Self {
                name,

                version: McVersion::V1_17_4,
                description: String::new(),

                path: PathBuf::from("."),
            }),
            false => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Name is not valid"))
        }
    }
}
