use std::{fs::File, io::{Read, Write}, path::{Path, PathBuf}};

use super::namespace::Namespace;

#[derive(Debug)]
pub struct Function<'a> {
    pub namespace: &'a Namespace<'a>,
    pub name: String,

    file: Option<File>,
}

impl<'a> Read for Function<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.file {
            Some(file) => file.read(buf),
            None => {
                self.file = Some(File::open(&self.name).unwrap());
                self.read(buf)
            }
        }
    }
}

impl<'a> Write for Function<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.file {
            Some(file) => file.write(buf),
            None => {
                let path = self.get_path();
                if let Some(parent) = Path::new(&path).parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent)?;
                    }
                }

                println!("writing to file: {}", path.to_string_lossy());

                self.file = Some(File::create(path).unwrap());
                self.write(buf)
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.file {
            Some(file) => file.flush(),
            None => {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "No bytes has been written yet"))
            }
        }
    }
}

impl<'a> Function<'a> {
    pub fn get_path(&self) -> PathBuf {
        self.namespace.datapack.path
            .join(&self.namespace.datapack.name)
            .join("data")
            .join(&self.namespace.name)
            .join("functions")
            .join(format!("{}.{}", self.name, "mcfunction"))
    }
    pub fn is_char_valid(c: char) -> bool {
        for legal_char in 'a'..='z' {
            if c == legal_char { return true }
        }
        for legal_char in '0'..='9' {
            if c == legal_char { return true }
        }
        for legal_char in "-_/".chars() {
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

    pub fn try_new(namespace: &'a Namespace, name: String) -> Result<Self, std::io::Error> {
        match Self::is_name_valid(&name) {
            true => Ok(Self {
                namespace,
                name,

                file: None,
            }),
            false => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Name is not valid"))
        }
    }
}
