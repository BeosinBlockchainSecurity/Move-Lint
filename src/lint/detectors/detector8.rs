use move_package::source_package::{
    layout::SourcePackageLayout,
    parsed_manifest::DependencyKind,
};

pub struct Detector8<'a> {
    context: &'a mut super::Context,
    ast: &'a super::Ast,
    meta: &'a mut super::Detector,
    branch_names: Vec<&'a str>,
}

impl<'a> Detector8<'a> {
    fn new(context: &'a mut super::Context, ast: &'a super::Ast, detector: &'a mut super::Detector) -> Self {
        Self {
            context,
            ast,
            meta: detector,
            branch_names: vec!["master", "main", "dev", "develop"],
        }
    }

    fn detect(&mut self) -> anyhow::Result<()> {
        let manifest = &self.ast.source_info.manifest;
        for (k, v) in &manifest.dependencies {
            if let DependencyKind::Git(info) = &v.kind {
                if self.branch_names.contains(&info.git_rev.as_str()) {
                    self.add_issue(k.as_str())
                }
            }
        }
        if self.ast.build_options.dev_mode {
            for (k, v) in &manifest.dev_dependencies {
                if let DependencyKind::Git(info) = &v.kind {
                    if self.branch_names.contains(&info.git_rev.as_str()) {
                        self.add_issue(k.as_str())
                    }
                }
            }
        }
        anyhow::Ok(())
    }

    fn add_issue(&mut self, package_name: &str) {
        let description = format!("依赖库'{}'使用分支名代替版本可能导致错误", package_name);
        self.context.issues.add(super::Issue::new(
            super::IssueInfo::from(&self.meta.info).description(description),
            super::IssueLoc {
                file: SourcePackageLayout::Manifest.location_str().to_string(),
                start: 0,
                end: 0,
                lines: vec![0],
            },
        ));
    }
}

impl<'a> super::AbstractDetector for Detector8<'a> {
    fn info() -> super::DetectorInfo {
        super::DetectorInfo {
            no: 8,
            wiki: String::from(""),
            title: String::from("依赖库未明确版本"),
            verbose: String::from("依赖库版本应使用版本号或commit号，避免使用分支名"),
            level: super::DetectorLevel::Info,
        }
    }

    fn detect(context: &mut super::Context, ast: &super::Ast, detector: &mut super::Detector) -> anyhow::Result<()> {
        Detector8::new(context, ast, detector).detect()
    }
}