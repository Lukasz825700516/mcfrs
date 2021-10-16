use std::error::Error;

use mcfrs::compiler::{FileCompiler, MacroCompilerExt, ScopeBurnerExt, ScopesCompilerExt, SubstitutionsCompilerExt};
use vanilla::{datapack::Datapack, namespace::Namespace};

mod mcfrs;
mod vanilla;

fn main() -> Result<(), Box<dyn Error>>{
    let datapack = Datapack::try_new(String::from("test"))?;
    let namespace = Namespace::try_new(&datapack, String::from("ns"))?;

    FileCompiler::new(&namespace)
        .macros()
        .scopes()
        .substitutions()

        .for_each(|scope| {
            println!("{}:\n{}\n", scope.get_reference_name(), &scope.content);
        });
        // .burn()?;

    Ok(())
}
