use std::{collections::{HashMap, VecDeque}, fs::File, io::{Read, Write}, iter::Rev, ops::{Range, RangeFrom}, path::PathBuf};

use itertools::Itertools;
use regex::Regex;
use sha2::Digest;
use walkdir::WalkDir;

use crate::vanilla::{function::Function, namespace::Namespace};

use super::{scope::Scope, util::get_indent};

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

pub struct ScopesCompiler<'a, I>
where I: Iterator<Item = Scope<'a>>{
    source: I,

    buffered_out: VecDeque<Scope<'a>>,
    next_anonymous_scope_name: RangeFrom<usize>,
}

impl<'a, I> Iterator for ScopesCompiler<'a, I>
where I: Iterator<Item = Scope<'a>> {
    type Item = Scope<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.buffered_out.pop_front() {
            Some(scope) => Some(scope),
            None => match self.source.next() {
                Some(scope) => {
                    let lines = scope.content.lines()
                        .map(|l| l.trim_end())
                        .filter(|l| l.len() > 0);

                    let mut stack: Vec<Scope<'a>> = vec![Scope::new(scope.name.clone(), scope.namespace)];

                    for line in lines {
                        let indent = get_indent(line);

                        if stack.len() < indent {
                            let new_scope = Scope::new_unnamed(self.next_anonymous_scope_name.next().unwrap(), scope.namespace);
                            stack.last_mut().unwrap().content += " ";
                            stack.last_mut().unwrap().content += new_scope.get_reference_name().as_str();
                            stack.push(new_scope);
                        } 
                        while stack.len() > indent {
                            self.buffered_out.push_back(stack.pop().unwrap())
                        }

                        stack.last_mut().unwrap().content += "\n";
                        stack.last_mut().unwrap().content += line.trim_start();
                    }

                    while stack.len() > 0 {
                        self.buffered_out.push_back(stack.pop().unwrap());
                    }

                    self.next()
                }
                None => None
            }
        }
    }
}

pub trait ScopesCompilerExt<'a>: Sized + Iterator<Item = Scope<'a>> {
    // Converts indentations into scopes
    fn scopes(self) -> ScopesCompiler<'a, Self>;
}

impl<'a, I> ScopesCompilerExt<'a> for I
where I: Iterator<Item = Scope<'a>> {
    fn scopes(self) -> ScopesCompiler<'a, Self> { ScopesCompiler::new(self) }
}

impl<'a, I> ScopesCompiler<'a, I>
where I: Iterator<Item = Scope<'a>> {
    pub fn new(source: I) -> Self {
        Self {
            source,
            buffered_out: VecDeque::new(),
            next_anonymous_scope_name: (0 as usize)..
        }
    }
}

pub trait ScopeBurnerExt<'a>: Sized + Iterator<Item = Scope<'a>> {
    fn burn(self) -> Result<(), std::io::Error>;
}

impl<'a, I> ScopeBurnerExt<'a> for I
where I: Iterator<Item = Scope<'a>> {
    fn burn(self) -> Result<(), std::io::Error> {
        for scope in self {
            Function::try_new(scope.namespace, scope.name)?
                .write_all(scope.content.as_bytes())?;
        }

        Ok(())
    }
}

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

pub struct MacroCall {
    scope_id: usize,
    parameters: Vec<String>,
}

impl MacroCall {
    pub fn new(scope_id: usize, parameters: Vec<String>) -> Self { Self { scope_id, parameters } }
}

pub struct MacroCompiler<'a, I>
where I: Iterator<Item = Scope<'a>> {
    source: I,
    buffered: Vec<Scope<'a>>,

    calls: HashMap<String, Vec<MacroCall>>,
    next_scope_id: Rev<Range<usize>>,
}

pub trait MacroCompilerExt<'a, I>: Sized + Iterator<Item = Scope<'a>>
where I: Iterator<Item = Scope<'a>> {
    // Compiles generate function stuff
    fn macros(self) -> MacroCompiler<'a, I>;
}

impl<'a, I> MacroCompilerExt<'a, I> for I
where I: Iterator<Item = Scope<'a>> {
    fn macros(self) -> MacroCompiler<'a, I> {
        MacroCompiler::new(self)
    }
}

impl<'a, I> MacroCompiler<'a, I>
where I: Iterator<Item = Scope<'a>> {
    pub fn new(source: I) -> Self {
        Self {
            source,
            buffered: Vec::new(),
            calls: HashMap::new(),
            next_scope_id: (0 as usize..usize::MAX).rev()
        }
    }
}


