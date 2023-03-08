use move_compiler::typing::ast as AST4;

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
                self.parse_func(func);
            }
        }
        anyhow::Ok(())
    }

    fn parse_func(&mut self, func: &AST4::Function) {
        match &func.body.value {
            AST4::FunctionBody_::Defined(block) => {
                self.parse_func_block(block);
            },
            AST4::FunctionBody_::Native => (),
        }
    }

    fn parse_func_block(&mut self, block: &AST4::Sequence) {
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

    fn parse_func_exp(&mut self, exp: &AST4::Exp) {
        let mut exps: Vec<&AST4::Exp> = Vec::new();
        match &exp.exp.value {
            AST4::UnannotatedExp_::Cast(e, typ) => {
                // e as typ
                if &e.ty.value == &typ.value {
                    self.add_issue(&exp.exp.loc);
                }
                exps.push(e);
            },
            AST4::UnannotatedExp_::Block(block) => {
                self.parse_func_block(block);
            },
            AST4::UnannotatedExp_::Pack(_, _, _, vars) => {
                exps.append(&mut vars.iter().map(|(_, _, (_, (_, e)))| e).collect::<Vec<&AST4::Exp>>())
            },
            AST4::UnannotatedExp_::ModuleCall(call) => {
                exps.push(&call.arguments);
            }
            // Vec<e>
            AST4::UnannotatedExp_::ExpList(es) => {
                for el in es {
                    match el {
                        AST4::ExpListItem::Single(e, _) |
                        AST4::ExpListItem::Splat(_, e, _) => exps.push(e)
                    }
                }
            },
            // (e1, e2, e3)
            AST4::UnannotatedExp_::IfElse(e1, e2, e3) => {
                exps.push(e1);
                exps.push(e2);
                exps.push(e3);
            },
            // (e1, e2)
            AST4::UnannotatedExp_::While(e1, e2) |
            AST4::UnannotatedExp_::Mutate(e1, e2) |
            AST4::UnannotatedExp_::BinopExp(e1, _, _, e2) => {
                exps.push(e1);
                exps.push(e2);
            },
            // e
            AST4::UnannotatedExp_::Builtin(_, e) |
            AST4::UnannotatedExp_::Vector(_, _, _, e) |
            AST4::UnannotatedExp_::Borrow(_, e, _) |
            AST4::UnannotatedExp_::TempBorrow(_, e) |
            AST4::UnannotatedExp_::Return(e) |
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

impl<'a> super::AbstractDetector for Detector3<'a> {
    fn info() -> super::DetectorInfo {
        super::DetectorInfo {
            no: 3,
            wiki: String::from(""),
            title: String::from("不必要的类型转换"),
            verbose: String::from("不必要的类型转换，例如：let a: u64; a as u64;"),
            level: super::DetectorLevel::Info,
        }
    }

    fn detect(context: &mut super::Context, ast: &super::Ast, detector: &mut super::Detector) -> anyhow::Result<()> {
        Detector3::new(context, ast, detector).detect()
    }
}