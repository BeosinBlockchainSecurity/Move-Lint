use move_compiler::expansion::ast as AST2;
use move_compiler::typing::ast as AST4;

pub struct Detector4<'a> {
    context: &'a mut super::Context,
    ast: &'a super::Ast,
    meta: &'a mut super::Detector,
    private_funcs: std::collections::HashSet<(&'a AST2::ModuleIdent_, &'a move_symbol_pool::Symbol)>,
}

impl<'a> Detector4<'a> {
    fn new(context: &'a mut super::Context, ast: &'a super::Ast, detector: &'a mut super::Detector) -> Self {
        Self {
            context,
            ast,
            meta: detector,
            private_funcs: std::collections::HashSet::new(),
        }
    }

    fn detect(&mut self) -> anyhow::Result<()> {
        // 记录private函数
        for (_, module_ident, module) in &self.ast.full_ast.typing.modules {
            for (_, fname, func) in &module.functions {
                if func.visibility == AST2::Visibility::Internal  {
                    self.private_funcs.insert((module_ident, fname));
                }
            }
        }
        for (_, _, module) in &self.ast.full_ast.typing.modules {
            for (_, _, func) in &module.functions {
                self.parse_func(func);
            }
        }
        for (module_ident, fname) in self.private_funcs.clone() {
            if let Some(module) = self.ast.full_ast.typing.modules.get_(module_ident) {
                if let Some(loc) = module.functions.get_loc_(fname) {
                    self.add_issue(loc);
                }
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
        match &exp.exp.value {
            AST4::UnannotatedExp_::ModuleCall(call) => {
                let func_ident = (&call.module.value, &call.name.0.value);
                if self.private_funcs.contains(&func_ident) {
                    // 移除被调用的private函数
                    self.private_funcs.remove(&func_ident);
                }
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
            AST4::UnannotatedExp_::Mutate(e1, e2) |
            AST4::UnannotatedExp_::BinopExp(e1, _, _, e2) => {
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