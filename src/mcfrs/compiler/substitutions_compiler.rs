use regex::Regex;
use sha2::Digest;

use crate::mcfrs::scope::Scope;

pub struct SubstitutionsCompiler<'a, I>
where I: Iterator<Item = Scope<'a>> {
    source: I,
    score_regex: Regex,
    hash_regex: Regex,
}

impl<'a, I> Iterator for SubstitutionsCompiler<'a, I>
where I: Iterator<Item = Scope<'a>> {
    type Item = Scope<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.source.next() {
            Some(mut scope) => {
                scope.content = scope.content.replace("$this", &scope.get_reference_name());
                scope.content = scope.content.replace("$namespace", &scope.namespace.name);

                for capture in self.hash_regex.captures_iter(&scope.content.clone()) {
                    let full_match = &capture[0];
                    let unhashed_value = &capture[1];

                    let mut hasher = sha2::Sha256::new();
                    hasher.update(unhashed_value);

                    let hashed_value = hasher.finalize();
                    let hashed_value = data_encoding::BASE32.encode(&hashed_value);
                    let hashed_value = &hashed_value[0..16];
                    let hashed_value = hashed_value.to_lowercase();

                    scope.content = scope.content.replace(full_match, &hashed_value);
                }

                for capture in self.score_regex.captures_iter(&scope.content.clone()) {
                    let full_match = &capture[0];
                    let score_name = &capture[1];
                    let score_objective = &capture[2];

                    scope.content = scope.content.replace(full_match, format!("{} {}", score_name, score_objective).as_str());
                }

                Some(scope)
            }
            None => None,
        }
    }
}

impl<'a, I> SubstitutionsCompiler<'a, I>
where I: Iterator<Item = Scope<'a>> {
    pub fn new(source: I) -> Self { 
        Self {
            source,
            score_regex: Regex::new(r"([a-z0-9\\-_]+)@([a-z0-9\\-_]+)").unwrap(),
            hash_regex: Regex::new(r"#\[([a-z0-9\-_.]+)\]").unwrap(),
        }
    }
}

pub trait SubstitutionsCompilerExt<'a, I>: Sized + Iterator<Item = Scope<'a>> 
where I: Iterator<Item = Scope<'a>> {
    // Converts:
    // $this -> current scope reference name
    // score@board -> score board
    // #[some_value] -> first 16 base32 chars of sha256
    fn substitutions(self) -> SubstitutionsCompiler<'a, I>;
}

impl<'a, I> SubstitutionsCompilerExt<'a, I> for I
where I: Iterator<Item = Scope<'a>> {
    fn substitutions(self) -> SubstitutionsCompiler<'a, I> {
        SubstitutionsCompiler::new(self)
    }
}

