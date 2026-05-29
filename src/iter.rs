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
                    RangeRelation::Overlap(_) => {
                        if let Some(begin) = self.cmp.borrow().inc(n.get_end(), &self.step) {
                            next = otmo(next_range_begin_end(
                                &begin,
                                &[MrsP { r: b }, MrsP { r: n }],
                                self.cmp.borrow(),
                            ));
                        }
                    }
                    RangeRelation::Before(_) => {
                        if let Some(begin) = self.cmp.borrow().inc(n.get_end(), &self.step) {
                            next = otmo(next_range_begin_end(
                                &begin,
                                &*self.src.borrow().borrow(),
                                self.cmp.borrow(),
                            ));
                        }
                    }
                    RangeRelation::After(_) => return None,
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
                RangeRelation::Overlap(_) => {
                    if let Some(end) = self.cmp.borrow().dec(b.get_begin(), &self.step) {
                        back = otmo(previous_range_begin_end(
                            &end,
                            &[MrsP { r: n }, MrsP { r: b }],
                            self.cmp.borrow(),
                        ));
                    }
                }
                RangeRelation::After(_) => {
                    if let Some(end) = self.cmp.borrow().dec(b.get_begin(), &self.step) {
                        back = otmo(previous_range_begin_end(
                            &end,
                            &*self.src.borrow().borrow(),
                            self.cmp.borrow(),
                        ));
                    }
                }
                RangeRelation::Before(_) => return None,
            }
        }
        return mem::replace(&mut self.back, back);
    }
}

/// This object acts as a conversion tool for accumulating instances of [std::ops::RangeBounds] and converting them to an internal representation.
/// The Internal representation is used by the [std::iter::IntoIterator] instance that is created to find the most common intersections.
/// This implementation is meant to be generic and can be easily tailored to any data type or structure that requires computing intersections.
pub struct Accumulate<T, V, C: IncDecCpCmp<T, V>> {
    list: Vec<Mrs<T>>,
    step: V,
    rebound: V,
    cmp: C,
}

/// This object acts as a conversion tool for accumulating instances of [std::ops::RangeBounds] and converting them to an internal representation.
/// The Internal representation is used by the [std::iter::IntoIterator] instance that is created to find the most common intersections.
/// Unlike [crate::Accumulate], [crate::AccumulateDefaults] works on a set of defaults that work for most primitive number types in rust.
pub struct AccumulateDefaults<T> {
    list: Vec<Mrs<T>>,
    step: T,
    rebound: T,
    cmp: BlanketIncDecCpCmp<T>,
}
impl<T> AccumulateDefaults<T>
where
    BlanketIncDecCpCmp<T>: DefaultValues<T, T>,
{
    pub fn new() -> Self {
        let cmp: BlanketIncDecCpCmp<T> = BlanketIncDecCpCmp::new();
        Self {
            list: Vec::new(),
            step: cmp.default_step(),
            rebound: cmp.default_rebound(),
            cmp,
        }
    }
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

pub trait Accumulator<T, V, C: IncDecCpCmp<T, V>> {
    type Cmp;
    fn get_rebound(&self) -> &V;
    fn get_step(&self) -> &V;
    fn add_mrs(&mut self, src: Mrs<T>) -> Option<(usize, &Mrs<T>)>;
    fn set_rebound(&mut self, rebound: V);
    fn set_step(&mut self, step: V);

    fn get_cmp(&self) -> &C;
    fn rebound(&self, r: &impl RangeBounds<T>) -> Option<(T, T)> {
        return range_bounds_to_values(r, self.get_rebound(), self.get_cmp());
    }

    fn add_range(&mut self, r: &impl RangeBounds<T>) -> Option<(usize, &Mrs<T>)> {
        match self.rebound(r) {
            Some(src) => self.add_tuple(src),
            None => None,
        }
    }
    fn add_tuple(&mut self, src: (T, T)) -> Option<(usize, &Mrs<T>)> {
        let (a, z) = src;
        return self.add_mrs(Mrs::new(a, z));
    }
}

impl<T, V, C: IncDecCpCmp<T, V>> Accumulator<T, V, C> for Accumulate<T, V, C> {
    type Cmp = C;
    fn add_mrs(&mut self, mrs: Mrs<T>) -> Option<(usize, &Mrs<T>)> {
        let (a, z) = mrs.to_tuple_ref();
        if self.cmp.is_invalid_set(a, z) {
            return None;
        }
        self.list.push(mrs);
        let id = self.list.len() - 1;
        return Some((id, &self.list[id]));
    }

    fn get_rebound(&self) -> &V {
        return &self.rebound;
    }

    fn get_step(&self) -> &V {
        return &self.step;
    }

    fn set_rebound(&mut self, rebound: V) {
        self.rebound = rebound;
    }

    fn set_step(&mut self, step: V) {
        self.step = step;
    }
    fn get_cmp(&self) -> &C {
        return &self.cmp;
    }
}

impl<T> Accumulator<T, T, BlanketIncDecCpCmp<T>> for AccumulateDefaults<T>
where
    BlanketIncDecCpCmp<T>: DefaultValues<T, T>,
{
    type Cmp = BlanketIncDecCpCmp<T>;

    fn add_mrs(&mut self, mrs: Mrs<T>) -> Option<(usize, &Mrs<T>)> {
        let (a, z) = mrs.to_tuple_ref();
        if IncDecCpCmp::is_invalid_set(&self.cmp, a, z) {
            return None;
        }
        self.list.push(mrs);
        let id = self.list.len() - 1;
        return Some((id, &self.list[id]));
    }

    fn get_rebound(&self) -> &T {
        return &self.rebound;
    }

    fn get_step(&self) -> &T {
        return &self.step;
    }

    fn set_rebound(&mut self, rebound: T) {
        self.rebound = rebound;
    }

    fn set_step(&mut self, step: T) {
        self.step = step;
    }
    fn get_cmp(&self) -> &BlanketIncDecCpCmp<T> {
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

impl<T> IntoIterator for AccumulateDefaults<T>
where
    BlanketIncDecCpCmp<T>: DefaultValues<T, T>,
{
    type Item = Mrs<T>;

    type IntoIter = OverlapIter<
        T,
        T,
        BlanketIncDecCpCmp<T>,
        Mrs<T>,
        Rc<RefCell<Vec<Mrs<T>>>>,
        Rc<BlanketIncDecCpCmp<T>>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        return OverlapIter::new(
            Rc::new(RefCell::new(self.list)),
            self.step,
            Rc::new(self.cmp),
        );
    }
}
