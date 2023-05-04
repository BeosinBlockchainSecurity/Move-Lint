use move_compiler::parser::ast as AST1;
use move_compiler::typing::ast as AST4;
use super::utils::visitor::Visitor;

pub struct Detector7<'a> {
    context: &'a mut super::Context,
    ast: &'a super::Ast,
    meta: &'a mut super::Detector,
}

impl<'a> Detector7<'a> {
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
                        AST4::UnannotatedExp_::BinopExp(e1, op, _, _) => {
                            // Operations are resolved from outer layer to inner layer, example: x / y * z => (e1 / e2) * e2
                            if let AST1::BinOp_::Mul = &op.value {
                                // let mut has_div = false;
                                // e1.visit_pre(&mut |e, stop| {
                                //     if let AST4::UnannotatedExp_::BinopExp(_, op, _, _) = e.exp.value {
                                //         if let AST1::BinOp_::Div = &op.value {
                                //             has_div = true;
                                //             *stop = true;
                                //         }
                                //     }
                                // });
                                // if has_div {
                                //     self.add_issue(&exp.exp.loc);
                                // }
                                if self.has_binop_div_in_exp(e1) {
                                    self.add_issue(&exp.exp.loc);
                                }
                            }
                        },
                        _ => (),
                    }
                });
            }
        }
        anyhow::Ok(())
    }

    fn has_binop_div_in_exp(&mut self, exp: &AST4::Exp) -> bool {
        return match &exp.exp.value {
            AST4::UnannotatedExp_::BinopExp(e1, op, _, e2) => {
                match &op.value {
                    AST1::BinOp_::Div => true,
                    _ => self.has_binop_div_in_exp(e1) || self.has_binop_div_in_exp(e2),
                }
            }
            _ => false,
        }
    }

    fn add_issue(&mut self, loc: &move_ir_types::location::Loc) {
        self.context.issues.add(super::Issue::new(
            super::IssueInfo::from(&self.meta.info),
            super::IssueLoc::from(&self.ast, loc),
        ));
    }
}

impl<'a> super::AbstractDetector for Detector7<'a> {
    fn info() -> super::DetectorInfo {
        super::DetectorInfo {
            no: 7,
            wiki: String::from(""),
            title: String::from("multiplication comes before division"),
            verbose: String::from("Multiplication comes before division, otherwise the result precision may be lower."),
            level: super::DetectorLevel::Info,
        }
    }

    fn detect(context: &mut super::Context, ast: &super::Ast, detector: &mut super::Detector) -> anyhow::Result<()> {
        Detector7::new(context, ast, detector).detect()
    }
}