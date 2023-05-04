use std::{path::PathBuf, collections::HashMap};
use anyhow::Result;
use move_lint::{
    self,
    Config,
    lint::{
        Detector,
        Detectors,
        IssueInfoNo,
        IssueLocLine,
        Issues,
    }
};

type IssueTags = HashMap<IssueLocLine, Vec<IssueInfoNo>>;
type FileIssueTags = HashMap<String, IssueTags>;

fn parse_issue_tags(text: String) -> IssueTags {
    use regex::{Regex, Captures};
    // /*xxx*/
    let reg = Regex::new(r"/\*[\s\S]*?\*/").unwrap();
    let tag_reg = Regex::new(r"//[ ]*<Issue(:\d{1,}){1,}>").unwrap();
    reg.replace_all(text.as_str(), |caps: &Captures| {
        if let Some(s) = caps.get(0) {
            // replace to space
            s.as_str().chars().map(|c| { if c == '\n' || c == '\r' { c } else { ' ' } }).collect()
        } else {
            "".to_string()
        }
    })
    .split('\n').enumerate().filter_map(|(idx, s)| {
        if s.trim_start().starts_with("//") {
            None
        } else {
            tag_reg.captures(s).and_then(|x| {
                Some((
                    (idx + 1) as IssueLocLine,
                    x.get(0).unwrap().as_str()
                        .split("<Issue:").collect::<Vec<_>>().get(1).unwrap()
                        .split(">").collect::<Vec<_>>().get(0).unwrap()
                        .split(":").map(|x| { x.parse::<IssueInfoNo>().unwrap() })
                        .collect::<Vec<_>>()
                ))
            })
        }
    }).collect()
}

fn run_detector(path: PathBuf, detector: Detector) -> Result<(Issues, FileIssueTags)> {
    let config = Config::default();
    let ast = move_lint::gen_move_ast(Some(path), config.ast_config)?;
    let ret = move_lint::lint::main(config.lint_config, &ast, Some(Detectors::from(vec![detector])))?;
    let issues = ret.issues;
    let issue_tags = ast.source_info.files.meta().values().filter_map(|file| {
        let temp = parse_issue_tags(file.content());
        if temp.is_empty() {
            None
        } else {
            Some((file.filename(), temp))
        }
    }).collect();
    Ok((issues, issue_tags))
}

fn test_detector(detector: Detector) {
    let no = detector.info.no;
    let path = PathBuf::from("tests").join("cases").join(format!("Detector{}", &no));
    let t_path = path.clone();
    let ret = run_detector(path, detector);
    match &ret {
        Ok((issues, tags)) => {
            let mut i_tags: FileIssueTags = HashMap::new();
            issues.iter().for_each(|issue| {
                i_tags.entry(issue.loc.file.clone())
                    .and_modify(|x| {
                        x.entry(issue.loc.lines[0])
                            .and_modify(|t| t.push(issue.info.no))
                            .or_insert(vec![issue.info.no]);
                    })
                    .or_insert(HashMap::from([(issue.loc.lines[0], vec![issue.info.no])]));
            });
            println!("{} => {:?}: \n\t{:?} \n\t{:?}", &no, &t_path, &i_tags, tags);
            assert!(&i_tags == tags, "Detector error: {}", no);
        },
        Err(error) => {
            println!("{} => {:?}: \n\t{:?}", &no, &t_path, &error);
            assert!(false, "{}", error);
        },
    };
}

#[test]
fn test_detectors() {
    for detector in Detectors::default().meta() {
        test_detector(detector);
    }
}
