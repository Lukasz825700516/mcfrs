use std::{fs::File, io::Read, path::PathBuf};

use walkdir::WalkDir;

use crate::{mcfrs::scope::Scope, vanilla::namespace::Namespace};

pub struct FileCompiler<'a> {
    namespace: &'a Namespace<'a>,
    files: Box<dyn Iterator<Item = PathBuf> + 'a>,
}

impl<'a> Iterator for FileCompiler<'a> {
    type Item = Scope<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.files.next() {
            Some(path) => {
                let functions_dir = self.namespace.get_functions_path();
                let mut functions_dir = functions_dir.into_iter();

                let scope_path: PathBuf = path.into_iter()
                    .skip_while(|_| functions_dir.next().is_some())
                    .collect();

                let mut scope_path = scope_path.to_string_lossy()
                    .chars()
                    .rev()
                    .skip_while(|&c| c != '.')
                    .skip(1)
                    .collect::<Vec<_>>();

                scope_path.reverse();
                let scope_path = scope_path.into_iter()
                    .collect::<String>();

                let mut scope = Scope::new(scope_path, self.namespace);

                match File::open(path) {
                    Ok(mut f) => {
                        f.read_to_string(&mut scope.content).unwrap();
                        Some(scope)
                    },
                    Err(_) => None
                }
            },
            None => None,
        }
    }
}

impl<'a> FileCompiler<'a> {
    fn get_files_iterator(namespace: &'a Namespace) -> Box<dyn Iterator<Item = PathBuf> + 'a> {
        let datapack = namespace.datapack;
        let functions_path = datapack.path
            .join(&datapack.name)
            .join("data")
            .join(&namespace.name)
            .join("functions");

        let functions = WalkDir::new(functions_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| e.path().extension().is_some())
            .filter(|e| e.path().extension().unwrap().to_str().is_some())
            .filter(|e| e.path().extension().unwrap().to_str().unwrap() == "mcf")
            .map(|e| PathBuf::from(e.path()));

        Box::new(functions)
    }

    pub fn new(namespace: &'a Namespace<'a>) -> Self {
        Self {
            namespace,
            files: Self::get_files_iterator(namespace)
        }
    }
}

