mod config;
mod context;
mod detector;
mod detectors;
mod issue;

use crate::ast::PackageAst as Ast;

pub use self::config::LintConfig;
pub use self::context::Context;
pub use self::detector::{DetectorLevel, DetectorInfo, Detector};
pub use self::detectors::{AbstractDetector, Detectors};
pub use self::issue::{IssueInfo, IssueLoc, Issue, Issues};

pub fn main(config: LintConfig, ast: &Ast, detectors: Option<Detectors>) -> anyhow::Result<Context> {
    let mut context = Context::new(config);
    if let Err(err) = context.lint(ast, detectors) {
        return Err(err);
    }
    anyhow::Ok(context)
}