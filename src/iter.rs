use crate::builder::IncDecCpCmp;
use crate::{
    BlanketIncDecCpCmp, DefaultValues, GetBeginEnd, Mrs, MrsP, RangeRelation,
    first_range_begin_end, last_range_begin_end, next_range_begin_end, otmo,
    previous_range_begin_end, range_bounds_to_values, range_relation,
};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;
use std::ops::RangeBounds;

pub struct OverlapIter<'r, 'v, 'c, T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>> {
    src: &'r [R],
    step: &'v V,
    cmp: &'c C,
    next: Option<Mrs<T>>,
    back: Option<Mrs<T>>,
    _marker: PhantomData<(T, V)>,
}

impl<'r, 'v, 'c, T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>
    OverlapIter<'r, 'v, 'c, T, V, C, R>
{
    /// Creates a new [crate::OverlapIter] from the slice of R.
    /// If the any of the objects passed into the constructor are modified durring the lifetime of the
    /// iteraotr then the behavior is undefined!
    pub fn new(src: &'r [R], step: &'v V, cmp: &'c C) -> Self {
        let next = otmo(first_range_begin_end(src, cmp));
        let back = otmo(last_range_begin_end(src, cmp));
        Self {
            src,
            step,
            cmp,
            next,
            back,
            _marker: PhantomData,
        }
    }

    /// Creates a new [crate::OverlapIter] from the [Vec] by creating a slice of the Vec<R>.  
    /// If the any of the objects passed into the constructor are modified durring the lifetime of the
    /// iteraotr then the behavior is undefined!
    pub fn from_vec(list: &Vec<R>, step: &'v V, cmp: &'c C) -> Self {
        let src = unsafe { mem::transmute::<&'_ [R], &'r [R]>(list.as_slice()) };
        let next = otmo(first_range_begin_end(src, cmp));
        let back = otmo(last_range_begin_end(src, cmp));
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

impl<'r, 'v, 'c, T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>> Iterator
    for OverlapIter<'r, 'v, 'c, T, V, C, R>
{
    type Item = Mrs<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut next = None;
        if let Some(src) = &self.next {
            match &self.back {
                Some(back) => match range_relation(src, back, self.cmp) {
                    RangeRelation::Overlap => {
                        if let Some(begin) = self.cmp.inc(src.get_end(), self.step) {
                            next = otmo(next_range_begin_end(&begin, &[MrsP { r: back }], self.cmp))
                        }
                    }
                    RangeRelation::Before => {
                        if let Some(begin) = self.cmp.inc(src.get_end(), self.step) {
                            next = otmo(next_range_begin_end(&begin, self.src, self.cmp))
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

impl<'r, 'v, 'c, T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>> DoubleEndedIterator
    for OverlapIter<'r, 'v, 'c, T, V, C, R>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut back = None;
        if let Some(src) = &self.back {
            match &self.next {
                Some(next) => match range_relation(src, next, self.cmp) {
                    RangeRelation::Overlap => {
                        if let Some(end) = self.cmp.dec(src.get_begin(), self.step) {
                            back = otmo(previous_range_begin_end(
                                &end,
                                &[MrsP { r: next }],
                                self.cmp,
                            ))
                        }
                    }
                    RangeRelation::After => {
                        if let Some(end) = self.cmp.dec(src.get_begin(), self.step) {
                            back = otmo(previous_range_begin_end(&end, self.src, self.cmp))
                        }
                    }
                    RangeRelation::Before => return None,
                },
                None => (),
            }
        }

        return mem::replace(&mut self.back, back);
    }
}
pub struct OwnedOverlapIter<T, V, C: IncDecCpCmp<T, V>> {
    cols: RefCell<Vec<Mrs<T>>>,
    step: V,
    cmp: C,
    next: Option<(T, T)>,
    _marker: std::marker::PhantomData<(T, V)>,
}

impl<T, V, C: IncDecCpCmp<T, V>> OwnedOverlapIter<T, V, C> {
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

impl<T, V, C: IncDecCpCmp<T, V>> Iterator for OwnedOverlapIter<T, V, C> {
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

pub struct Intersector<T, V, C: IncDecCpCmp<T, V>> {
    iter: OwnedOverlapIter<T, V, C>,
}

impl<T, V, C: IncDecCpCmp<T, V>> Intersector<T, V, C> {
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

impl<'v, 'c, T, V, C: IncDecCpCmp<T, V>> Iterator for Intersector<T, V, C> {
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        return self.iter.next();
    }
}

pub struct Accumulate<T, V, C: IncDecCpCmp<T, V>> {
    list: Vec<Mrs<T>>,
    step: V,
    rebound: V,
    cmp: C,
}

impl<T, V, C: IncDecCpCmp<T, V>> Accumulate<T, V, C> {
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

    pub fn add_ranges<R: RangeBounds<T>>(
        &mut self,
        ranges: &[R],
        on_add: impl Fn(usize, bool) -> bool,
    ) {
        for (i, r) in ranges.iter().enumerate() {
            if !on_add(i, self.add_range(r)) {
                break;
            }
        }
    }
}

impl<T, V, C: IncDecCpCmp<T, V>> IntoIterator for Accumulate<T, V, C> {
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
