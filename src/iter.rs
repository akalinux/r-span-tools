use crate::builder::IncDecCpCmp;
use crate::{
    BlanketIncDecCpCmp, DefaultValues, GetBeginEnd, Mrs, MrsP, RangeRelation,
    first_range_begin_end, last_range_begin_end, next_range_begin_end, otmo,
    previous_range_begin_end, range_bounds_to_values, range_relation,
};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;
use std::ops::RangeBounds;
use std::rc::Rc;

pub struct OverlapIter<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, L, X>
where
    L: Borrow<RefCell<Vec<R>>>,
    X: Borrow<C>,
{
    src: L,
    step: V,
    cmp: X,
    next: Option<Mrs<T>>,
    back: Option<Mrs<T>>,
    _marker: PhantomData<(T, R, C)>,
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, L, X> OverlapIter<T, V, C, R, L, X>
where
    L: Borrow<RefCell<Vec<R>>>,
    X: Borrow<C>,
{
    /// Creates a new [crate::OverlapIter] from the slice of R.
    pub fn new(src: L, step: V, cmp: X) -> Self {
        let next = otmo(first_range_begin_end(&*src.borrow().borrow(), cmp.borrow()));
        let back = otmo(last_range_begin_end(&*src.borrow().borrow(), cmp.borrow()));
        Self {
            src,
            step,
            cmp,
            next,
            back,
            _marker: PhantomData,
        }
    }
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, L, X> Iterator for OverlapIter<T, V, C, R, L, X>
where
    L: Borrow<RefCell<Vec<R>>>,
    X: Borrow<C>,
{
    type Item = Mrs<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut next = None;
        if let Some(n) = &self.next {
            match &self.back {
                Some(b) => match range_relation(n, b, self.cmp.borrow()) {
                    RangeRelation::Overlap => {
                        if let Some(begin) = self.cmp.borrow().inc(n.get_end(), &self.step) {
                            next = otmo(next_range_begin_end(
                                &begin,
                                &[MrsP { r: b }],
                                self.cmp.borrow(),
                            ));
                        }
                    }
                    RangeRelation::Before => {
                        if let Some(begin) = self.cmp.borrow().inc(n.get_end(), &self.step) {
                            next = otmo(next_range_begin_end(
                                &begin,
                                &*self.src.borrow().borrow(),
                                self.cmp.borrow(),
                            ));
                        }
                    }
                    RangeRelation::After => return None,
                },
                None => (),
            }
        }
        return mem::replace(&mut self.next, next);
    }
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, L, X> DoubleEndedIterator
    for OverlapIter<T, V, C, R, L, X>
where
    L: Borrow<RefCell<Vec<R>>>,
    X: Borrow<C>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut back = None;
        if let Some(b) = &self.back
            && let Some(n) = &self.next
        {
            match range_relation(b, n, self.cmp.borrow()) {
                RangeRelation::Overlap => {
                    if let Some(end) = self.cmp.borrow().dec(b.get_begin(), &self.step) {
                        back = otmo(previous_range_begin_end(
                            &end,
                            &[MrsP { r: n }],
                            self.cmp.borrow(),
                        ));
                    }
                }
                RangeRelation::After => {
                    if let Some(end) = self.cmp.borrow().dec(b.get_begin(), &self.step) {
                        back = otmo(previous_range_begin_end(
                            &end,
                            &*self.src.borrow().borrow(),
                            self.cmp.borrow(),
                        ));
                    }
                }
                RangeRelation::Before => return None,
            }
        }
        return mem::replace(&mut self.back, back);
    }
}

/// This object acts as a conversion tool for accumulating instances of [std::ops::RangeBounds] and converting them to an internal representation.
/// The Internal representation is used by the [std::iter::IntoIterator] instance that is created to find the most common intersections.
pub struct Accumulate<T, V, C: IncDecCpCmp<T, V>> {
    list: Vec<Mrs<T>>,
    step: V,
    rebound: V,
    cmp: C,
}

impl<T, V, C: IncDecCpCmp<T, V>> Accumulate<T, V, C> {
    pub fn new(list: Vec<Mrs<T>>, step: V, rebound: V, cmp: C) -> Self {
        Self {
            list,
            step,
            rebound,
            cmp,
        }
    }
}

impl<T, V, C: IncDecCpCmp<T, V>> Accumulate<T, V, C>
where
    BlanketIncDecCpCmp: DefaultValues<T, V>,
{
    pub fn defaults() -> Accumulate<T, V, BlanketIncDecCpCmp> {
        let cmp = BlanketIncDecCpCmp::new();
        return Accumulate::new(Vec::new(), cmp.default_step(), cmp.default_rebound(), cmp);
    }
}

impl<T, V, C: IncDecCpCmp<T, V>> Accumulate<T, V, C> {
    pub fn rebound(&self, r: &impl RangeBounds<T>) -> Option<(T, T)> {
        return range_bounds_to_values(r, &self.rebound, &self.cmp);
    }

    pub fn add_range(&mut self, r: &impl RangeBounds<T>) -> Option<(&Mrs<T>, usize)> {
        match self.rebound(r) {
            Some(src) => self.add_tuple(src),
            None => None,
        }
    }
    pub fn add_tuple(&mut self, src: (T, T)) -> Option<(&Mrs<T>, usize)> {
        let (a, z) = src;
        return self.add_mrs(Mrs::new(a, z));
    }

    pub fn add_mrs(&mut self, mrs: Mrs<T>) -> Option<(&Mrs<T>, usize)> {
        let (a, z) = mrs.to_tuple_ref();
        if self.cmp.is_invalid_set(a, z) {
            return None;
        }
        self.list.push(mrs);
        let id = self.list.len() - 1;
        return Some((&self.list[id], id));
    }

    pub fn get_rebound(&self) -> &V {
        return &self.rebound;
    }

    pub fn get_cmp(&self) -> &impl IncDecCpCmp<T, V> {
        return &self.cmp;
    }
}

impl<T, V, C: IncDecCpCmp<T, V>> IntoIterator for Accumulate<T, V, C> {
    type Item = Mrs<T>;

    type IntoIter = OverlapIter<T, V, C, Mrs<T>, Rc<RefCell<Vec<Mrs<T>>>>, Rc<C>>;

    fn into_iter(self) -> Self::IntoIter {
        return OverlapIter::new(
            Rc::new(RefCell::new(self.list)),
            self.step,
            Rc::new(self.cmp),
        );
    }
}
