use super::{Ast, DetectorInfo, DetectorLevel};

#[derive(Debug, Clone)]
pub struct IssueInfo {
    pub no: u16,
    pub wiki: String,
    pub title: String,
    pub verbose: String,
    pub level: DetectorLevel,
    pub description: Option<String>,
}

impl PartialEq for IssueInfo {
    fn eq(&self, other: &Self) -> bool {
        self.no == other.no &&
        self.wiki == other.wiki &&
        self.title == other.title &&
        self.verbose == other.verbose &&
        self.level == other.level &&
        self.description == other.description
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl IssueInfo {
    pub fn from(info: &DetectorInfo) -> Self {
        let info = info.clone();
        Self {
            no: info.no,
            wiki: info.wiki,
            title: info.title,
            verbose: info.verbose,
            level: info.level,
            description: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IssueLoc {
    pub file: String,
    pub start: u32,
    pub end: u32,
    pub lines: Vec<u32>,
}

impl PartialEq for IssueLoc {
    fn eq(&self, other: &Self) -> bool {
        self.file == other.file &&
        self.start == other.start &&
        self.end == other.end &&
        self.lines == other.lines
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl IssueLoc {
    pub fn from(ast: &Ast, loc: &move_ir_types::location::Loc) -> Self {
        if let Some(f) = ast.files.get_file(&loc.file_hash()) {
            let range = loc.usize_range();
            Self {
                file: f.filename(),
                start: loc.start() as u32,
                end: loc.end() as u32,
                lines: f.get_lines(range),
            }
        } else {
            Self::empty()
        }
    }

    pub fn empty() -> Self {
        Self { file: String::from(""), start: 0, end: 0, lines: vec![] }
    }
}

#[derive(Debug)]
pub struct Issue {
    pub info: IssueInfo,
    pub loc: IssueLoc,
}

impl Issue {
    pub fn new(info: IssueInfo, loc: IssueLoc) -> Self {
        Self {
            info,
            loc,
        }
    }
}

impl PartialEq for Issue {
    fn eq(&self, other: &Self) -> bool {
        self.info == other.info && self.loc == other.loc
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

#[derive(Debug)]
pub struct Issues(Vec<Issue>);

impl Issues {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn contains(&self, x: &Issue) -> bool {
        self.0.contains(x)
    }

    pub fn add(&mut self, x: Issue) -> &mut Self {
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

    pub fn iter(&self) -> core::slice::Iter<Issue> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<Issue> {
        self.0.iter_mut()
    }

    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&Issue, &Issue) -> core::cmp::Ordering,
    {
        self.0.sort_by(compare)
    }
}