impl<'a, I> Iterator for MacroCompiler<'a, I>
where I: Iterator<Item = Scope<'a>> {
    type Item = Scope<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let scope;

        match self.source.next() {
            Some(pulled_scope) => scope = Some(pulled_scope),
            None => scope = self.buffered.pop()
        }

        match scope {
            Some(mut scope) => {
                let mut lines = scope.content
                    .lines()
                    .peekable();

                let mut new_lines: Vec<String> = Vec::new();

                loop {
                    match lines.next() {
                        Some(line) => {
                            // let line = line.trim();
                            let mut words = line.split(" ");

                            match words.next() { 
                                None => {},
                                Some("generate") => if words.next() == Some("function") {
                                    let (macro_name, parameters, content) = get_macro_definition(line, words, &mut lines);

                                    // This long multiline spaggetti finds and converts macro calls
                                    // by the given name, and for each of them generates new scope,
                                    // that is then put into the buffer for later use.
                                    self.calls.iter()
                                        .filter_map(|(name, calls)| {
                                            if name == macro_name { Some(calls.into_iter()) }
                                            else { None }
                                        })
                                        .flatten()
                                        .map(|call| {
                                            let mut replaced_content = content.clone();

                                            // Zip into key value pairs names and values of
                                            // parameters, then replace all keys to values in macro
                                            // content
                                            parameters.iter()
                                                .zip(call.parameters.iter())
                                                .for_each(|(key, value)| {
                                                    replaced_content = replaced_content.replace(key, value);
                                                });

                                            (call.scope_id, replaced_content)
                                        })
                                        .collect::<Vec<_>>()
                                        .into_iter()
                                        .for_each(|(id, content)| {
                                            let mut scope = Scope::new_unnamed(id, scope.namespace);
                                            scope.content = content;
                                            self.buffered.push(scope);
                                        });
                                },
                                Some(_) => {
                                    let mut words = line.split(" ").peekable();
                                    let mut replaced_line = words
                                        .by_ref()
                                        .peeking_take_while(|&word| word.trim() != "call")
                                        .collect::<Vec<_>>()
                                        .join(" ");

                                    if words.next() == Some("call") {
                                        let macro_name = words.next().unwrap().to_string();
                                        let parameters = words.map(|p| p.to_string())
                                            .collect::<Vec<_>>();

                                        let previous_same_call = self.calls.iter()
                                            .filter_map(|(name, calls)| if name.as_str() == macro_name { Some(calls) } else { None })
                                            .flatten()
                                            .find(|call| call.parameters == parameters);

                                        match previous_same_call {
                                            Some(call) => {
                                                let function_call = match replaced_line.len() > 0 {
                                                    true => format!(" function {}", Scope::new_unnamed(call.scope_id, scope.namespace).get_reference_name()),
                                                    false => format!("function {}", Scope::new_unnamed(call.scope_id, scope.namespace).get_reference_name())
                                                };

                                                replaced_line += function_call.as_str();
                                            },
                                            None => {
                                                let new_scope = MacroCall::new(self.next_scope_id.next().unwrap(), parameters);
                                                let function_call = match replaced_line.len() > 0 {
                                                    true => format!(" function {}", Scope::new_unnamed(new_scope.scope_id, scope.namespace).get_reference_name()),
                                                    false => format!("function {}", Scope::new_unnamed(new_scope.scope_id, scope.namespace).get_reference_name()),
                                                };

                                                replaced_line += function_call.as_str();
                                                if !self.calls.contains_key(&macro_name) {
                                                    self.calls.insert(macro_name, vec![new_scope]);
                                                } else {
                                                    self.calls.get_mut(&macro_name)
                                                        .unwrap()
                                                        .push(new_scope);
                                                }

                                            }
                                        }

                                    } 
                                    new_lines.push(replaced_line);

                                }
                            }


                        },
                        None => break
                    }
                }

                let new_content = new_lines.join("\n");

                scope.content = new_content;

                Some(scope)
            },
            None => None
        }
    }
}

fn get_macro_definition<'a>(line: &'a str, mut words: std::str::Split<'a, &'a str>, lines: &mut std::iter::Peekable<std::str::Lines>) -> (&'a str, Vec<&'a str>, String) {
    let current_indent = get_indent(line);
    let macro_name = words.next().unwrap();
    let parameters: Vec<&str> = words.collect();
    let content: String = lines
        .by_ref()
        .peeking_take_while(|l| get_indent(l) > current_indent)
        .map(|line| &line[current_indent..])
        .collect::<Vec<_>>()
        .join("\n");
    (macro_name, parameters, content)
}


pub struct CommentRemover<'a, I>
where I: Iterator<Item = Scope<'a>> {
    source: I,
}

impl<'a, I> CommentRemover<'a, I>
where I: Iterator<Item = Scope<'a>>
{
    pub fn new(source: I) -> Self { Self { source } }
}

pub trait CommentRemoverExt<'a, I>: Sized + Iterator<Item = Scope<'a>>
where I: Iterator<Item = Scope<'a>> {
    // Compiles generate function stuff
    fn comment_remove(self) -> CommentRemover<'a, I>;
}

impl<'a, I> CommentRemoverExt<'a, I> for I
where I: Iterator<Item = Scope<'a>> {
    fn comment_remove(self) -> CommentRemover<'a, I> {
        CommentRemover::new(self)
    }
}

impl<'a, I> Iterator for CommentRemover<'a, I>
where I: Iterator<Item = Scope<'a>> {
    type Item = Scope<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.source.next() {
            Some(mut scope) => {
                let new_content = scope.content.lines()
                    .filter(|line| line.trim().chars().next().unwrap_or('#') != '#')
                    .collect::<Vec<_>>()
                    .join("\n");

                scope.content = new_content;
                Some(scope)
            }
            None => None
        }
    }
}
