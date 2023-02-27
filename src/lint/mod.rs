mod config;
mod context;
mod detectors;
mod detector;
mod issue;

use crate::ast::compiled_ast::CompiledAst as Ast;
use detectors::Detectors;

pub use config::LintConfig;
pub use context::Context;
pub use detector::{DetectorLevel, DetectorInfo, Detector};
pub use issue::{IssueInfo, IssueLoc, Issue, Issues};

pub fn main(config: LintConfig, ast: &Ast) -> anyhow::Result<Context> {
    let mut context = Context::new(config);
    if let Err(err) = context.lint(ast, None) {
        return Err(err);
    }
    anyhow::Ok(context)
}