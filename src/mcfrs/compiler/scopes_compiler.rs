use std::{collections::{HashMap, VecDeque}, ops::RangeFrom};

use crate::mcfrs::{scope::Scope, util::get_indent};

pub struct ScopesCompiler<'a, I>
where I: Iterator<Item = Scope<'a>>{
    source: I,

    buffered_out: VecDeque<Scope<'a>>,
    next_anonymous_scope_name: RangeFrom<usize>,
    compiled_scopes: HashMap<String, String>,
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
                            let new_scope = Scope::new_unnamed(0, scope.namespace);
                            stack.push(new_scope);
                        } 
                        while stack.len() > indent {
                            let mut new_scope = stack.pop().unwrap();
                            match self.compiled_scopes.get(&new_scope.content) {
                                Some(reference_name) => {
                                    stack.last_mut().unwrap().content += " ";
                                    stack.last_mut().unwrap().content += reference_name.as_str();
                                },
                                None => {
                                    new_scope.set_reference_id(self.next_anonymous_scope_name.next().unwrap());
                                    stack.last_mut().unwrap().content += " ";
                                    stack.last_mut().unwrap().content += new_scope.get_reference_name().as_str();
                                    self.compiled_scopes.insert(new_scope.content.clone(), new_scope.get_reference_name());
                                    self.buffered_out.push_back(new_scope);
                                }
                            }
                        }

                        stack.last_mut().unwrap().content += "\n";
                        stack.last_mut().unwrap().content += line.trim_start();
                    }

                    while stack.len() > 0 {
                        let mut new_scope = stack.pop().unwrap();
                        match self.compiled_scopes.get(&new_scope.content) {
                            Some(_) => {
                            },
                            None => {
                                if stack.len() > 0 {
                                    new_scope.set_reference_id(self.next_anonymous_scope_name.next().unwrap());
                                }
                                self.compiled_scopes.insert(new_scope.content.clone(), new_scope.get_reference_name());
                                self.buffered_out.push_back(new_scope);
                            }
                        }
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
            next_anonymous_scope_name: (0 as usize)..,
            compiled_scopes: HashMap::new(),
        }
    }
}

