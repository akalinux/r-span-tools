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
    builder: C,
}

impl<T, V, C: IncDecCpCmpTrait<T, V>> Accumulate<T, V, C> {
    pub fn new(step: V, rebound: V, builder: C) -> Self {
        Self {
            list: Vec::new(),
            step,
            rebound,
            builder,
        }
    }

    pub fn add_range(&mut self, range: &impl RangeBounds<T>) -> bool {
        if let Some((a, z)) = range_bounds_to_values(range, &self.rebound, &self.builder) {
            let r = Mrs::new(a, z);
            self.list.push(r);
            return true;
        }
        return false;
    }

    pub fn consume(self) -> OwnedOverlapIter<T, V, C> {
        return OwnedOverlapIter::new(self.list, self.step, self.builder);
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

/*
pub struct BoxedIter<T, V, C: DefaultValues<T, V>> {
    data: Box<[Mrs<T>]>,
    next: Option<(T, T)>,
    cmp: C,
    _marker: PhantomData<V>,
}
pub struct IterBuilder<T>(Box<[Mrs<T>]>);

impl<T> IterBuilder<T> {
    pub fn new<V, C: DefaultValues<T, V>>(self) -> BoxedIter<T, V, C> {
        let src = self.0.as_ref();
    }
}
*/

#[cfg(test)]
mod tests {
    use crate::{
        BlanketIncDecCpCmp, DefaultValues, Intersector, Mrs, OverlapIter, OwnedOverlapIter,
    };

    #[test]
    fn iter_test() {
        let checkset = [
            (0, 2),
            (3, 3),
            (4, 5),
            (6, 6),
            (8, 11),
            (13, 15),
            (16, 19),
            (20, 22),
        ];

        let mut check = [
            Mrs::new(4, 5),
            Mrs::new(4, 6),
            Mrs::new(0, 3),
            Mrs::new(1, 2),
            // gap 1 is 7-7
            Mrs::new(8, 11),
            // gap 2 is 12-12
            Mrs::new(13, 22),
            Mrs::new(15, 19),
        ];
        let t = BlanketIncDecCpCmp::new();
        let iter = OverlapIter::new(&mut check, &1, &t);
        for (i, res) in iter.enumerate() {
            assert_eq!(res, checkset[i])
        }
    }

    #[test]
    fn owned_iter_test() {
        let checkset = [
            (0, 2),
            (3, 3),
            (4, 5),
            (6, 6),
            (8, 11),
            (13, 15),
            (16, 19),
            (20, 22),
        ];

        let check = vec![
            Mrs::new(4, 5),
            Mrs::new(4, 6),
            Mrs::new(0, 3),
            Mrs::new(1, 2),
            // gap 1 is 7-7
            Mrs::new(8, 11),
            // gap 2 is 12-12
            Mrs::new(13, 22),
            Mrs::new(15, 19),
        ];
        let t = BlanketIncDecCpCmp::new();
        let iter = OwnedOverlapIter::new(check, 1, t);
        for (i, res) in iter.enumerate() {
            assert_eq!(res, checkset[i])
        }
    }

    #[test]
    fn intersector_test() {
        let checkset = [
            (0, 2),
            (3, 3),
            (4, 5),
            (6, 6),
            (8, 11),
            (13, 15),
            (16, 19),
            (20, 22),
        ];

        let check = [4..=5, 4..=6, 0..=3, 1..=2, 8..=11, 13..=22, 15..=19];
        let t = BlanketIncDecCpCmp::new();
        let iter = Intersector::new(&check, t.default_step(), t.default_rebound(), t);
        for (i, res) in iter.enumerate() {
            assert_eq!(res, checkset[i])
        }
    }

    #[test]
    fn intersector_defaults_test() {
        let checkset = [
            (0, 2),
            (3, 3),
            (4, 5),
            (6, 6),
            (8, 11),
            (13, 15),
            (16, 19),
            (20, 22),
        ];

        let check = [4..=5, 4..=6, 0..=3, 1..=2, 8..=11, 13..=22, 15..=19];
        let iter = Intersector::defaults(&check);
        for (i, res) in iter.enumerate() {
            assert_eq!(res, checkset[i])
        }
    }
}
