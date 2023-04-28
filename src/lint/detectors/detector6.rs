use move_compiler::typing::ast as AST4;
use super::utils::{self, visitor::Visitor};

pub struct Detector6<'a> {
    context: &'a mut super::Context,
    ast: &'a super::Ast,
    meta: &'a mut super::Detector,
    deprecated_funcs: std::collections::HashSet<String>,
}

impl<'a> Detector6<'a> {
    fn new(context: &'a mut super::Context, ast: &'a super::Ast, detector: &'a mut super::Detector) -> Self {
        // TODO: 补全已弃用函数集合
        // 弃用的函数集合
        let mut deprecated_funcs = std::collections::HashSet::new();
        deprecated_funcs.insert(format!("{}::NFT::register", utils::account::fmt_address_hex("0x1")));
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
                func.visit_pre(&mut |exp, _| {
                    match &exp.exp.value {
                        AST4::UnannotatedExp_::ModuleCall(call) => {
                            let account_address = &call.module.value.address.into_addr_bytes().into_inner();
                            let address = account_address.to_canonical_string();
                            let mname = call.module.value.module.to_string();
                            let fname = call.name.to_string();
                            if self.deprecated_funcs.contains(&format!("{address}::{mname}::{fname}")) {
                                self.add_issue(&exp.exp.loc, format!("{}::{mname}::{fname}", account_address.to_hex_literal()))
                            }
                        },
                        _ => (),
                    }
                });
            }
        }
        anyhow::Ok(())
    }

    fn add_issue(&mut self, loc: &move_ir_types::location::Loc, fname: String) {
        let description = format!("函数`{fname}`已经弃用，调用它可能导能导致逻辑错误");
        self.context.issues.add(super::Issue::new(
            super::IssueInfo::from(&self.meta.info).description(description),
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