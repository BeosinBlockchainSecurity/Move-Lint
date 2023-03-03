use serde::{ser::{self, SerializeStruct, SerializeSeq}, Serialize};
use super::{Ast, DetectorInfo, DetectorLevel};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct IssueInfo {
    pub no: u16,
    pub wiki: String,
    pub title: String,
    pub verbose: String,
    pub level: DetectorLevel,
    pub description: Option<String>,
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct IssueLoc {
    pub file: String,
    pub start: u32,
    pub end: u32,
    pub lines: Vec<u32>,
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

#[derive(Debug, PartialEq, Eq)]
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

impl ser::Serialize for Issue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let mut s = serializer.serialize_struct("Issue", 10)?;
        // info
        s.serialize_field("no", &self.info.no)?;
        s.serialize_field("wiki", &self.info.wiki)?;
        s.serialize_field("title", &self.info.title)?;
        s.serialize_field("verbose", &self.info.verbose)?;
        s.serialize_field("level", &self.info.level)?;
        s.serialize_field("description", &self.info.description)?;
        // local
        s.serialize_field("file", &self.loc.file)?;
        s.serialize_field("start", &self.loc.start)?;
        s.serialize_field("end", &self.loc.end)?;
        s.serialize_field("lines", &self.loc.lines)?;
        s.end()
    }
}

#[derive(Debug)]
pub struct Issues(Vec<Issue>);
impl ser::Serialize for Issues {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for element in &self.0 {
            seq.serialize_element(element)?;
        }
        seq.end()
    }
}

impl Issues {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn to_vec(&self) -> Vec<&Issue> {
        self.0.iter().map(|i|i).collect::<Vec<_>>()
    }

    pub fn contains(&self, x: &Issue) -> bool {
        self.0.contains(x)
    }

    pub fn get(&self, idx: usize) -> &Issue {
        &self.0[idx]
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

    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&Issue, &Issue) -> core::cmp::Ordering,
    {
        self.0.sort_by(compare)
    }
}

//**************************************************************************************************
// Display
//**************************************************************************************************

impl std::fmt::Display for IssueInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "no: {}\nwiki: {}\ntitle: {}\nverbose: {}\nlevel: {}\ndescription: {}",
            self.no,
            self.wiki,
            self.title,
            self.verbose,
            self.level,
            self.description.clone().unwrap_or("None".to_string()),
        )
    }
}

impl std::fmt::Display for IssueLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "file: {}\nrange: ({}, {})\nlines: {:?}",
            self.file,
            self.start,
            self.end,
            self.lines,
        )
    }
}

impl std::fmt::Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.info, self.loc)
    }
}

impl std::fmt::Display for Issues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.iter().map(|i| i.to_string()).fold("".to_string(), |x, y| format!("{x}\n\n\n{y}")))
    }
}