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
}

impl FileSource {
    pub fn new(filename: String, content: String) -> Self {
        Self {
            filename,
            content,
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
        let lines_count = self.content[range.clone()].split('\n').collect::<Vec<_>>().len() as u32;
        let start_line = self.content[0..range.end].split('\n').collect::<Vec<_>>().len() as u32 - lines_count + 1;
        let end_line = start_line + lines_count;
        (start_line..end_line).collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone)]
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

impl core::ops::Index<FileHash> for FileSources {
    type Output = FileSource;
    fn index(&self, index: FileHash) -> &Self::Output {
        &self.0[&index]
    }
}