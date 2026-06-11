use std::{cell::RefCell, mem, rc::Rc};
pub mod columns;

use crate::{
    ConsolidateChecker, ConsolidateMrsP, CpCmp, GetBeginEnd, GetBeginEndOption, IncDecCpCmp,
    Intersector, OverlapIter,
};

/// Represents a column in an instance of [OverlapIter] in which the [Column] itself is an [Iterator].
pub struct Column<
    T,
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = S>,
    C: CpCmp<T>,
> {
    col: Result<usize, &'static str>,
    checker: RefCell<ConsolidateChecker<T, R, S, F, I, C>>,
    rows: RefCell<Vec<Rc<ConsolidateMrsP<T, R, S>>>>,
    err: Result<(), &'static str>,
}

impl<
    T,
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = S>,
    C: CpCmp<T>,
> Column<T, R, S, F, I, C>
{
    fn update_iter<V, Q: GetBeginEnd<T>, X: GetBeginEndOption<T, Q>>(
        &mut self,
        iter: &mut OverlapIter<T, V, C, Q, X>,
        idx: usize,
        col: Q,
    ) -> bool
    where
        C: IncDecCpCmp<T, V>,
    {
        if let Err(e) = iter.update_column(idx, col) {
            self.err = Err(e);
            return false;
        }
        return true;
    }

    fn get_cmp<'r>(&self) -> &'r C {
        let checker = &*self.checker.borrow();
        return unsafe { mem::transmute(checker.get_cmp()) };
    }

    pub fn update_column<V, Q: GetBeginEnd<T>, X: GetBeginEndOption<T, Q>>(
        &mut self,
        last: &Q,
        iter: &mut OverlapIter<T, V, C, Q, X>,
        reset: bool,
    ) -> bool
    where
        C: IncDecCpCmp<T, V>,
    {
        let col: usize;
        match &self.col {
            Ok(idx) => col = *idx,
            Err(_) => return false,
        }

        // reset the column error state each time we process a column.
        self.err = Ok(());
        let order;
        let cmp = self.get_cmp();
        {
            let checker = self.checker.borrow();
            order = checker.get_order();
            if let Some(row) = self.rows.borrow().last() {
                if reset {
                    if order.is_beyond(row.as_ref(), last, cmp) {
                        return false;
                    }
                } else {
                    let (overlap, done) = order.check_position(row.as_ref(), last, cmp);

                    if overlap || done {
                        // if we get here then we overlap with the current set or we are beyond it!
                        return false;
                    }
                }
            }
        }

        loop {
            let nc = self.checker.borrow_mut().next();
            if let Some(next) = nc {
                match next {
                    Err((msg, row)) => {
                        self.rows.borrow_mut().clear();
                        let clone = iter.factory.new_range(cmp.cp_tpl_ref(row.to_tuple_ref()));
                        self.rows.borrow_mut().push(Rc::new(row));
                        self.err = Err(msg);
                        return self.update_iter(iter, col, clone);
                    }
                    Ok(row) => {
                        let rc = Rc::new(row);
                        let clone = Rc::clone(&rc);
                        self.rows.borrow_mut().push(rc);
                        let r = clone.as_ref();
                        if reset {
                            if order.is_beyond(r, last, cmp) {
                                let clone =
                                    iter.factory.new_range(cmp.cp_tpl_ref(r.to_tuple_ref()));
                                return self.update_iter(iter, col, clone);
                            }
                        } else if order.is_before(r, last, cmp) {
                            let clone = iter.factory.new_range(cmp.cp_tpl_ref(r.to_tuple_ref()));
                            return self.update_iter(iter, col, clone);
                        } else {
                            let (overlap, done) = order.check_position(r, last, cmp);
                            if overlap || done {
                                let clone =
                                    iter.factory.new_range(cmp.cp_tpl_ref(r.to_tuple_ref()));

                                return self.update_iter(iter, col, clone);
                            }
                        }
                    }
                }
            } else {
                return false;
            }
        }
    }

    pub fn filter_column<Q: GetBeginEnd<T>>(
        &self,
        next: &Q,
    ) -> Result<Vec<Rc<ConsolidateMrsP<T, R, S>>>, &'static str> {
        if let Err(e) = &self.col {
            return Err(e);
        } else if let Err(e) = &self.err {
            // make sure we can progress on this column instances by clearing the error.
            return Err(e);
        }
        let mut results = Vec::new();
        let checker = self.checker.borrow();

        let cmp = checker.get_cmp();
        let order = checker.get_order();
        let rows = self.rows.replace(Vec::new());
        for row in rows {
            let (overlap, done) = order.check_position(row.as_ref(), next, cmp);
            if overlap {
                results.push(Rc::clone(&row));
                self.rows.borrow_mut().push(row);
            } else if done {
                self.rows.borrow_mut().push(row);
            }
        }
        return Ok(results);
    }

    pub fn in_err(&self) -> bool {
        return self.col.is_err() || self.err.is_err();
    }

    pub fn get_column(&self) -> Result<usize, &'static str> {
        let col = self.col?;
        return Ok(col);
    }

    pub fn get_rows<'a>(&self) -> &'a Vec<Rc<ConsolidateMrsP<T, R, S>>> {
        let res = &*self.rows.borrow();
        return unsafe {
            mem::transmute::<
                &'_ Vec<Rc<ConsolidateMrsP<T, R, S>>>,
                &'a Vec<Rc<ConsolidateMrsP<T, R, S>>>,
            >(res)
        };
    }

    /// Unwraps the current object state into a tuple.  
    /// The resulting values from the returned tuple can be used to crate a new instance of [Column] with via the [Column::builder].
    pub fn to_inner(
        self,
    ) -> (
        Result<usize, &'static str>,
        Vec<Rc<ConsolidateMrsP<T, R, S>>>,
        ConsolidateChecker<T, R, S, F, I, C>,
    ) {
        return (self.col, self.rows.into_inner(), self.checker.into_inner());
    }
    pub fn new<V, Q: GetBeginEnd<T>, X: GetBeginEndOption<T, Q>>(
        isec: &mut Intersector<T, V, C, Q, X>,
        mut checker: ConsolidateChecker<T, R, S, F, I, C>,
    ) -> Result<Self, Self>
    where
        C: IncDecCpCmp<T, V>,
    {
        let mut col = Err("checker has no next()");
        let mut rows = Vec::new();

        if let Some(next) = checker.next() {
            match next {
                Ok(r) => {
                    let rc = Rc::new(r);
                    let c = Rc::clone(&rc);

                    rows.push(rc);
                    match isec.add_from_tuple_ref(c.as_ref().to_tuple_ref()) {
                        Some((idx, _)) => {
                            col = Ok(idx);
                            return Ok(Self::builder((col, checker, rows)));
                        }
                        None => {
                            col = Err("Failed to add column from: checker");
                            return Err(Self::builder((col, checker, rows)));
                        }
                    }
                }
                Err((e, r)) => {
                    col = Err(e);
                    rows.push(Rc::new(r));
                    return Err(Self::builder((col, checker, rows)));
                }
            }
        } else {
            return Err(Self::builder((col, checker, rows)));
        }
    }

    /// This method alllows construction of a new instance of [Column] bypassing the operations performed by new.
    pub fn builder(
        inner: (
            Result<usize, &'static str>,
            ConsolidateChecker<T, R, S, F, I, C>,
            Vec<Rc<ConsolidateMrsP<T, R, S>>>,
        ),
    ) -> Self {
        return Self {
            err: Ok(()),
            col: inner.0,
            checker: RefCell::new(inner.1),
            rows: RefCell::new(inner.2),
        };
    }
}
