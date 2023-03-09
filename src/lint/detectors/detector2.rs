use move_compiler::expansion::ast as AST2;
use move_compiler::naming::ast as AST3;
use move_ir_types::sp;
use super::utils::visitor::Visitor;

pub struct Detector2<'a> {
    context: &'a mut super::Context,
    ast: &'a super::Ast,
    meta: &'a mut super::Detector,
}

impl<'a> Detector2<'a> {
    fn new(context: &'a mut super::Context, ast: &'a super::Ast, detector: &'a mut super::Detector) -> Self {
        Self {
            context,
            ast,
            meta: detector,
        }
    }

    fn detect(&mut self) -> anyhow::Result<()> {
        for (_, _, module) in &self.ast.full_ast.naming.modules {
            for (_, _, func) in &module.functions {
                func.visit_pre(&mut |exp, _| {
                    match &exp.value {
                        AST3::Exp_::Builtin(func, sp!(_, es)) => {
                            if let AST3::BuiltinFunction_::Assert(_) = &func.value {
                                assert!(es.len() == 2);
                                if let AST3::Exp_::Value(arg) = &es[1].value {
                                    let is_zero = match arg.value {
                                        AST2::Value_::InferredNum(v) |
                                        AST2::Value_::U256(v) => v == move_core_types::u256::U256::zero(),
                                        AST2::Value_::U8(v) => v == 0,
                                        AST2::Value_::U16(v) => v == 0,
                                        AST2::Value_::U32(v) => v == 0,
                                        AST2::Value_::U64(v) => v == 0,
                                        AST2::Value_::U128(v) => v == 0,
                                        _ => false,
                                    };
                                    if is_zero {
                                        self.add_issue(&exp.loc)
                                    }
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

    fn add_issue(&mut self, loc: &move_ir_types::location::Loc) {
        self.context.issues.add(super::Issue::new(
            super::IssueInfo::from(&self.meta.info),
            super::IssueLoc::from(&self.ast, loc),
        ));
    }
}

impl<'a> super::AbstractDetector for Detector2<'a> {
    fn info() -> super::DetectorInfo {
        super::DetectorInfo {
            no: 2,
            wiki: String::from(""),
            title: String::from("assert错误码使用"),
            verbose: String::from("对于assert的错误码未定义，直接使用0"),
            level: super::DetectorLevel::Info,
        }
    }

    fn detect(context: &mut super::Context, ast: &super::Ast, detector: &mut super::Detector) -> anyhow::Result<()> {
        Detector2::new(context, ast, detector).detect()
    }
}