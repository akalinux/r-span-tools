use std::{
    cell::RefCell,
    ops::{Add, RangeInclusive, Sub},
    rc::Rc,
};

use crate::{
    AnyIncDecCpCmp, Column, Consolidate, ConsolidateChecker, ConsolidateMrsP, ConsolidationOrder,
    DefaultValues, GetBeginEnd, GetBeginEndOption, IncDecCpCmp, Intersector, NumberIncDecCpCmp,
    OverlapIter, RiFactory,
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
        let step = cmp.default_step();
        let rebound = cmp.default_rebound();
        let factory = RiFactory::new();
        return Self::new(ConsolidationOrder::Forward, cmp, factory, step, rebound);
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
        let next;
        let iter = &mut *self.iter.borrow_mut();
        match &self.order {
            ConsolidationOrder::Forward => next = iter.next(),
            ConsolidationOrder::Reverse => next = iter.next_back(),
        }
        if let Some(r) = next {
            let mut cols = Vec::new();

            for col in &mut self.cols {
                let result = col.update_column(&r, iter);
                cols.push(result);
            }
            return Some((r, cols));
        }
        return None;
    }
}
