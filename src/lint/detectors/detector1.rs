use std::ops::Deref;
use std::collections::HashSet;
use move_compiler::parser;

pub struct Detector1<'a> {
    context: &'a mut super::Context,
    ast: &'a super::Ast,
    meta: &'a mut super::Detector,
    should_detect_assert: bool,
}

impl<'a> Detector1<'a> {
    fn is_assert(exp: &parser::ast::Exp) -> bool {
        if let parser::ast::Exp_::Call(name_chain, is_native, _, args) = &exp.value {
            if let parser::ast::NameAccessChain_::One(name) = &name_chain.value {
                return *is_native && name.value.as_str() == "assert" && args.value.len() == 2;
            }
        }
        return false;
    }

    fn set_should_detect_assert(&mut self, cond: &bool, exp: Option<&parser::ast::Exp>) {
        if !self.should_detect_assert && *cond {
            if let Some(e) = exp {
                // 只要当前指令不是assert，则就应该检测当前检测项
                if !Self::is_assert(e) {
                    self.should_detect_assert = true;
                }
            } else {
                self.should_detect_assert = true;
            }
        }
    }

    fn new(context: &'a mut super::Context, ast: &'a super::Ast, detector: &'a mut super::Detector) -> Self {
        Self {
            context,
            ast,
            meta: detector,
            should_detect_assert: false,
        }
    }

    fn detect(&mut self) -> anyhow::Result<()> {
        for package_def in &self.ast.full_ast.parser.source_definitions {
            match &package_def.def {
                parser::ast::Definition::Address(def) => {
                    self.parser_address_def(def);
                },
                parser::ast::Definition::Module(def) => {
                    self.parser_module_def(def);
                },
                parser::ast::Definition::Script(_def) => (),
            }
        }
        anyhow::Ok(())
    }

    fn parser_address_def(&mut self, def: &parser::ast::AddressDefinition) {
        for member in &def.modules {
            self.parser_module_def(member);
        }
    }

    fn parser_module_def(&mut self, def: &parser::ast::ModuleDefinition) {
        for member in &def.members {
            match member {
                parser::ast::ModuleMember::Function(func) => {
                    self.parser_func(func);
                },
                _ => (),
            }
        }
    }

    fn parser_func(&mut self, func: &parser::ast::Function) {
        if func.signature.parameters.is_empty() {
            return;
        }
        self.should_detect_assert = false;
        let mut func_params = func.signature.parameters.iter().map(|(var, _)| {var.0.value.as_str()}).collect::<HashSet<&str>>();
        match &func.body.value {
            parser::ast::FunctionBody_::Defined(block) => {
                self.parser_func_block(block, &mut func_params, &true, &false);
            },
            parser::ast::FunctionBody_::Native => (),
        }
    }

    fn remove_var_in_params(var: &parser::ast::Bind, params: &mut HashSet<&str>) {
        match &var.value {
            parser::ast::Bind_::Var(v) => {
                params.remove(v.0.value.as_str());
            },
            parser::ast::Bind_::Unpack(_, _, vs) => {
                for (_, v) in vs {
                    Self::remove_var_in_params(v, params);
                }
            }
        }
    }

    fn parser_func_block(&mut self, block: &parser::ast::Sequence, params: &mut HashSet<&str>, is_func_body: &bool, in_assert: &bool) {
        if params.is_empty() {
            return;
        }
        // move中函数域，use语句只能放在域首
        // (use语句数组，函数语句数组，<option>函数中最后一个;的loc，<option>返回值语句)
        let (_, items, _, return_exp) = block;
        for item in items {
            if params.is_empty() {
                break;
            }
            match &item.value {
                parser::ast::SequenceItem_::Seq(exp) => {
                    self.set_should_detect_assert(is_func_body, Some(exp));
                    self.parser_func_exp(exp.deref(), params, in_assert);
                },
                // let b : t; let b;
                // let b : t = e; let b = e;
                parser::ast::SequenceItem_::Declare(vars, _) | parser::ast::SequenceItem_::Bind(vars, _, _) => {
                    self.set_should_detect_assert(is_func_body, None);
                    // 移除与参数名相同的变量
                    for var in &vars.value {
                        Self::remove_var_in_params(var, params);
                    }
                },
            }
        }
        if let Some(exp) = return_exp.deref() {
            if params.is_empty() {
                return;
            }
            self.set_should_detect_assert(is_func_body, Some(exp));
            self.parser_func_exp(exp, params, in_assert);
        }
    }

