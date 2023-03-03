use std::ops::Deref;
use std::str::FromStr;

use move_compiler::parser::ast as AST1;
use move_compiler::expansion::ast as AST2;
use move_compiler::naming::ast as AST3;
use move_compiler::typing::ast as AST4;
use move_ir_types::sp;

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
                self.parse_func(func);
            }
        }
        anyhow::Ok(())
    }

    fn parse_func(&mut self, func: &'a AST4::Function) {
        match &func.body.value {
            AST4::FunctionBody_::Defined(block) => {
                self.parse_func_block(block);
            },
            AST4::FunctionBody_::Native => (),
        }
    }

    fn parse_func_block(&mut self, block: &'a AST4::Sequence) {
        for seq in block {
            match &seq.value {
                AST4::SequenceItem_::Bind(_, _, exp) |
                AST4::SequenceItem_::Seq(exp) => {
                    self.parse_func_exp(exp)
                },
                AST4::SequenceItem_::Declare(_) => (),
            }
        }
    }

    fn parse_func_exp(&mut self, exp: &'a AST4::Exp) {
        let mut exps: Vec<&AST4::Exp> = Vec::new();
        match &exp.exp.value { |
            AST4::UnannotatedExp_::BinopExp(e1, op, _, e2) => {
                match &op.value {
                    AST1::BinOp_::Shl | AST1::BinOp_::Shr => {
                        // e1 << e2 | e1 >> e2
                        if let AST3::Type_::Apply(_, sp!(_, AST3::TypeName_::Builtin(sp!(_, typ))), _) = &e1.deref().ty.value {
                            // e1类型的位数
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
                                // AST4::UnannotatedExp_::Value 常量节点
                                // 只能判断常量，变量值无法判断
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
                // e1、e2中可能嵌套其他exp
                exps.push(e1);
                exps.push(e2);
            },
            AST4::UnannotatedExp_::ModuleCall(call) => {
                exps.push(&call.arguments);
            },
            AST4::UnannotatedExp_::Block(block) => {
                self.parse_func_block(block)
            },
            AST4::UnannotatedExp_::Pack(_, _, _, vars) => {
                exps.append(&mut vars.iter().map(|(_, _, (_, (_, e)))| e).collect::<Vec<&AST4::Exp>>())
            },
            // Vec<e>
            AST4::UnannotatedExp_::ExpList(es) => {
                for el in es {
                    match el {
                        AST4::ExpListItem::Single(e, _) |
                        AST4::ExpListItem::Splat(_, e, _) => exps.push(e)
                    }
                }
            },
            // (e1, e2)
            AST4::UnannotatedExp_::IfElse(_, e1, e2) |
            AST4::UnannotatedExp_::Mutate(e1, e2) => {
                exps.push(e1);
                exps.push(e2);
            },
            // e
            AST4::UnannotatedExp_::Cast(e, _) |
            AST4::UnannotatedExp_::Builtin(_, e) |
            AST4::UnannotatedExp_::Vector(_, _, _, e) |
            AST4::UnannotatedExp_::Borrow(_, e, _) |
            AST4::UnannotatedExp_::TempBorrow(_, e) |
            AST4::UnannotatedExp_::Return(e) |
            AST4::UnannotatedExp_::While(_, e) |
            AST4::UnannotatedExp_::Loop { has_break: _, body: e } |
            AST4::UnannotatedExp_::Assign(_, _, e) |
            AST4::UnannotatedExp_::Abort(e) |
            AST4::UnannotatedExp_::Dereference(e) |
            AST4::UnannotatedExp_::UnaryExp(_, e) |
            AST4::UnannotatedExp_::Annotate(e, _) => exps.push(e),
            // ()
            AST4::UnannotatedExp_::BorrowLocal(_, _) |
            AST4::UnannotatedExp_::Use(_) |
            AST4::UnannotatedExp_::Constant(_, _) |
            AST4::UnannotatedExp_::Value(_) |
            AST4::UnannotatedExp_::Move { from_user: _, var: _ } |
            AST4::UnannotatedExp_::Copy { from_user: _, var: _ } |
            AST4::UnannotatedExp_::Unit{ trailing: _ } |
            AST4::UnannotatedExp_::Break |
            AST4::UnannotatedExp_::Continue |
            AST4::UnannotatedExp_::Spec(_, _) | // sepc
            AST4::UnannotatedExp_::UnresolvedError => (),
        }
        for e in exps {
            self.parse_func_exp(e);
        }
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
            title: String::from("位移运算溢出"),
            verbose: String::from("位移运算时，保证位移数<被位移数的位数，确保左右位移不移除"),
            level: super::DetectorLevel::Info,
        }
    }

    fn detect(context: &mut super::Context, ast: &super::Ast, detector: &mut super::Detector) -> anyhow::Result<()> {
        Detector5::new(context, ast, detector).detect()
    }
}