use std::{collections::{HashMap, HashSet}, iter::{Peekable, Rev, repeat}, ops::Range};

use itertools::Itertools;
use sha2::digest::generic_array::sequence::Split;

use crate::{mcfrs::{scope::Scope, util::get_indent}, vanilla::namespace::Namespace};

pub struct MacroDefinition<'a> {
    namespace: &'a Namespace<'a>,

    name: String,
    parameters: Vec<String>,
    body: String,

    has_separate_scope: bool,
}

impl<'a> MacroDefinition<'a> {
    pub fn new<'b, I>(namespace: &'a Namespace<'a>, definition: &str, content: &mut Peekable<I>) -> Self 
    where I: Iterator<Item = &'b str> {
        let mut words = definition
            .trim()
            .split(" ")
            .skip(2);

        let macro_name = words
            .next()
            .unwrap()
            .to_string();

        let macro_parameters = words
            .map(|p| p.to_string())
            .collect::<Vec<_>>();


        let indent = definition
            .find(|c| c != '\t')
            .unwrap();

        let mut has_separate_scope = false;

        content
            .peeking_take_while(|line| match line.split_once("with") {
                Some((prefix, _)) => prefix.trim().len() == 0,
                None => false
            })
            .map(|line| line.split_once("with").unwrap().1.trim())
            .for_each(|setting| {
                if setting == "scope" { has_separate_scope = true; }
            });

        let macro_body = content
            .peeking_take_while(|line| line.chars().skip(indent).next() == Some('\t'))
            .map(|line| &line[indent + 1..])
            .join("\n");

        Self {
            namespace,
            name: macro_name,
            parameters: macro_parameters,
            body: macro_body,
            has_separate_scope
        }
    }

    pub fn call_into_string<'b, I>(&'b self, parameters: I) -> String
    where I: Iterator<Item = &'b str> {
        let mut body: Vec<String> = Vec::new();

        self.call_into_code(parameters, &mut body, 0);

        body
            .into_iter()
            .collect::<String>()
    }

    pub fn call_into_code<'b, I>(&'b self, parameters: I, out: &mut Vec<String>, indent: usize)
    where I: Iterator<Item = &'b str> {
        let parameters = self
            .parameters
            .iter()
            .zip(parameters);

        let mut body = self
            .body
            .clone();

        for (name, value) in parameters {
            body = body
                .replace(name, value);
        }

        let indent = repeat("\t")
            .take(indent)
            .collect::<String>();

        let indent = indent.as_str();

        body
            .lines()
            .map(|line| format!("{}{}\n", indent, line))
            .for_each(|line| out.push(line));

    }

}

pub struct MacroCompiler<'a, I>
where I: Iterator<Item = Scope<'a>> {
    source: I,
    buffered: Vec<Scope<'a>>,

    definitions: Vec<MacroDefinition<'a>>,
    calls: HashMap<(String, String), String>,
    next_scope_id: Rev<Range<usize>>,
}

pub trait MacroCompilerExt<'a, I>: Sized + Iterator<Item = Scope<'a>>
where I: Iterator<Item = Scope<'a>> {
    // Compiles generate function stuff
    // If definition preceeds call, call will paste definition content, not call generated function
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
            definitions: Vec::new(),
            calls: HashMap::new(),
            next_scope_id: (0 as usize..usize::MAX).rev()
        }
    }
}


impl<'a, I> Iterator for MacroCompiler<'a, I>
where I: Iterator<Item = Scope<'a>> {
    type Item = Scope<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let scope = match self.source.next() {
            Some(scope) => Some(scope),
            None => self.buffered.pop(),
        };

        match scope {
            Some(mut scope) => {
                let content = scope
                    .content;

                let mut content = content
                    .lines()
                    .peekable();

                let mut new_body = Vec::new();

                // Separate all macro definitions from normal scope's code
                while content.peek().is_some() {
                    let mut body = content
                        .peeking_take_while(|line| line.trim().split(" ").next() != Some("generate"))
                        .collect::<Vec<_>>();

                    new_body.append(&mut body);

                    if let Some(definition) = content.next() {
                        self
                            .definitions
                            .push(MacroDefinition::new(scope.namespace, definition, &mut content));
                    }
                }

                let mut newer_body: Vec<String> = Vec::new();
                let mut scope_was_polluted = false;

                // Convert macro calls into valid commands
                for line in new_body {
                    match line.split_once("call") {
                        Some((prefix, payload)) => {
                            let mut payload = payload.trim().split(" ");
                            let macro_name = payload.next().unwrap();
                            let macro_parameters = payload.collect::<Vec<_>>();

                            let name_param = (
                                macro_name.to_string(),
                                macro_parameters.iter().join(" ")
                            );

                            match self
                                .calls
                                .iter() 
                                .find(|(key, _)| **key == name_param) {
                                
                                Some((_, name)) => {
                                    newer_body.push(format!("{}function {}\n", prefix, name));
                                },
                                None => {
                                    let definition = self
                                        .definitions
                                        .iter()
                                        .find(|definition| definition.name == macro_name)
                                        .expect(&format!("Missing macro definition for \"{}\"!", macro_name));

                                    if definition.has_separate_scope {

                                        let next_id = self.next_scope_id.next().unwrap();
                                        let scope = Scope::new_unnamed_scope(
                                            next_id,
                                            scope.namespace,
                                            definition.call_into_string(macro_parameters.into_iter()));

                                        self
                                            .calls
                                            .insert(name_param, scope.get_reference_name());

                                        newer_body.push(format!("{}function {}\n", prefix, scope.get_reference_name()));
                                        self.buffered.push(scope);

                                    } else {
                                        scope_was_polluted = true;
                                        let indent = if prefix.trim().len() == 0 {
                                            prefix.len()
                                        } else {
                                            get_indent(prefix)
                                        };
                                        definition.call_into_code(macro_parameters.into_iter(), &mut newer_body, indent);
                                    }
                                }
                            }

                        },
                        None => newer_body.push(format!("{}\n", line)),
                    } 
                }

                scope.content = newer_body
                    .into_iter()
                    .collect();

                if scope_was_polluted {
                    self.buffered.push(scope);
                    self.next()
                } else {
                    Some(scope)
                }
            },
            None => None
        }
    }


}

