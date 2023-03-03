use move_compiler::expansion::ast as AST2;
use move_compiler::naming::ast as AST;
use move_ir_types::sp;

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
                self.parse_func(func);
            }
        }
        anyhow::Ok(())
    }

    fn parse_func(&mut self, func: &AST::Function) {
        match &func.body.value {
            AST::FunctionBody_::Defined(block) => {
                self.parse_func_block(block);
            },
            AST::FunctionBody_::Native => (),
        }
    }

    fn parse_func_block(&mut self, block: &AST::Sequence) {
        for seq in block {
            match &seq.value {
                AST::SequenceItem_::Seq(exp) => {
                    self.parse_func_exp(exp)
                },
                _ => (),
            }
        }
    }

    fn parse_func_exp(&mut self, exp: &AST::Exp) {
        let mut exps: Vec<&AST::Exp> = Vec::new();
        match &exp.value {
            AST::Exp_::Builtin(func, sp!(_, es)) => {
                match &func.value {
                    AST::BuiltinFunction_::Assert(_) => {
                        assert!(es.len() == 2);
                        if let AST::Exp_::Value(arg) = &es[1].value {
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
                    },
                    _ => (),
                }
            },
            AST::Exp_::Block(block) => {
                self.parse_func_block(block)
            },
            AST::Exp_::Pack(_, _, _, vars) => {
                exps.append(&mut vars.iter().map(|(_, _, (_, e))| e).collect::<Vec<&AST::Exp>>())
            },
            // Vec<e>
            AST::Exp_::ModuleCall(_, _, _, sp!(_, es)) |
            AST::Exp_::Vector(_, _, sp!(_, es)) |
            AST::Exp_::ExpList(es) => {
                exps.append(&mut es.iter().collect::<Vec<&AST::Exp>>());
            },
            // (e1, e2)
            AST::Exp_::IfElse(_, e1, e2) |
            AST::Exp_::Mutate(e1, e2) |
            AST::Exp_::BinopExp(e1, _, e2) => {
                exps.push(e1);
                exps.push(e2);
            },
            // e
            AST::Exp_::Return(e) |
            AST::Exp_::While(_, e) |
            AST::Exp_::Loop(e) |
            AST::Exp_::Assign(_, e) |
            AST::Exp_::FieldMutate(_, e) |
            AST::Exp_::Abort(e) |
            AST::Exp_::Dereference(e) |
            AST::Exp_::UnaryExp(_, e) |
            AST::Exp_::Cast(e, _) |
            AST::Exp_::Annotate(e, _) => exps.push(e),
            // ()
            AST::Exp_::Use(_) |
            AST::Exp_::Constant(_, _) |
            AST::Exp_::DerefBorrow(_) |
            AST::Exp_::Borrow(_, _) |
            AST::Exp_::Value(_) |
            AST::Exp_::Move(_) |
            AST::Exp_::Copy(_) |
            AST::Exp_::Unit{ trailing: _ } |
            AST::Exp_::Break |
            AST::Exp_::Continue |
            AST::Exp_::Spec(_, _) | // sepc
            AST::Exp_::UnresolvedError => (),
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