    fn parser_func_exp(&mut self, exp: &parser::ast::Exp, params: &mut HashSet<&str>, in_assert: &bool) {
        use parser::ast;
        let mut exps: Vec<&ast::Exp> = Vec::new();
        match &exp.value {
            ast::Exp_::Name(name_chain, _) => {
                if let ast::NameAccessChain_::One(name) = &name_chain.value {
                    if *in_assert && params.contains(name.value.as_str()) {
                        self.add_issue(&name.loc);
                    }
                }
            },
            ast::Exp_::Call(_, _, _, args) => {
                if Self::is_assert(exp) {  // assert
                    assert!(!*in_assert);  // assert中包含assert？
                    if self.should_detect_assert {
                        self.parser_func_exp(&args.value[0], params, &true);
                    }
                } else {
                    exps.append(&mut args.value.iter().collect::<Vec<&ast::Exp>>());
                }
            },
            ast::Exp_::Block(block) => {
                self.parser_func_block(block, &mut params.clone(), &false, in_assert)
            },

            ast::Exp_::Pack(_, _, vars) => {
                exps.append(&mut vars.iter().map(|(_, var)| var).collect::<Vec<&ast::Exp>>())
            }
            ast::Exp_::Vector(_, _, vars) => {
                exps.append(&mut vars.value.iter().map(|var| var).collect::<Vec<&ast::Exp>>())
            }
            ast::Exp_::IfElse(_, exp_ture, exp_false) => {
                exps.push(exp_ture);
                if let Some(e) = exp_false {
                    exps.push(e);
                }
            },
            ast::Exp_::ExpList(es) => {
                exps.append(&mut es.iter().collect::<Vec<&ast::Exp>>());
            }
            // (e1, e2)
            ast::Exp_::Assign(e1, e2) |
            ast::Exp_::BinopExp(e1, _, e2) => {
                exps.push(e1);
                exps.push(e2);
            }
            // option<e>
            ast::Exp_::Return(e) => {
                if let Some(e) = e {
                    exps.push(e);
                }
            },
            // e
            ast::Exp_::While(_, e) |
            ast::Exp_::Loop(e) |
            ast::Exp_::Abort(e) |
            ast::Exp_::Dereference(e) |
            ast::Exp_::UnaryExp(_, e) |
            ast::Exp_::Borrow(_, e) |
            ast::Exp_::Dot(e, _) |
            ast::Exp_::Cast(e, _) |
            ast::Exp_::Annotate(e, _) => exps.push(e),
            // ()
            ast::Exp_::Value(_) |
            ast::Exp_::Move(_) |
            ast::Exp_::Copy(_) |
            ast::Exp_::Lambda(_, _) | // spec only
            ast::Exp_::Quant(_, _, _, _, _) | // spec only
            ast::Exp_::Unit |
            ast::Exp_::Break |
            ast::Exp_::Continue |
            ast::Exp_::Index(_, _) | // spec only
            ast::Exp_::Spec(_) | // sepc
            ast::Exp_::UnresolvedError => (),
        }
        for e in exps {
            self.parser_func_exp(e, params, in_assert);
        }
    }

    fn add_issue(&mut self, loc: &move_ir_types::location::Loc) {
        self.context.issues.add(super::Issue::new(
            super::IssueInfo::from(&self.meta.info),
            super::IssueLoc::from(&self.ast, loc),
        ));
    }
}

impl<'a> super::AbstractDetector for Detector1<'a> {
    fn info() -> super::DetectorInfo {
        super::DetectorInfo {
            no: 1,
            wiki: String::from(""),
            title: String::from("参数校验可以放在首行"),
            verbose: String::from("5.8 Some assertions can be optimized：进行参数校验的assert没放开头，开头可以快速失败，省gas。"),
            level: super::DetectorLevel::Warning,
        }
    }

    fn detect(context: &mut super::Context, ast: &super::Ast, detector: &mut super::Detector) -> anyhow::Result<()> {
        Detector1::new(context, ast, detector).detect()
    }
}