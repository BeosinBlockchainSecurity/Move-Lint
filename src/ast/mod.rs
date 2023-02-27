pub mod config;
pub mod build;
pub mod compiled_ast;
mod file;
pub mod lock;

use move_package::source_package::layout::SourcePackageLayout;
use std::path::PathBuf;

pub use file::{FileSource, FileSources};

pub fn reroot_path(path: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let path = path.unwrap_or_else(|| PathBuf::from("."));
    // Always root ourselves to the package root, and then compile relative to that.
    let rooted_path = SourcePackageLayout::try_find_root(&path.canonicalize()?)?;
    std::env::set_current_dir(&rooted_path).unwrap();

    Ok(PathBuf::from("."))
}

pub fn main(path: Option<PathBuf>, config: config::AstBuildConfig) -> anyhow::Result<compiled_ast::CompiledAst> {
    build::ast(path, config)
    // build::main(path, config)
}