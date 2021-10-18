use std::{collections::HashMap, iter::{Peekable, Rev}, ops::Range};

use itertools::Itertools;

use crate::{mcfrs::{scope::Scope, util::get_indent}, vanilla::namespace::Namespace};

pub struct MacroDefinition<'a> {
    parameters: Vec<String>,
    namespace: &'a Namespace<'a>,
    name: String,
    content: String,
}

impl<'b> MacroDefinition<'b> {
    fn new<'a>(namespace: &'b Namespace<'b>, line: &'a str, mut words: Peekable<std::str::Split<'a, &'a str>>, lines: &mut std::iter::Peekable<std::str::Lines>) -> Self {
        let current_indent = get_indent(line);
        let macro_name = words.next().unwrap().to_string();
        let parameters: Vec<String> = words.map(|param| param.to_string()).collect();
        let content: String = lines
            .by_ref()
            .peeking_take_while(|l| get_indent(l) > current_indent)
            .map(|line| &line[current_indent..])
            .collect::<Vec<_>>()
            .join("\n");

        Self {
            name: macro_name,
            parameters,
            content,
            namespace,
        }
    }
}

pub struct MacroCall {
    scope_id: usize,
    parameters: Vec<String>,
}

impl MacroCall {
    pub fn new(scope_id: usize, parameters: Vec<String>) -> Self { Self { scope_id, parameters } }
    pub fn get_content(&self, definition: &MacroDefinition) -> String {
        let mut replaced_content = definition.content.clone();

        // Zip into key value pairs names and values of
        // parameters, then replace all keys to values in macro
        // content
        definition.parameters.iter()
            .zip(self.parameters.iter())
            .for_each(|(key, value)| {
                replaced_content = replaced_content.replace(key, value);
            });

        replaced_content
    }
}

pub struct MacroCompiler<'a, I>
where I: Iterator<Item = Scope<'a>> {
    source: I,
    buffered: Vec<Scope<'a>>,

    definitions: Vec<MacroDefinition<'a>>,
    calls: HashMap<String, Vec<MacroCall>>,
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
            calls: HashMap::new(),
            definitions: Vec::new(),
            next_scope_id: (0 as usize..usize::MAX).rev()
        }
    }
    fn save_macro(&mut self, name: String, call: MacroCall) {
        if !self.calls.contains_key(&name) {
            self.calls.insert(name.clone(), vec![call]);
        } else {
            self.calls.get_mut(&name)
                .unwrap()
                .push(call);
        }
    }
    fn generate_scopes(&mut self, definition: &MacroDefinition<'a>) {
        self.calls.iter()
            .filter_map(|(name, calls)| {
                if *name == definition.name { Some(calls.into_iter()) }
                else { None }
            })
            .flatten()
            .map(|call| (call.scope_id, call.get_content(&definition), definition.namespace))
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|(id, content, namespace)| {
                let mut scope = Scope::new_unnamed(id, namespace);
                scope.content = content;
                self.buffered.push(scope);
            });
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
                let mut scope_needs_reprocessing = false;
                let mut lines = scope.content
                    .lines()
                    .peekable();

                let mut new_lines: Vec<String> = Vec::new();

                loop {
                    match lines.next() {
                        Some(line) => {
                            // let line = line.trim();
                            let mut words = line.split(" ").peekable();

                            match words.peek() { 
                                None => {},
                                Some(&"generate") => { words.next(); if words.next() == Some("function") {
                                    let definition = MacroDefinition::new(scope.namespace, line, words, &mut lines);
                                    self.generate_scopes(&definition);
                                    self.definitions.push(definition);
                                }},
                                Some(_) => {

                                    if let Some((prefix, payload)) = line.split_once("call") {
                                        scope_needs_reprocessing = true;

                                        let mut words = payload.trim().split(" ");
                                        let macro_name = words.next().unwrap().to_string();
                                        let parameters = words.map(|p| p.to_string())
                                            .collect::<Vec<_>>();


                                        let previous_same_call = self.calls.iter()
                                            .filter_map(|(name, calls)| if name.as_str() == macro_name { Some(calls) } else { None })
                                            .flatten()
                                            .find(|call| call.parameters == parameters);

                                        let new_line = match self.definitions.iter()
                                            .find(|def| def.name == macro_name && prefix.trim().len() == 0) {
                                            Some(macro_definition) => {
                                                let call = MacroCall::new(self.next_scope_id.next().unwrap(), parameters);
                                                let new_line = call.get_content(macro_definition)
                                                    .lines()
                                                    .map(|line| format!("{}{}", prefix, line))
                                                    .join("\n");

                                                if previous_same_call.is_none() {
                                                    self.save_macro(macro_name, call);
                                                }

                                                new_line
                                            },
                                            None => match previous_same_call {
                                                Some(call) => {
                                                    let function_call = get_call_call(call.scope_id, scope.namespace);
                                                    format!("{}{}", prefix, function_call)
                                                },
                                                None => {
                                                    let new_scope = MacroCall::new(self.next_scope_id.next().unwrap(), parameters);
                                                    let function_call = get_call_call(new_scope.scope_id, scope.namespace);

                                                    self.save_macro(macro_name, new_scope);

                                                    format!("{}{}", prefix, function_call)
                                                }
                                            }
                                        };

                                        new_lines.push(new_line);
                                    } else {
                                        new_lines.push(line.to_string());
                                    }

                                }
                            }


                        },
                        None => break
                    }
                }

                let new_content = new_lines.join("\n");

                scope.content = new_content;

                match scope_needs_reprocessing {
                    true => { self.buffered.push(scope); self.next() }
                    false => Some(scope)
                }
            },
            None => {
                while let Some(definition) = self.definitions.pop() {
                    self.generate_scopes(&definition);
                }

                if self.buffered.len() > 0 { self.next() }
                else { None }
            }
        }
    }


}

fn get_call_call(scope_id: usize, namespace: &Namespace) -> String {
    format!("function {}", Scope::new_unnamed(scope_id, namespace).get_reference_name())
}

