use move_compiler::typing::ast as AST4;
use super::utils;

pub struct Detector6<'a> {
    context: &'a mut super::Context,
    ast: &'a super::Ast,
    meta: &'a mut super::Detector,
    deprecated_funcs: std::collections::HashSet<String>,
}

impl<'a> Detector6<'a> {
    fn new(context: &'a mut super::Context, ast: &'a super::Ast, detector: &'a mut super::Detector) -> Self {
        // 停用的函数集合
        let mut deprecated_funcs = std::collections::HashSet::new();
        // Todo: leocll，哪些函数已弃用
        deprecated_funcs.insert(format!("{}::NFT::register", utils::fmt_address_hex("0x1")));
        Self {
            context,
            ast,
            meta: detector,
            deprecated_funcs,
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
            AST4::UnannotatedExp_::ModuleCall(call) => {
                let account_address = &call.module.value.address.into_addr_bytes().into_inner();
                let address = account_address.to_canonical_string();
                let mname = call.module.value.module.to_string();
                let fname = call.name.to_string();
                if self.deprecated_funcs.contains(&format!("{address}::{mname}::{fname}")) {
                    self.add_issue(&exp.exp.loc, format!("{}::{mname}::{fname}", account_address.to_hex_literal()))
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
            AST4::UnannotatedExp_::BinopExp(e1, _, _, e2) |
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

    fn add_issue(&mut self, loc: &move_ir_types::location::Loc, fname: String) {
        let mut info = super::IssueInfo::from(&self.meta.info);
        info.description = Some(format!("函数`{fname}`已经弃用，调用它可能导能导致逻辑错误"));
        self.context.issues.add(super::Issue::new(
            info,
            super::IssueLoc::from(&self.ast, loc),
        ));
    }
}

impl<'a> super::AbstractDetector for Detector6<'a> {
    fn info() -> super::DetectorInfo {
        super::DetectorInfo {
            no: 6,
            wiki: String::from(""),
            title: String::from("调用了其他模块已经弃用的函数"),
            verbose: String::from("调用了其他模块已经弃用的函数，可能导能导致逻辑错误"),
            level: super::DetectorLevel::Info,
        }
    }

    fn detect(context: &mut super::Context, ast: &super::Ast, detector: &mut super::Detector) -> anyhow::Result<()> {
        Detector6::new(context, ast, detector).detect()
    }
}