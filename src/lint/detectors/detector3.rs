use move_compiler::typing::ast as AST4;
use super::utils::visitor::Visitor;

pub struct Detector3<'a> {
    context: &'a mut super::Context,
    ast: &'a super::Ast,
    meta: &'a mut super::Detector,
}

impl<'a> Detector3<'a> {
    fn new(context: &'a mut super::Context, ast: &'a super::Ast, detector: &'a mut super::Detector) -> Self {
        Self {
            context,
            ast,
            meta: detector,
        }
    }

    fn detect(&mut self) -> anyhow::Result<()> {
        for (_, _, module) in &self.ast.full_ast.typing.modules {
            for (_, _, func) in &module.functions {
                func.visit_pre(&mut |exp, _| {
                    match &exp.exp.value {
                        AST4::UnannotatedExp_::Cast(e, typ) => {
                            // e as typ
                            if &e.ty.value == &typ.value {
                                self.add_issue(&exp.exp.loc);
                            }
                        },
                        _ => (),
                    }
                });
            }
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

impl<'a> super::AbstractDetector for Detector3<'a> {
    fn info() -> super::DetectorInfo {
        super::DetectorInfo {
            no: 3,
            wiki: String::from(""),
            title: String::from("unnecessary type conversion"),
            verbose: String::from("Unnecessary type conversion, for example let a: u64; a as u64;"),
            level: super::DetectorLevel::Info,
        }
    }

    fn detect(context: &mut super::Context, ast: &super::Ast, detector: &mut super::Detector) -> anyhow::Result<()> {
        Detector3::new(context, ast, detector).detect()
    }
}