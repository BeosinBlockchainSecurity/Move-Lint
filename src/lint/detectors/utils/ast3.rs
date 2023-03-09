use move_compiler::naming::ast as AST3;
use move_ir_types::sp;

impl super::visitor::Visitor<AST3::Exp> for AST3::Function {
    fn items(&self) -> Vec<&AST3::Exp> {
        match &self.body.value {
            AST3::FunctionBody_::Defined(block) => {
                block.iter().filter_map(|item| {
                    match &item.value {
                        AST3::SequenceItem_::Seq(e) |
                        AST3::SequenceItem_::Bind(_, e) => Some(e),
                        AST3::SequenceItem_::Declare(_, _) => None,
                    }
                }).collect()
            },
            AST3::FunctionBody_::Native => vec![],
        }
    }
}

impl super::visitor::Visitor<AST3::Exp> for AST3::Exp {
    fn items(&self) -> Vec<&AST3::Exp> {
        match &self.value {
            AST3::Exp_::Block(block) => {
                block.iter().filter_map(|item| {
                    match &item.value {
                        AST3::SequenceItem_::Seq(e) |
                        AST3::SequenceItem_::Bind(_, e) => Some(e),
                        AST3::SequenceItem_::Declare(_, _) => None,
                    }
                }).collect()
            },
            AST3::Exp_::Pack(_, _, _, vars) => {
                vars.iter().map(|(_, _, (_, e))| e).collect()
            },
            // Vec<e>
            AST3::Exp_::Builtin(_, sp!(_, es)) |
            AST3::Exp_::ModuleCall(_, _, _, sp!(_, es)) |
            AST3::Exp_::Vector(_, _, sp!(_, es)) |
            AST3::Exp_::ExpList(es) => {
                es.iter().collect()
            },
            // (e1, e2, e3)
            AST3::Exp_::IfElse(e1, e2, e3) => {
                vec![e1, e2, e3]
            },
            // (e1, e2)
            AST3::Exp_::While(e1, e2) |
            AST3::Exp_::Mutate(e1, e2) |
            AST3::Exp_::BinopExp(e1, _, e2) => {
                vec![e1, e2]
            },
            // e
            AST3::Exp_::Return(e) |
            AST3::Exp_::Loop(e) |
            AST3::Exp_::Assign(_, e) |
            AST3::Exp_::FieldMutate(_, e) |
            AST3::Exp_::Abort(e) |
            AST3::Exp_::Dereference(e) |
            AST3::Exp_::UnaryExp(_, e) |
            AST3::Exp_::Cast(e, _) |
            AST3::Exp_::Annotate(e, _) => vec![e],
            // ()
            AST3::Exp_::Use(_) |
            AST3::Exp_::Constant(_, _) |
            AST3::Exp_::DerefBorrow(_) |
            AST3::Exp_::Borrow(_, _) |
            AST3::Exp_::Value(_) |
            AST3::Exp_::Move(_) |
            AST3::Exp_::Copy(_) |
            AST3::Exp_::Unit{ trailing: _ } |
            AST3::Exp_::Break |
            AST3::Exp_::Continue |
            AST3::Exp_::Spec(_, _) | // sepc
            AST3::Exp_::UnresolvedError => vec![],
        }
    }

    fn visit<F>(&self, visitor: &mut F)
    where F: FnMut(bool, &AST3::Exp, &mut bool) {
        Self::_visit_item(self, visitor, &mut false);    
    }
}
