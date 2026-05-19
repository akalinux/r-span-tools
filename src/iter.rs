use crate::{GetBeginEnd, IncDecCpCmpTrait, Mrs, first_range_begin_end, next_range_begin_end};
use std::cell::RefCell;
use std::mem;
use std::ops::{Bound, RangeBounds};

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
    pub fn new<S: RangeBounds<T>>(src: &[S], step: &'v V, cmp: &'c C) -> Self {
        let mut list: Vec<Mrs<T>> = Vec::new();

        for range in src {
            let a;
            let b;
            match range.start_bound() {
                Bound::Included(begin) => a = Some(cmp.cp(begin)),
                Bound::Excluded(begin) => a = cmp.dec(begin, step),
                Bound::Unbounded => a = Some(cmp.min()),
            }
            match range.end_bound() {
                Bound::Included(end) => b = Some(cmp.cp(end)),
                Bound::Excluded(end) => b = cmp.inc(end, step),
                Bound::Unbounded => b = Some(cmp.max()),
            }
            if let Some(a) = a
                && let Some(b) = b
            {
                if cmp.is_invalid_set(&a, &b) {
                    continue;
                }
                list.push(Mrs::new(a, b));
            }
        }
        let iter = OwnedMrsOverlapIter::new(list, step, cmp);

        Self { iter: iter }
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
