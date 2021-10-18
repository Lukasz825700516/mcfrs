use itertools::Itertools;

use crate::mcfrs::scope::Scope;

pub struct BackCompiler<'a, I>
where I: Iterator<Item = Scope<'a>> {
    source: I,
}

impl<'a, I> BackCompiler<'a, I>
where I: Iterator<Item = Scope<'a>>
{
    pub fn new(source: I) -> Self { Self { source } }
}

pub trait BackCompilerExt<'a, I>: Sized + Iterator<Item = Scope<'a>>
where I: Iterator<Item = Scope<'a>> {
    // Removes lines that begin with "#"
    fn back(self) -> BackCompiler<'a, I>;
}

impl<'a, I> BackCompilerExt<'a, I> for I
where I: Iterator<Item = Scope<'a>> {
    fn back(self) -> BackCompiler<'a, I> {
        BackCompiler::new(self)
    }
}

impl<'a, I> Iterator for BackCompiler<'a, I>
where I: Iterator<Item = Scope<'a>> {
    type Item = Scope<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.source.next() {
            Some(mut scope) => {
                let mut lines = scope.content.lines().peekable();
                let mut new_lines: Vec<String> = Vec::new();
                while let Some(line) = lines.next() {
                    let continuation = lines.by_ref()
                        .peeking_take_while(|line| line.trim().split(" ").next() == Some("back"))
                        .map(|line| line.split_once("back").unwrap().1);

                    let new_line = vec![line].into_iter()
                        .chain(continuation)
                        .collect::<String>();

                    new_lines.push(new_line);
                }

                scope.content = new_lines.into_iter()
                    .join("\n");

                Some(scope)
            }
            None => None
        }
    }
}


