use serde::Serialize;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum DetectorLevel {
    Error,
    Warning,
    Info,
    Unimplemented,
}

#[derive(Debug, Clone)]
pub struct DetectorInfo {
    pub no: u16,
    pub wiki: String,
    pub title: String,
    pub verbose: String,
    pub level: DetectorLevel,
}

pub struct Detector {
    pub info: DetectorInfo,
    detect: fn (&mut super::Context, &super::Ast, &mut Detector) -> Result<()>,
    detected: bool,
}

impl PartialEq for Detector {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }

    fn ne(&self, other: &Self) -> bool {
        self != other
    }
}

impl Detector {
    pub fn new(info: DetectorInfo, detect: fn (&mut super::Context, &super::Ast, &mut Detector) -> Result<()>) -> Self {
        Self {
            info,
            detect,
            detected: false,
        }
    }

    pub fn detect(&mut self, context: &mut super::Context, ast: &super::Ast) -> Result<()> {
        if self.detected {
            anyhow::bail!("The detector is detected, {:?} {:?}.", self.info.no, self.info.title)
        } else {
            self.detected = true;
            let handle = self.detect;
            handle(context, ast, self)
        }
    }

    pub fn detected(&self) -> &bool {
        &self.detected
    }
}
