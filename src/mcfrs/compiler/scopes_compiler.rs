use std::{iter::Peekable, ops::RangeFrom};

use itertools::Itertools;

use crate::mcfrs::scope::Scope;

pub struct ScopesCompiler<'a, I>
where I: Iterator<Item = Scope<'a>>{
    source: I,

    buffered: Vec<Scope<'a>>,
    next_anonymous_scope_name: RangeFrom<usize>,
}

impl<'a, I> Iterator for ScopesCompiler<'a, I>
where I: Iterator<Item = Scope<'a>> {
    type Item = Scope<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.buffered.pop() {
            None => match self.source.next() {
                Some(mut scope) => {
                    let lines = scope
                        .content
                        .clone();
                    let lines = lines
                        .lines()
                        .peekable();
                    let mut new_scopes: Vec<Scope<'a>> = Vec::new();

                    self.generate_scopes(&mut scope, lines, &mut new_scopes);
                    self.buffered = new_scopes;
                    Some(scope)
                }
                None => None
            },
            scope => scope
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
            buffered: Vec::new(),
            next_anonymous_scope_name: (0 as usize)..,
        }
    }

    pub fn generate_scopes<'b, J>(&mut self, scope: &mut Scope<'a>, mut lines: Peekable<J>, new_scopes: &mut Vec<Scope<'a>>) -> ()
    where J: Iterator<Item = &'b str> {
        let mut new_content = String::new();

        while lines.peek().is_some() {
            new_content += lines
                .by_ref()
                .peeking_take_while(|line| line.chars().next() != Some('\t'))
                .join("\n")
                .as_str();

            let new_scope = lines
                .by_ref()
                .peeking_take_while(|line| line.chars().next() == Some('\t'))
                .map(|line| &line[1..])
                .collect::<Vec<_>>();

            if new_scope.len() > 0 {
                let mut scope = Scope::new_unnamed(self.next_anonymous_scope_name.next().unwrap(), scope.namespace);

                new_content += " ";
                new_content += scope
                    .get_reference_name()
                    .as_str();
                new_content += "\n";

                self.generate_scopes(&mut scope, new_scope.into_iter().peekable(), new_scopes);
                new_scopes.push(scope);
            }
        }

        scope.content = new_content;
    }
}

