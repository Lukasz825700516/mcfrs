use std::io::Write;

use crate::{mcfrs::scope::Scope, vanilla::function::Function};

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

