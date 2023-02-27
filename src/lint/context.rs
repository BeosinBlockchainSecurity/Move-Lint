use super::{LintConfig, Issues, Detectors, Ast};

pub struct Context {
    pub config: LintConfig,
    pub issues: Issues,
}

impl Context {
    pub fn new(config: LintConfig) -> Self {
        Self {
            config,
            issues: Issues::new(),
        }
    }

    pub fn lint(&mut self, ast: &Ast, detectors: Option<Detectors>) -> anyhow::Result<()> {
        let mut detectors = if let Some(x) = detectors { x } else { Detectors::default() };
        for d in detectors.iter_mut() {
            if let Err(err) = d.detect(self, ast) {
                return Err(err);
            }
        }
        Ok(())
    }
}