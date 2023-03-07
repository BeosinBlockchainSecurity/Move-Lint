use super::{
    Context as _Context,
    Ast as _Ast,
    DetectorLevel as _DetectorLevel,
    DetectorInfo as _DetectorInfo,
    Detector as _Detector,
    IssueInfo,
    IssueLoc,
    Issue,
};
type Context = _Context;
type Ast = _Ast;
type DetectorLevel = _DetectorLevel;
type DetectorInfo = _DetectorInfo;
type Detector = _Detector;

pub trait AbstractDetector {
    fn info() -> DetectorInfo;
    fn detector() -> Detector {
        Detector::new(Self::info(), Self::detect)
    }
    fn detect(context: &mut Context, ast: &Ast, detector: &mut Detector) -> anyhow::Result<()>;
}

mod utils;
mod detector1;
mod detector2;
mod detector3;
mod detector4;
mod detector5;
mod detector6;
mod detector7;
mod detector8;

pub struct Detectors(Vec<Detector>);

impl Default for Detectors {
    fn default() -> Self {
        Self(vec![
            detector1::Detector1::detector(),
            detector2::Detector2::detector(),
            detector3::Detector3::detector(),
            detector4::Detector4::detector(),
            detector5::Detector5::detector(),
            detector6::Detector6::detector(),
            detector7::Detector7::detector(),
            detector8::Detector8::detector(),
        ])
    }
}

impl Detectors {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn contains(&self, x: &Detector) -> bool {
        self.0.contains(x)
    }

    pub fn add(&mut self, x: Detector) -> &mut Self {
        if !self.contains(&x) {
            self.0.push(x);
        }
        self
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> core::slice::Iter<Detector> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<Detector> {
        self.0.iter_mut()
    }
}
