mod build;
mod config;
mod core;
mod file;
mod lock;

pub use self::config::AstConfig;
pub use self::core::{PackageAst, FullyAst};
pub use self::file::{FileSource, FileSources};

pub fn main(path: Option<std::path::PathBuf>, config: AstConfig) -> anyhow::Result<PackageAst> {
    build::build_ast(path, config)
}