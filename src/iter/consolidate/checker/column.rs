use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::{
    ConsolidateChecker, ConsolidateMrsP, CpCmp, GetBeginEnd, GetBeginEndOption, IncDecCpCmp,
    Intersector, OverlapIter, range_relation,
};

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
    pub fn update_column<V, Q: GetBeginEnd<T>, X: GetBeginEndOption<T, Q>>(
        &mut self,
        pos: &Q,
        iter: &mut OverlapIter<T, V, C, Q, X>,
    ) -> Result<Vec<Rc<ConsolidateMrsP<T, R, S>>>, &'static str>
    where
        C: IncDecCpCmp<T, V>,
    {
        let col: usize;
        match &self.col {
            Err(e) => return Err(e),
            Ok(idx) => col = idx.clone(),
        }

        let rows = self.rows.replace(Vec::new());
        let mut next = self.rows.borrow_mut();
        let checker = self.checker.borrow();
        let order = checker.get_order();
        let cmp = checker.get_cmp();
        let mut results = Vec::new();
        for r in rows {
            let rel = range_relation(r.as_ref(), pos, cmp);
            if order.wants_next(&rel) {
                results.push(Rc::clone(&r));
                next.push(r);
            } else {
                // we are beyond the current intersection!
                next.push(r);
                return Ok(results);
            }
        }

        // if we got here, we need to start pulling from the checker.
        loop {
            let row = self.checker.borrow_mut().next();
            if let Some(res) = row {
                match res {
                    Err((msg, r)) => {
                        // Something went wrong with our source iter!!!

                        // clear all values.. they re not the cause of our failure!
                        results.clear();
                        let src = r.unwrap();

                        // save the message to the itnernals.
                        self.col = Err(msg);

                        // Shove our only suspect into next!
                        next.push(Rc::new(ConsolidateMrsP {
                            r: src.0,
                            src: src.1,
                            _t: PhantomData,
                        }));
                        // bail here!
                        return Err(msg);
                    }
                    Ok(r) => {
                        // if we got here, then we didn't get an error
                        let rel = range_relation(&r, pos, cmp);
                        let r = Rc::new(r);

                        if order.wants_next(&rel) {
                            results.push(Rc::clone(&r));
                            next.push(r);
                        } else {
                            next.push(r);

                            if let Err(e) = self.process_results(col, iter, &results) {
                                self.col = Err(e);
                                return Err(e);
                            }
                            return Ok(results);
                        }
                    }
                }
            } else {
                // No more results!

                if let Err(e) = self.process_results(col, iter, &results) {
                    self.col = Err(e);
                    return Err(e);
                }
                return Ok(results);
            }
        }
    }

    fn process_results<V, Q: GetBeginEnd<T>, X: GetBeginEndOption<T, Q>>(
        &self,
        col: usize,
        iter: &mut OverlapIter<T, V, C, Q, X>,
        results: &Vec<Rc<ConsolidateMrsP<T, R, S>>>,
    ) -> Result<(), &'static str>
    where
        C: IncDecCpCmp<T, V>,
    {
        if let Some(r) = results.last() {
            if let Some(r) = iter.copy_range(r.as_ref()) {
                if let Err(e) = iter.update_column(col, r) {
                    return Err(e);
                }
            }
        }
        return Ok(());
    }
    pub fn in_err(&self) -> bool {
        return self.col.is_err();
    }

    pub fn get_column(&self) -> Result<usize, &'static str> {
        match self.col {
            Err(e) => Err(e),
            Ok(idx) => Ok(idx),
        }
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
                            return Ok(Self::builder(col, checker, rows));
                        }
                        None => {
                            col = Err("Failed to add column from: checker");
                            return Err(Self::builder(col, checker, rows));
                        }
                    }
                }
                Err((e, d)) => {
                    col = Err(e);
                    let (r, src) = d.unwrap();
                    rows.push(Rc::new(ConsolidateMrsP {
                        r,
                        src,
                        _t: PhantomData,
                    }));
                    return Err(Self::builder(col, checker, rows));
                }
            }
        } else {
            return Err(Self::builder(col, checker, rows));
        }
    }

    /// This method alllows construction of a new instance of [Column] bypassing the operations performed by new.
    pub fn builder(
        col: Result<usize, &'static str>,
        checker: ConsolidateChecker<T, R, S, F, I, C>,
        rows: Vec<Rc<ConsolidateMrsP<T, R, S>>>,
    ) -> Self {
        return Self {
            col,
            checker: RefCell::new(checker),
            rows: RefCell::new(rows),
        };
    }
}
