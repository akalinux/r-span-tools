use crate::builder::IncDecCpCmpTrait;
use crate::{
    GetBeginEnd, Mrs, first_range_begin_end, next_range_begin_end, range_bounds_to_values,
};
use std::cell::RefCell;
use std::mem;
use std::ops::RangeBounds;

pub struct Intersector<'v, 'c, T, V, C: IncDecCpCmpTrait<T, V>> {
    iter: OwnedMrsOverlapIter<'v, 'c, T, V, C>,
}

impl<'v, 'c, T, V, C: IncDecCpCmpTrait<T, V>> Iterator for Intersector<'v, 'c, T, V, C> {
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        return self.iter.next();
    }
}

impl<'v, 'c, T, V, C: IncDecCpCmpTrait<T, V>> Intersector<'v, 'c, T, V, C> {
    pub fn new<S: RangeBounds<T>>(src: &[S], step: &'v V, rebound: &V, cmp: &'c C) -> Self {
        let mut list: Vec<Mrs<T>> = Vec::new();

        for range in src {
            let (a, z) = range_bounds_to_values(range, rebound, cmp);
            list.push(Mrs::new(a, z));
        }

        Self {
            iter: OwnedMrsOverlapIter::new(list, step, cmp),
        }
    }
}

pub struct OverlapIter<'r, 'v, 'c, T, V, C: IncDecCpCmpTrait<T, V>, R: GetBeginEnd<T>> {
    src: &'r mut [R],
    step: &'v V,
    cmp: &'c C,
    next: Option<(T, T)>,
    _marker: std::marker::PhantomData<(T, V)>,
}

impl<'r, 'v, 'c, T, V, C: IncDecCpCmpTrait<T, V>, R: GetBeginEnd<T>>
    OverlapIter<'r, 'v, 'c, T, V, C, R>
{
    pub fn new(src: &'r mut [R], step: &'v V, cmp: &'c C) -> Self {
        let next = first_range_begin_end(&*src, cmp);
        Self {
            src,
            step,
            cmp,
            next,
            _marker: std::marker::PhantomData,
        }
    }
    pub fn update_col(&mut self, idx: usize, range: R) -> Result<(), &'static str> {
        if let Some(col) = self.src.get_mut(idx) {
            *col = range;
            return Ok(());
        }
        return Err(&"idex out of bounds");
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

pub struct OwnedMrsOverlapIter<'v, 'c, T, V, C: IncDecCpCmpTrait<T, V>> {
    cols: RefCell<Vec<Mrs<T>>>,
    step: &'v V,
    cmp: &'c C,
    next: Option<(T, T)>,
    _marker: std::marker::PhantomData<(T, V)>,
}

impl<'v, 'c, T, V, C: IncDecCpCmpTrait<T, V>> OwnedMrsOverlapIter<'v, 'c, T, V, C> {
    pub fn new(cols: Vec<Mrs<T>>, step: &'v V, cmp: &'c C) -> Self {
        let next = first_range_begin_end(&*cols, cmp);
        Self {
            cols: RefCell::new(cols),
            step,
            cmp,
            next,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn update_col(&mut self, idx: usize, range: Mrs<T>) -> Result<(), &'static str> {
        if let Some(col) = self.cols.get_mut().get_mut(idx) {
            *col = range;
            return Ok(());
        }
        return Err("idx out of bounds");
    }
}

impl<'b, 'c, T, V, C: IncDecCpCmpTrait<T, V>> Iterator for OwnedMrsOverlapIter<'b, 'c, T, V, C> {
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        let mut target: Option<(T, T)> = None;
        if let Some((_, finish)) = &self.next {
            if let Some(begin) = self.cmp.inc(finish, self.step) {
                target = next_range_begin_end(&begin, &self.cols.borrow().as_ref(), self.cmp)
            }
        }
        return mem::replace(&mut self.next, target);
    }
}
