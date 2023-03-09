use move_compiler::typing::ast as AST4;

impl super::visitor::Visitor<AST4::Exp> for AST4::Function {
    fn items(&self) -> Vec<&AST4::Exp> {
        match &self.body.value {
            AST4::FunctionBody_::Defined(block) => {
                block.iter().filter_map(|item| {
                    match &item.value {
                        AST4::SequenceItem_::Seq(e) |
                        AST4::SequenceItem_::Bind(_, _, e) => Some(&**e),
                        AST4::SequenceItem_::Declare(_) => None,
                    }
                }).collect()
            },
            AST4::FunctionBody_::Native => vec![],
        }
    }
}

impl super::visitor::Visitor<AST4::Exp> for AST4::Exp {
    fn items(&self) -> Vec<&AST4::Exp> {
        match &self.exp.value {
            AST4::UnannotatedExp_::Block(block) => {
                block.iter().filter_map(|item| {
                    match &item.value {
                        AST4::SequenceItem_::Seq(e) |
                        AST4::SequenceItem_::Bind(_, _, e) => Some(&**e),
                        AST4::SequenceItem_::Declare(_) => None,
                    }
                }).collect()
            },
            AST4::UnannotatedExp_::Pack(_, _, _, vars) => {
                vars.iter().map(|(_, _, (_, (_, e)))| e).collect()
            },
            AST4::UnannotatedExp_::ModuleCall(call) => {
                vec![&call.arguments]
            }
            // Vec<e>
            AST4::UnannotatedExp_::ExpList(es) => {
                es.iter().map(|el| {
                    match el {
                        AST4::ExpListItem::Single(e, _) |
                        AST4::ExpListItem::Splat(_, e, _) => e
                    }
                }).collect()
            },
            // (e1, e2, e3)
            AST4::UnannotatedExp_::IfElse(e1, e2, e3) => {
                vec![e1, e2, e3]
            },
            // (e1, e2)
            AST4::UnannotatedExp_::While(e1, e2) |
            AST4::UnannotatedExp_::Mutate(e1, e2) |
            AST4::UnannotatedExp_::BinopExp(e1, _, _, e2) => {
                vec![e1, e2]
            },
            // e
            AST4::UnannotatedExp_::Builtin(_, e) |
            AST4::UnannotatedExp_::Assign(_, _, e) |
            AST4::UnannotatedExp_::Cast(e, _) |
            AST4::UnannotatedExp_::Vector(_, _, _, e) |
            AST4::UnannotatedExp_::Borrow(_, e, _) |
            AST4::UnannotatedExp_::TempBorrow(_, e) |
            AST4::UnannotatedExp_::Return(e) |
            AST4::UnannotatedExp_::Loop { has_break: _, body: e } |
            AST4::UnannotatedExp_::Abort(e) |
            AST4::UnannotatedExp_::Dereference(e) |
            AST4::UnannotatedExp_::UnaryExp(_, e) |
            AST4::UnannotatedExp_::Annotate(e, _) => vec![e],
            // ()
            AST4::UnannotatedExp_::Use(_) | // 貌似此节点在AST4貌似没用，被Move、Copy代替，待确认
            AST4::UnannotatedExp_::Move { from_user: _, var: _ } |
            AST4::UnannotatedExp_::Copy { from_user: _, var: _ } |
            AST4::UnannotatedExp_::BorrowLocal(_, _) |
            AST4::UnannotatedExp_::Constant(_, _) |
            AST4::UnannotatedExp_::Value(_) |
            AST4::UnannotatedExp_::Unit{ trailing: _ } |
            AST4::UnannotatedExp_::Break |
            AST4::UnannotatedExp_::Continue |
            AST4::UnannotatedExp_::Spec(_, _) | // sepc
            AST4::UnannotatedExp_::UnresolvedError => vec![],
        }
    }

    fn visit<F>(&self, visitor: &mut F)
    where F: FnMut(bool, &AST4::Exp, &mut bool) {
        Self::_visit_item(self, visitor, &mut false);    
    }
}
