use std::ops::Deref;
use std::str::FromStr;

use move_compiler::parser::ast as AST1;
use move_compiler::expansion::ast as AST2;
use move_compiler::naming::ast as AST3;
use move_compiler::typing::ast as AST4;
use move_ir_types::sp;
use super::utils::visitor::Visitor;

pub struct Detector5<'a> {
    context: &'a mut super::Context,
    ast: &'a super::Ast,
    meta: &'a mut super::Detector,
}

impl<'a> Detector5<'a> {
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
                        AST4::UnannotatedExp_::BinopExp(e1, op, _, e2) => {
                            match &op.value {
                                AST1::BinOp_::Shl | AST1::BinOp_::Shr => {
                                    // e1 << e2 | e1 >> e2
                                    if let AST3::Type_::Apply(_, sp!(_, AST3::TypeName_::Builtin(sp!(_, typ))), _) = &e1.deref().ty.value {
                                        // bit of e1
                                        let v1_bit: Option<u128> = match &typ {
                                            AST3::BuiltinTypeName_::U8 => Some(8),
                                            AST3::BuiltinTypeName_::U16 => Some(16),
                                            AST3::BuiltinTypeName_::U32 => Some(32),
                                            AST3::BuiltinTypeName_::U64 => Some(64),
                                            AST3::BuiltinTypeName_::U128 => Some(128),
                                            AST3::BuiltinTypeName_::U256 => Some(256),
                                            _ => None,
                                        };
                                        if let Some(v1_bit) = v1_bit {
                                            // AST4::UnannotatedExp_::Value // constant node
                                            if let AST4::UnannotatedExp_::Value(v2) = &e2.deref().exp.value {
                                                let is_overflow = match &v2.value {
                                                    AST2::Value_::InferredNum(v) |
                                                    AST2::Value_::U256(v) => {
                                                        if let Ok(v1_bit_256) = move_core_types::u256::U256::from_str(v1_bit.to_string().as_str()) {
                                                            v >= &v1_bit_256
                                                        } else {
                                                            false
                                                        }
                                                    },
                                                    AST2::Value_::U8(v) => (*v as u128) >= v1_bit,
                                                    AST2::Value_::U16(v) => (*v as u128) >= v1_bit,
                                                    AST2::Value_::U32(v) => (*v as u128) >= v1_bit,
                                                    AST2::Value_::U64(v) => (*v as u128) >= v1_bit,
                                                    AST2::Value_::U128(v) => (*v as u128) >= v1_bit,
                                                    _ => false,
                                                };
                                                if is_overflow {
                                                    self.add_issue(&exp.exp.loc);
                                                }
                                            }
                                        }
                                    }
                                },
                                _ => (),
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

impl<'a> super::AbstractDetector for Detector5<'a> {
    fn info() -> super::DetectorInfo {
        super::DetectorInfo {
            no: 5,
            wiki: String::from(""),
            title: String::from("shift operation overflow"),
            verbose: String::from("Make sure that the second operand is less than the width in bits of the first operand and no overflow during a shift operation."),
            level: super::DetectorLevel::Info,
        }
    }

    fn detect(context: &mut super::Context, ast: &super::Ast, detector: &mut super::Detector) -> anyhow::Result<()> {
        Detector5::new(context, ast, detector).detect()
    }
}