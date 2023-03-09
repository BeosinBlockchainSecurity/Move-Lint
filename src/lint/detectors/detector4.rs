use move_compiler::expansion::ast as AST2;
use move_compiler::typing::ast as AST4;
use super::utils::visitor::Visitor;

pub struct Detector4<'a> {
    context: &'a mut super::Context,
    ast: &'a super::Ast,
    meta: &'a mut super::Detector,
}

impl<'a> Detector4<'a> {
    fn new(context: &'a mut super::Context, ast: &'a super::Ast, detector: &'a mut super::Detector) -> Self {
        Self {
            context,
            ast,
            meta: detector,
        }
    }

    fn detect(&mut self) -> anyhow::Result<()> {
        use std::collections::BTreeMap;
        let mut private_funcs: BTreeMap<String, move_ir_types::location::Loc> = BTreeMap::new();
        for (_, module_ident, module) in &self.ast.full_ast.typing.modules {
            for (loc, fname, func) in &module.functions {
                if func.visibility == AST2::Visibility::Internal  {
                    // 记录private函数
                    private_funcs.insert(format!("{}::{}", module_ident, fname), loc);
                }
            }
        }
        for (_, _, module) in &self.ast.full_ast.typing.modules {
            for (_, _, func) in &module.functions {
                func.visit_pre(&mut |exp, _| {
                    match &exp.exp.value {
                        AST4::UnannotatedExp_::ModuleCall(call) => {
                            let (module_ident, fname) = (&call.module.value, &call.name.0.value);
                            // 移除被调用的private函数
                            private_funcs.remove_entry(&format!("{}::{}", module_ident, fname));
                        },
                        _ => (),
                    }
                });
            }
        }
        for loc in private_funcs.values() {
            self.add_issue(loc);
        }
        anyhow::Ok(())
    }

    fn add_issue(&mut self, loc: &move_ir_types::location::Loc) {
        self.context.issues.add(super::Issue::new(
            super::IssueInfo::from(&self.meta.info),
            super::IssueLoc::from(&self.ast, loc),
        ));
    }
}

impl<'a> super::AbstractDetector for Detector4<'a> {
    fn info() -> super::DetectorInfo {
        super::DetectorInfo {
            no: 4,
            wiki: String::from(""),
            title: String::from("未使用的private接口"),
            verbose: String::from("存在未使用的private接口"),
            level: super::DetectorLevel::Info,
        }
    }

    fn detect(context: &mut super::Context, ast: &super::Ast, detector: &mut super::Detector) -> anyhow::Result<()> {
        Detector4::new(context, ast, detector).detect()
    }
}