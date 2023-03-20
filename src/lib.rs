pub mod ast;
pub mod lint;

use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;
use ast::{AstConfig, PackageAst};
use lint::{LintConfig};

#[derive(Debug, Parser, Default)]
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
    pub ast_config: AstConfig,

    /// Lint options
    #[clap(flatten)]
    pub lint_config: LintConfig,

    /// Print results as json if available.
    #[clap(long = "json", short = 'j', global = true)]
    pub json: bool,
}

pub fn gen_move_ast(path: Option<PathBuf>, config: AstConfig) -> Result<PackageAst> {
    ast::main(path, config)
}

pub fn move_lint(config: lint::LintConfig, ast: &PackageAst) -> Result<lint::Issues> {
    lint::main(config, ast, None).and_then(|c| Ok(c.issues))
}