use std::{
    ops::Range,
    collections::HashMap,
};
use move_compiler::diagnostics;
use move_command_line_common::files::FileHash;

#[derive(Debug, Clone)]
pub struct FileSource {
    filename: String,
    content: String,
    lines: Vec<u32>,
}

impl FileSource {
    pub fn new(filename: String, content: String) -> Self {
        let lines = content.clone().split('\n').enumerate().map(|(i, _)|i as u32).collect::<Vec<_>>();
        Self {
            filename,
            content,
            lines,
        }
    }

    pub fn from(f: (diagnostics::FileName, String)) -> Self {
        Self::new(f.0.to_string(), f.1)
    }

    pub fn filename(&self) -> String {
        self.filename.clone()
    }

    pub fn content(&self) -> String {
        self.content.clone()
    }

    pub fn get_lines(&self, range: Range<usize>) -> Vec<u32> {
        let lines_count = self.content[0..range.end].split('\n').collect::<Vec<_>>().len();
        let range_lines_count = self.content[range].split('\n').collect::<Vec<_>>().len();
        self.lines[lines_count-range_lines_count..lines_count].to_vec()
    }
}

#[derive(Debug, Clone)]
// pub struct FileSources(diagnostics::FilesSourceText);
pub struct FileSources(HashMap<FileHash, FileSource>);

impl FileSources {
    pub fn new(files: HashMap<FileHash, FileSource>) -> Self {
        Self(files)
    }

    pub fn from(files: diagnostics::FilesSourceText) -> Self {
        let mut files = files;
        Self(files.drain().map(|(k, v)|(k, FileSource::from(v))).collect::<HashMap<_, _>>())
    }

    pub fn meta(&self) -> &HashMap<FileHash, FileSource> {
        &self.0
    }

    pub fn has_file(&self, file_hash: &FileHash) -> bool {
        self.0.contains_key(file_hash)
    }

    pub fn get_file(&self, file_hash: &FileHash) -> Option<&FileSource> {
        self.0.get(file_hash)
    }
}
