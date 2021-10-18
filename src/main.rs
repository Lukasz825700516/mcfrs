use std::error::Error;

use mcfrs::compiler::{back_compiler::BackCompilerExt, comment_remover::CommentRemoverExt, file_compiler::FileCompiler, macro_compiler::MacroCompilerExt, scopes_compiler::ScopesCompilerExt, substitutions_compiler::SubstitutionsCompilerExt};
use vanilla::{datapack::Datapack, namespace::Namespace};

mod mcfrs;
mod vanilla;

fn main() -> Result<(), Box<dyn Error>>{
    let datapack = Datapack::try_new(String::from("test"))?;
    let namespace = Namespace::try_new(&datapack, String::from("ns"))?;

    FileCompiler::new(&namespace)
        .comment_remove()
        .macros()
        .scopes()
        .substitutions()
        .back()

        .for_each(|scope| {
            println!("{}:\n{}\n", scope.get_reference_name(), &scope.content);
        });
        // .burn()?;

    Ok(())
}
