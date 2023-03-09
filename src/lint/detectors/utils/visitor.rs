/*
pub trait FunctionExpVisitor<T: ExpVisitor> {
    fn exps(&self) -> Vec<&T>;

    fn _visit_exp<F>(&self, visitor: &mut F, stop: &mut bool)
    where F: FnMut(bool, &T, &mut bool) {
        let exps = self.exps();
        for exp in exps {
            exp._visit(visitor, stop);
            if *stop {
                return;
            }
        }
    }

    fn visit_exp<F>(&self, visitor: &mut F)
    where F: FnMut(bool, &T, &mut bool) {
        self._visit_exp(visitor, &mut false);
    }

    fn visit_exp_pre<F>(&self, visitor: &mut F)
    where F: FnMut(&T, &mut bool) {
        self.visit_exp(&mut |post, e, stop| {
            if !post {
                visitor(e, stop);
            }
        });
    }

    fn visit_exp_post<F>(&self, visitor: &mut F)
    where F: FnMut(&T, &mut bool) {
        self.visit_exp(&mut |post, e, stop| {
            if post {
                visitor(e, stop);
            }
        });
    }
}

pub trait ExpVisitor {
    fn sub_exps(&self) -> Vec<&Self>;

    fn _visit<F>(&self, visitor: &mut F, stop: &mut bool)
    where F: FnMut(bool, &Self, &mut bool) {
        visitor(false, &self, stop);
        if *stop {
            return;
        }
        let exps = self.sub_exps();
        for exp in exps {
            exp._visit(visitor, stop);
            if *stop {
                return;
            }
        }
        visitor(true, &self, stop);
        if *stop {
            return;
        }
    }

    fn visit<F>(&self, visitor: &mut F)
    where F: FnMut(bool, &Self, &mut bool) {
        self._visit(visitor, &mut false);
    }

    fn visit_pre<F>(&self, visitor: &mut F)
    where F: FnMut(&Self, &mut bool) {
        self.visit(&mut |post, e, stop| {
            if !post {
                visitor(e, stop);
            }
        });
    }

    fn visit_post<F>(&self, visitor: &mut F)
    where F: FnMut(&Self, &mut bool) {
        self.visit(&mut |post, e, stop| {
            if post {
                visitor(e, stop);
            }
        });
    }
}
*/

pub trait Visitor<T: Visitor<T>> {
    fn items(&self) -> Vec<&T>;

    fn _visit_item<F>(item: &T, visitor: &mut F, stop: &mut bool)
    where F: FnMut(bool, &T, &mut bool) {
        visitor(false, item, stop);
        if !*stop {
            item._visit_items(visitor, stop);
            if !*stop {
                visitor(true, item, stop);
            }
        }
    }

    fn _visit_items<F>(&self, visitor: &mut F, stop: &mut bool)
    where F: FnMut(bool, &T, &mut bool) {
        for item in self.items() {
            Self::_visit_item(item, visitor, stop);
            if *stop {
                return;
            }
        }
    }

    fn visit<F>(&self, visitor: &mut F)
    where F: FnMut(bool, &T, &mut bool) {
        self._visit_items(visitor, &mut false);
    }

    fn visit_pre<F>(&self, visitor: &mut F)
    where F: FnMut(&T, &mut bool) {
        self.visit(&mut |post, e, stop| {
            if !post {
                visitor(e, stop);
            }
        });
    }

    fn visit_post<F>(&self, visitor: &mut F)
    where F: FnMut(&T, &mut bool) {
        self.visit(&mut |post, e, stop| {
            if post {
                visitor(e, stop);
            }
        });
    }
}
