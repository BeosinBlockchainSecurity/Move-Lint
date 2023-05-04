use std::collections::HashSet;
use move_compiler::typing::ast as AST4;

pub struct Detector1<'a> {
    context: &'a mut super::Context,
    ast: &'a super::Ast,
    meta: &'a mut super::Detector,
    top_exp_count: usize,
    top_assert_count: usize,
}

impl<'a> Detector1<'a> {
    fn new(context: &'a mut super::Context, ast: &'a super::Ast, detector: &'a mut super::Detector) -> Self {
        Self {
            context,
            ast,
            meta: detector,
            top_exp_count: 0,
            top_assert_count: 0,
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
        if func.signature.parameters.is_empty() {
            return;
        }
        let mut func_params = func.signature.parameters.iter().map(|(var, _)| {var.0.value.as_str()}).collect::<HashSet<&str>>();
        match &func.body.value {
            AST4::FunctionBody_::Defined(block) => {
                self.parse_func_block(block, &mut func_params, true, false);
            },
            AST4::FunctionBody_::Native => (),
        }
    }

    fn parse_func_block(&mut self, block: &AST4::Sequence, params: &mut HashSet<&str>, is_body: bool, in_assert: bool) {
        if params.is_empty() {
            return;
        }
        if is_body {
            self.top_exp_count = 0;
            self.top_assert_count = 0;
        }
        for seq in block {
            if is_body {
                self.top_exp_count += 1;
            }
            match &seq.value {
                AST4::SequenceItem_::Bind(lvalues, _, _) |
                AST4::SequenceItem_::Declare(lvalues) => {
                    for var in &lvalues.value {
                        Self::remove_var_in_params(var, params);
                    }
                },
                AST4::SequenceItem_::Seq(exp) => {
                    self.parse_func_exp(exp, params, is_body, in_assert);
                },
            }
        }
    }

    fn parse_func_exp(&mut self, exp: &AST4::Exp, params: &mut HashSet<&str>, is_top: bool, in_assert: bool) {
        if params.is_empty() {
            return;
        }
        let mut exps: Vec<&AST4::Exp> = Vec::new();
        let mut in_assert = in_assert;
        match &exp.exp.value {
            AST4::UnannotatedExp_::Assign(lvalues, _, e) => {
                for var in &lvalues.value {
                    Self::remove_var_in_params(var, params);
                }
                exps.push(e);
            },
            AST4::UnannotatedExp_::Builtin(func, e) => {
                if is_top {
                    self.top_assert_count += 1;
                }
                if let AST4::BuiltinFunction_::Assert(_) = func.value {
                    if self.top_exp_count != self.top_assert_count {
                        in_assert = true;
                    }
                }
                exps.push(e);
            },
            AST4::UnannotatedExp_::Use(var) |
            AST4::UnannotatedExp_::Move { from_user: _, var } |
            AST4::UnannotatedExp_::Copy { from_user: _, var } => {
                if in_assert && params.contains(var.0.value.as_str()) {
                    self.add_issue(&var.0.loc);
                }
            },
            AST4::UnannotatedExp_::Block(block) => {
                self.parse_func_block(block, &mut params.clone(), false, in_assert);
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
            AST4::UnannotatedExp_::Cast(e, _) |
            AST4::UnannotatedExp_::Vector(_, _, _, e) |
            AST4::UnannotatedExp_::Borrow(_, e, _) |
            AST4::UnannotatedExp_::TempBorrow(_, e) |
            AST4::UnannotatedExp_::Return(e) |
            AST4::UnannotatedExp_::Loop { has_break: _, body: e } |
            AST4::UnannotatedExp_::Abort(e) |
            AST4::UnannotatedExp_::Dereference(e) |
            AST4::UnannotatedExp_::UnaryExp(_, e) |
            AST4::UnannotatedExp_::Annotate(e, _) => exps.push(e),
            // ()
            AST4::UnannotatedExp_::BorrowLocal(_, _) |
            AST4::UnannotatedExp_::Constant(_, _) |
            AST4::UnannotatedExp_::Value(_) |
            AST4::UnannotatedExp_::Unit{ trailing: _ } |
            AST4::UnannotatedExp_::Break |
            AST4::UnannotatedExp_::Continue |
            AST4::UnannotatedExp_::Spec(_, _) | // sepc
            AST4::UnannotatedExp_::UnresolvedError => (),
        }
        for e in exps {
            self.parse_func_exp(e, params, false, in_assert);
        }
    }

    fn add_issue(&mut self, loc: &move_ir_types::location::Loc) {
        self.context.issues.add(super::Issue::new(
            super::IssueInfo::from(&self.meta.info),
            super::IssueLoc::from(&self.ast, loc),
        ));
    }

    fn remove_var_in_params(var: &AST4::LValue, params: &mut HashSet<&str>) {
        match &var.value {
            AST4::LValue_::Var(v, _) => {
                params.remove(v.0.value.as_str());
            },
            AST4::LValue_::Unpack(_, _, _, vs) |
            AST4::LValue_::BorrowUnpack(_, _, _, _, vs) => {
                for (_, _, (_, (_, v))) in vs {
                    Self::remove_var_in_params(v, params);
                }
            },
            AST4::LValue_::Ignore => (),
        }
    }
}

impl<'a> super::AbstractDetector for Detector1<'a> {
    fn info() -> super::DetectorInfo {
        super::DetectorInfo {
            no: 1,
            wiki: String::from(""),
            title: String::from("parameter validation can be placed in the first line"),
            verbose: String::from("Parameter validation with assertions can be placed at the beginning of functions. If failed, gas can be saved."),
            level: super::DetectorLevel::Warning,
        }
    }

    fn detect(context: &mut super::Context, ast: &super::Ast, detector: &mut super::Detector) -> anyhow::Result<()> {
        Detector1::new(context, ast, detector).detect()
    }
}