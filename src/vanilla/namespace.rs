use std::path::PathBuf;

use super::datapack::Datapack;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Namespace<'a> {
    pub datapack: &'a Datapack,
    pub name: String,
}

impl<'a> Namespace<'a> {
    pub fn get_functions_path(&self) -> PathBuf {
        self.datapack.path
            .join(&self.datapack.name)
            .join("data")
            .join(&self.name)
            .join("functions")
            .into()
    }

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

    pub fn try_new(datapack: &'a Datapack, name: String) -> Result<Self, std::io::Error> {
        match Self::is_name_valid(&name) {
            true => Ok(Self {
                datapack,
                name,
            }),
            false => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Name is not valid")),
        }
    }
}
