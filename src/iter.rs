mod tests;
use crate::builder::IncDecCpCmpTrait;
use crate::{
    BlanketIncDecCpCmp, DefaultValues, GetBeginEnd, Mrs, first_range_begin_end,
    next_range_begin_end, range_bounds_to_values,
};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;
use std::ops::RangeBounds;

pub struct OverlapIter<'r, 'v, 'c, T, V, C: IncDecCpCmpTrait<T, V>, R: GetBeginEnd<T>> {
    src: &'r [R],
    step: &'v V,
    cmp: &'c C,
    next: Option<(T, T)>,
    _marker: PhantomData<(T, V)>,
}

impl<'r, 'v, 'c, T, V, C: IncDecCpCmpTrait<T, V>, R: GetBeginEnd<T>>
    OverlapIter<'r, 'v, 'c, T, V, C, R>
{
    pub fn new(src: &'r [R], step: &'v V, cmp: &'c C) -> Self {
        let next = first_range_begin_end(&*src, cmp);
        Self {
            src,
            step,
            cmp,
            next,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'r, 'v, 'c, T, V, C: IncDecCpCmpTrait<T, V>, R: GetBeginEnd<T>> Iterator
    for OverlapIter<'r, 'v, 'c, T, V, C, R>
{
    type Item = (T, T);
    fn next(&mut self) -> Option<Self::Item> {
        let mut next = None;
        if let Some((_, end)) = &self.next {
            if let Some(begin) = self.cmp.inc(end, self.step) {
                next = next_range_begin_end(&begin, self.src, self.cmp)
            }
        }

        return mem::replace(&mut self.next, next);
    }
}

pub struct OwnedOverlapIter<T, V, C: IncDecCpCmpTrait<T, V>> {
    cols: RefCell<Vec<Mrs<T>>>,
    step: V,
    cmp: C,
    next: Option<(T, T)>,
    _marker: std::marker::PhantomData<(T, V)>,
}

impl<T, V, C: IncDecCpCmpTrait<T, V>> OwnedOverlapIter<T, V, C> {
    pub fn new(cols: Vec<Mrs<T>>, step: V, cmp: C) -> Self {
        let next = first_range_begin_end(&*cols, &cmp);
        Self {
            cols: RefCell::new(cols),
            step,
            cmp,
            next,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn get_builder(&self) -> &C {
        return &self.cmp;
    }

    pub fn update_col(&mut self, idx: usize, range: Mrs<T>) -> Result<(), &'static str> {
        if let Some(col) = self.cols.get_mut().get_mut(idx) {
            *col = range;
            return Ok(());
        }
        return Err("idx out of bounds");
    }

    pub fn replcae_cols(&self, cols: Vec<Mrs<T>>) -> Vec<Mrs<T>> {
        return self.cols.replace(cols);
    }
}

impl<T, V, C: IncDecCpCmpTrait<T, V>> Iterator for OwnedOverlapIter<T, V, C> {
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        let mut target: Option<(T, T)> = None;
        if let Some((_, finish)) = &self.next {
            if let Some(begin) = self.cmp.inc(finish, &self.step) {
                target = next_range_begin_end(&begin, &self.cols.borrow().as_ref(), &self.cmp)
            }
        }
        return mem::replace(&mut self.next, target);
    }
}

pub struct Intersector<T, V, C: IncDecCpCmpTrait<T, V>> {
    iter: OwnedOverlapIter<T, V, C>,
}

impl<T, V, C: IncDecCpCmpTrait<T, V>> Intersector<T, V, C> {
    pub fn new<S: RangeBounds<T>>(src: &[S], step: V, rebound: V, cmp: C) -> Self {
        let mut list: Vec<Mrs<T>> = Vec::new();

        for range in src {
            if let Some((a, z)) = range_bounds_to_values(range, &rebound, &cmp) {
                list.push(Mrs::new(a, z));
            }
        }

        Self {
            iter: OwnedOverlapIter::new(list, step, cmp),
        }
    }
}

impl<T, V> Intersector<T, V, BlanketIncDecCpCmp>
where
    BlanketIncDecCpCmp: DefaultValues<T, V>,
{
    pub fn defaults<S: RangeBounds<T>>(src: &[S]) -> Intersector<T, V, BlanketIncDecCpCmp> {
        let t = BlanketIncDecCpCmp::new();
        return Intersector::new(src, t.default_step(), t.default_rebound(), t);
    }
}

impl<'v, 'c, T, V, C: IncDecCpCmpTrait<T, V>> Iterator for Intersector<T, V, C> {
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        return self.iter.next();
    }
}

pub struct Accumulate<T, V, C: IncDecCpCmpTrait<T, V>> {
    list: Vec<Mrs<T>>,
    step: V,
    rebound: V,
    cmp: C,
}

impl<T, V, C: IncDecCpCmpTrait<T, V>> Accumulate<T, V, C> {
    pub fn new(step: V, rebound: V, cmp: C) -> Self {
        Self {
            list: Vec::new(),
            step,
            rebound,
            cmp,
        }
    }

    pub fn add_range(&mut self, range: &impl RangeBounds<T>) -> bool {
        if let Some((a, z)) = range_bounds_to_values(range, &self.rebound, &self.cmp) {
            let r = Mrs::new(a, z);
            self.list.push(r);
            return true;
        }
        return false;
    }
}

impl<T, V, C: IncDecCpCmpTrait<T, V>> IntoIterator for Accumulate<T, V, C> {
    type Item = (T, T);

    type IntoIter = OwnedOverlapIter<T, V, C>;
    fn into_iter(self) -> Self::IntoIter {
        OwnedOverlapIter::new(self.list, self.step, self.cmp)
    }
}

impl<T, V> Accumulate<T, V, BlanketIncDecCpCmp>
where
    BlanketIncDecCpCmp: DefaultValues<T, V>,
{
    pub fn defaults() -> Self {
        let t = BlanketIncDecCpCmp::new();
        Accumulate::new(t.default_step(), t.default_rebound(), t)
    }
}

pub struct BoxedOverlapIter<'r, 'v, 'c, T, V, C: IncDecCpCmpTrait<T, V>> {
    _src: &'r [Mrs<T>],
    iter: OverlapIter<'r, 'v, 'c, T, V, C, Mrs<T>>,
}

impl<'r, 'v, 'c, T, V, C: IncDecCpCmpTrait<T, V>> BoxedOverlapIter<'r, 'v, 'c, T, V, C> {
    pub fn new(nv: &Vec<Mrs<T>>, step: &'v V, cmp: &'c C) -> BoxedOverlapIter<'r, 'v, 'c, T, V, C>
    where
        BlanketIncDecCpCmp: DefaultValues<T, V>,
        Mrs<T>: GetBeginEnd<T>,
    {
        let src: &'r [Mrs<T>] =
            unsafe { mem::transmute::<&'_ [Mrs<T>], &'r [Mrs<T>]>(nv.as_slice()) };
        let iter: OverlapIter<'r, 'v, 'c, T, V, C, Mrs<T>> = OverlapIter::new(src, step, cmp);

        BoxedOverlapIter { _src: src, iter }
    }
}

impl<'r, 'v, 'c, T, V, C: IncDecCpCmpTrait<T, V>> Iterator
    for BoxedOverlapIter<'r, 'v, 'c, T, V, C>
{
    type Item = (T, T);
    fn next(&mut self) -> Option<Self::Item> {
        return self.iter.next();
    }
}
