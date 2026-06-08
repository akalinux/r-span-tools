use std::{
    cell::RefCell,
    ops::{Add, RangeInclusive, Sub},
    rc::Rc,
};

use crate::{
    AnyIncDecCpCmp, Column, Consolidate, ConsolidateChecker, ConsolidateMrsP, ConsolidationOrder,
    CpCmp, DefaultValues, GetBeginEnd, GetBeginEndOption, IncDecCpCmp, Intersector,
    NumberIncDecCpCmp, OverlapIter, RiFactory,
};

pub struct Columns<
    T,
    V,
    I: Iterator<Item = S>,
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
    C: IncDecCpCmp<T, V>,
    F: GetBeginEndOption<T, R> + Copy + Clone,
> {
    isec: RefCell<Intersector<T, V, C, RangeInclusive<T>, RiFactory<T>>>,
    factory: F,
    columns: RefCell<Vec<Column<T, R, S, F, I, C>>>,
    cmp: C,
    order: ConsolidationOrder,
}

impl<S: GetBeginEnd<T>, T: Copy + Clone + PartialOrd, I: Iterator<Item = S>>
    Columns<T, T, I, RangeInclusive<T>, S, NumberIncDecCpCmp<T>, RiFactory<T>>
where
    NumberIncDecCpCmp<T>: DefaultValues<T, T>,
{
    pub fn num(order: ConsolidationOrder, step: T, rebound: T, min: T, max: T) -> Self {
        let cmp = NumberIncDecCpCmp::new(min, max);
        let factory = RiFactory::new();
        return Self::new(order, cmp, factory, step, rebound);
    }

    pub fn num_defaults() -> Self {
        let cmp = NumberIncDecCpCmp::defaults();
        return Self::num(
            ConsolidationOrder::Forward,
            cmp.default_step(),
            cmp.default_step(),
            cmp.min(),
            cmp.max(),
        );
    }

    pub fn num_defaults_rev() -> Self {
        let cmp = NumberIncDecCpCmp::defaults();
        return Self::num(
            ConsolidationOrder::Reverse,
            cmp.default_step(),
            cmp.default_step(),
            cmp.min(),
            cmp.max(),
        );
    }
}

impl<S: GetBeginEnd<T>, T, V, I: Iterator<Item = S>>
    Columns<T, V, I, RangeInclusive<T>, S, AnyIncDecCpCmp<T>, RiFactory<T>>
where
    V: Copy,
    T: PartialOrd + Copy + Add<V, Output = T> + Sub<V, Output = T>,
{
    pub fn any(order: ConsolidationOrder, cmp: AnyIncDecCpCmp<T>, step: V, rebound: V) -> Self {
        return Self::new(order, cmp, RiFactory::new(), step, rebound);
    }
}
impl<
    T,
    V,
    I: Iterator<Item = S>,
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
    C: IncDecCpCmp<T, V> + Copy + Clone,
    F: GetBeginEndOption<T, R> + Copy + Clone,
> Columns<T, V, I, R, S, C, F>
{
    pub fn new(order: ConsolidationOrder, cmp: C, factory: F, step: V, rebound: V) -> Self {
        return Self {
            isec: RefCell::new(Intersector::new(
                Vec::new(),
                step,
                rebound,
                cmp,
                RiFactory::new(),
            )),
            factory,
            columns: RefCell::new(Vec::new()),
            cmp,
            order,
        };
    }

    pub fn add_column(&self, iter: I) -> Result<usize, Column<T, R, S, F, I, C>> {
        let con = Consolidate::new(iter, self.cmp, self.factory);
        let checker = ConsolidateChecker::new(self.order, con);
        match Column::new(&mut *self.isec.borrow_mut(), checker) {
            Ok(res) => {
                let idx = res.get_column();
                match idx {
                    Ok(id) => {
                        self.columns.borrow_mut().push(res);
                        return Ok(id);
                    }
                    Err(_) => {
                        return Err(res);
                    }
                }
            }
            Err(res) => Err(res),
        }
    }
}

impl<
    T,
    V,
    I: Iterator<Item = S>,
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
    C: IncDecCpCmp<T, V> + Copy + Clone,
    F: GetBeginEndOption<T, R> + Copy + Clone,
> IntoIterator for Columns<T, V, I, R, S, C, F>
{
    type Item = (
        RangeInclusive<T>,
        Vec<Result<Vec<Rc<ConsolidateMrsP<T, R, S>>>, &'static str>>,
    );

    type IntoIter = ColumnsIter<T, V, I, R, S, C, F>;

    fn into_iter(self) -> Self::IntoIter {
        return ColumnsIter {
            order: self.order,
            iter: RefCell::new(self.isec.into_inner().into_iter()),
            cols: self.columns.into_inner(),
            needs_init: true,
        };
    }
}
pub struct ColumnsIter<
    T,
    V,
    I: Iterator<Item = S>,
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
    C: IncDecCpCmp<T, V>,
    F: GetBeginEndOption<T, R> + Copy + Clone,
> {
    iter: RefCell<OverlapIter<T, V, C, RangeInclusive<T>, RiFactory<T>>>,
    cols: Vec<Column<T, R, S, F, I, C>>,
    order: ConsolidationOrder,
    needs_init: bool,
}

impl<
    T,
    V,
    I: Iterator<Item = S>,
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
    C: IncDecCpCmp<T, V>,
    F: GetBeginEndOption<T, R> + Copy + Clone,
> Iterator for ColumnsIter<T, V, I, R, S, C, F>
{
    type Item = (
        RangeInclusive<T>,
        Vec<Result<Vec<Rc<ConsolidateMrsP<T, R, S>>>, &'static str>>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if self.needs_init {
            let next;
            // if we got here.. then the instance requires being initalized.
            self.needs_init = false;
            match &self.order {
                ConsolidationOrder::Forward => next = self.iter.borrow_mut().next(),
                ConsolidationOrder::Reverse => next = self.iter.borrow_mut().next_back(),
            }
            if next.is_none() {
                return None;
            }

            let filter = next.unwrap();
            let mut cols = Vec::new();
            for col in &mut self.cols {
                cols.push(col.filter_column(&filter))
            }
            return Some((filter, cols));
        }
        let next;
        let last;
        {
            let iter = &*self.iter.borrow();
            match &self.order {
                ConsolidationOrder::Forward => (next, last) = iter.ln(),
                ConsolidationOrder::Reverse => (next, last) = iter.lb(),
            }
        }
        let mut redo = false;
        let n;

        {
            let iter = &mut *self.iter.borrow_mut();
            if let Some(r) = &next {
                for col in &mut self.cols {
                    if col.update_column(r, iter, false) {
                        redo = true;
                    }
                }
            } else if let Some(r) = &last {
                redo = true;
                for col in &mut self.cols {
                    col.update_column(r, iter, true);
                }
            } else {
                return None;
            }
            if redo {
                match &self.order {
                    ConsolidationOrder::Forward => n = iter.recompute_next(),
                    ConsolidationOrder::Reverse => n = iter.recompute_back(),
                }
            } else {
                match &self.order {
                    ConsolidationOrder::Forward => n = iter.next(),
                    ConsolidationOrder::Reverse => n = iter.next_back(),
                }
            }
        }

        let mut cols = Vec::new();
        if next.is_none() {
            return None;
        }
        let filter = n.unwrap();
        for col in &mut self.cols {
            cols.push(col.filter_column(&filter))
        }

        return Some((filter, cols));
    }
}
