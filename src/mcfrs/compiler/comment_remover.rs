use crate::mcfrs::scope::Scope;

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
    // Removes lines that begin with "#"
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

