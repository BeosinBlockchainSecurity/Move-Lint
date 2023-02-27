pub mod ast;
pub mod lint;

use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;
use ast::{config::AstBuildConfig, compiled_ast::CompiledAst};
use lint::{LintConfig};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Config {
    /// Path to a package which the command should be run with respect to.
    #[clap(long = "path", short = 'p', global = true, parse(from_os_str))]
    pub package_path: Option<PathBuf>,

    /// Print additional diagnostics if available.
    #[clap(short = 'v', global = true)]
    pub verbose: bool,

    /// Package build options
    #[clap(flatten)]
    pub build_config: AstBuildConfig,

    /// Lint options
    #[clap(flatten)]
    pub lint_config: LintConfig,

    /// Print results as json if available.
    #[clap(long = "json", short = 'j', global = true)]
    pub json: bool,
}

pub fn gen_move_ast(path: Option<PathBuf>, config: AstBuildConfig) -> Result<CompiledAst> {
    ast::main(path, config)
}

pub fn move_lint(config: lint::LintConfig, ast: &CompiledAst) -> Result<lint::Issues> {
    lint::main(config, ast).and_then(|c| Ok(c.issues))
}