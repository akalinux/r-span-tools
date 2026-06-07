use crate::{
    AnyIncDecCpCmp, ConsolidationOrder, DefaultValues, GetBeginEnd, GetBeginEndOption, IncDecCpCmp,
    MrsP, NumberIncDecCpCmp, RangeRelation, RiFactory, first_range_begin_end, last_range_begin_end,
    next_range_begin_end, previous_range_begin_end, range_bounds_to_values, range_relation,
};

use std::marker::PhantomData;
use std::mem;
use std::ops::RangeInclusive;
use std::ops::{Add, RangeBounds, Sub};
pub mod consolidate;

pub struct OverlapIter<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>> {
    src: Vec<R>,
    step: V,
    cmp: C,
    next: Option<R>,
    back: Option<R>,
    last_next: Option<R>,
    last_back: Option<R>,
    factory: F,
    _marker: PhantomData<T>,
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>>
    OverlapIter<T, V, C, R, F>
{
    /// Creates a new [OverlapIter] from the slice of R.
    pub fn new(src: Vec<R>, step: V, cmp: C, factory: F) -> Self {
        let next = factory.factory(first_range_begin_end(&src, &cmp));
        let back = factory.factory(last_range_begin_end(&src, &cmp));
        Self {
            src,
            step,
            cmp,
            next,
            back,
            last_next: None,
            last_back: None,
            factory,
            _marker: PhantomData,
        }
    }

    /// Tries to copy the src ref range via the internals.
    /// Returns None if it fails.
    pub fn copy_range<U: GetBeginEnd<T>>(&self, src: &U) -> Option<R> {
        let a = self.cmp.cp(src.get_begin());
        let z = self.cmp.cp(src.get_end());
        return self.factory.factory(Some((a, z)));
    }

    pub fn potential_next(&self) -> (bool, Option<&R>) {
        if let Some(next) = &self.next {
            return (true, Some(next));
        } else if let Some(next) = &self.last_next {
            return (false, Some(next));
        }
        return (false, None);
    }

    pub fn potential_back(&self) -> (bool, Option<&R>) {
        if let Some(back) = &self.back {
            return (true, Some(back));
        } else if let Some(back) = &self.last_back {
            return (false, Some(back));
        }
        return (false, None);
    }

    /// Updates the internal column to the new [GetBeginEnd] instance.
    /// Returns [Result::Err]f the range is invalid or if the index point does not exist.
    pub fn update_column(
        &mut self,
        idx: usize,
        range: R,
        order: &ConsolidationOrder,
    ) -> Result<(), &'static str> {
        let t = &self.cmp;
        if let Some(col) = self.src.get_mut(idx) {
            if self.cmp.is_invalid_set(range.get_begin(), range.get_end()) {
                return Err("Invalid Range");
            }
            *col = range;
        } else {
            return Err("No such Column");
        }
        let col = &self.src[idx];
        // I suppose we are rewinding time here??
        match order {
            ConsolidationOrder::Forward => {
                self.last_back = None;
                if let Some(back) = &self.back {
                    if order.is_beyond(col, back, t) {
                        self.back = self.copy_range(col)
                    }
                } else {
                    self.back = self.copy_range(col);
                }

                if let Some(next) = &self.next {
                    if order.is_before(col, next, t) {
                        println!("trying column rewind");
                        self.next = self.try_next(&self.last_next);
                    } else {
                        println!("no need to rewind")
                    }
                } else {
                    if let Some(next) = self.try_next(&self.last_next) {
                        self.next = Some(next);
                    }
                }
            }
            ConsolidationOrder::Reverse => {
                self.last_next = None;
                if let Some(next) = &self.next {
                    if order.is_beyond(col, next, t) {
                        self.next = self.copy_range(col);
                    }
                } else {
                    self.next = self.copy_range(col)
                }

                if self.back.is_none() {
                    if let Some(back) = self.try_next_back(&self.last_back) {
                        self.last_back = None;
                        self.back = Some(back);
                    }
                }

                if let Some(back) = &self.back {
                    if order.is_before(col, back, t) {
                        self.back = self.try_next_back(&self.last_back);
                    }
                } else {
                    if let Some(back) = self.try_next_back(&self.last_back) {
                        self.back = Some(back);
                    }
                }
            }
        }
        return Ok(());
    }

    pub fn try_next(&self, src: &Option<R>) -> Option<R> {
        let mut next = None;
        if let Some(n) = src {
            match &self.back {
                Some(b) => match range_relation(n, b, &self.cmp) {
                    RangeRelation::Overlap(_) => {
                        if let Some(begin) = self.cmp.inc(n.get_end(), &self.step) {
                            next = self.factory.factory(next_range_begin_end(
                                &begin,
                                &[
                                    MrsP {
                                        r: b,
                                        _t: PhantomData,
                                    },
                                    MrsP {
                                        r: n,
                                        _t: PhantomData,
                                    },
                                ],
                                &self.cmp,
                            ));
                        }
                    }
                    RangeRelation::Before(_) => {
                        if let Some(begin) = self.cmp.inc(n.get_end(), &self.step) {
                            next = self
                                .factory
                                .factory(next_range_begin_end(&begin, &self.src, &self.cmp));
                        }
                    }
                    _ => return None,
                },
                None => (),
            }
        }
        return next;
    }

    pub fn try_next_back(&self, src: &Option<R>) -> Option<R> {
        let mut back = None;
        if let Some(b) = src
            && let Some(n) = &self.next
        {
            match range_relation(b, n, &self.cmp) {
                RangeRelation::Overlap(_) => {
                    if let Some(end) = self.cmp.dec(b.get_begin(), &self.step) {
                        back = self.factory.factory(previous_range_begin_end(
                            &end,
                            &[
                                MrsP {
                                    r: n,
                                    _t: PhantomData,
                                },
                                MrsP {
                                    r: b,
                                    _t: PhantomData,
                                },
                            ],
                            &self.cmp,
                        ));
                    }
                }
                RangeRelation::After(_) => {
                    if let Some(end) = self.cmp.dec(b.get_begin(), &self.step) {
                        back = self
                            .factory
                            .factory(previous_range_begin_end(&end, &self.src, &self.cmp));
                    }
                }
                _ => return None,
            }
        }
        return back;
    }
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>> Iterator
    for OverlapIter<T, V, C, R, F>
{
    type Item = R;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.try_next(&self.next);
        if let Some(next) = &self.next {
            self.last_next = self.copy_range(next);
        }
        return mem::replace(&mut self.next, next);
    }
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>> DoubleEndedIterator
    for OverlapIter<T, V, C, R, F>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let back = self.try_next_back(&self.back);
        if let Some(back) = &self.back {
            self.last_back = self.copy_range(back);
        }
        return mem::replace(&mut self.back, back);
    }
}

/// This acts as a general [OverlapIter] factory.
///
/// *The self.add_* methods*:
///
/// The various add_* methods return the index of the column that was added and the generated instance of [GetBeginEnd].
/// The index can be used to update that column during the iteration process
/// of the returned [OverlapIter] object instance.
/// See [OverlapIter::update_column] for more details.
pub struct Intersector<T, V, C: IncDecCpCmp<T, V>, R, F> {
    list: Vec<R>,
    step: V,
    rebound: V,
    cmp: C,
    factory: F,
    _r: PhantomData<(T, R)>,
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, B: GetBeginEndOption<T, R>>
    Intersector<T, V, C, R, B>
{
    /// Constructs a new instance of [Intersector].
    pub fn new(list: Vec<R>, step: V, rebound: V, cmp: C, factory: B) -> Self {
        Self {
            list,
            step,
            rebound,
            cmp,
            factory,
            _r: PhantomData,
        }
    }
}

impl<T, V> Intersector<T, V, AnyIncDecCpCmp<T>, RangeInclusive<T>, RiFactory<T>>
where
    T: PartialOrd + Copy + Add<V, Output = T> + Sub<V, Output = T>,
    V: Copy,
{
    pub fn any(
        step: V,
        rebound: V,
        min: T,
        max: T,
    ) -> Intersector<T, V, AnyIncDecCpCmp<T>, RangeInclusive<T>, RiFactory<T>> {
        Self {
            list: Vec::new(),
            step,
            rebound,
            cmp: AnyIncDecCpCmp::new(min, max),
            factory: RiFactory::new(),
            _r: PhantomData,
        }
    }
}

impl<T> Intersector<T, T, NumberIncDecCpCmp<T>, RangeInclusive<T>, RiFactory<T>>
where
    T: PartialOrd + Copy + Add<T, Output = T> + Sub<T, Output = T>,
    NumberIncDecCpCmp<T>: DefaultValues<T, T>,
{
    /// Returns a new instance of [Intersector] configured to work with numbers.
    pub fn num_defaults() -> Self {
        let cmp = NumberIncDecCpCmp::defaults();
        return Self {
            list: Vec::new(),
            step: cmp.default_step(),
            rebound: cmp.default_rebound(),
            cmp,
            factory: RiFactory::new(),
            _r: PhantomData,
        };
    }

    /// Returns a new instance of [Intersector] configured to work with numbers based on the arguments passed in.
    pub fn num(step: T, rebound: T, min: T, max: T) -> Self {
        return Self {
            list: Vec::new(),
            step,
            rebound,
            cmp: NumberIncDecCpCmp::new(min, max),
            factory: RiFactory::new(),
            _r: PhantomData,
        };
    }
}

macro_rules! impl_intersector_num_core{
    ($($t:ty),*) => {
        $(
            impl Intersector<$t, $t, NumberIncDecCpCmp<$t>, RangeInclusive<$t>,RiFactory<$t>>
            where NumberIncDecCpCmp<$t>: DefaultValues<$t,$t> {}

        )*
    };
}
impl_intersector_num_core!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
);

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>>
    Intersector<T, V, C, R, F>
{
    /// Tries to add an instance of [GetBeginEnd] to this instance returns None if src is invalid.
    pub fn add_raw_range(&mut self, src: R) -> Option<(usize, &R)> {
        if self.cmp.is_invalid_set(&src.get_begin(), &src.get_end()) {
            return None;
        }
        self.list.push(src);
        let id = self.list.len() - 1;
        return Some((id, &self.list[id]));
    }

    /// Tries to create and add a valid internal range from the tuple of refs.
    pub fn add_from_tuple_ref(&mut self, src: (&T, &T)) -> Option<(usize, &R)> {
        let a = self.cmp.cp(src.0);
        let z = self.cmp.cp(src.1);
        return self.add_from_tuple((a, z));
    }
    /// Tries to add a tuple to the instance, returns None if it fails.
    pub fn add_from_tuple(&mut self, src: (T, T)) -> Option<(usize, &R)> {
        match self.factory.factory(Some(src)) {
            Some(mrs) => return self.add_raw_range(mrs),
            None => None,
        }
    }

    /// This is really a wrapper for [crate::range_bounds_to_values].
    pub fn rebound(&self, r: &impl RangeBounds<T>) -> Option<(T, T)> {
        return range_bounds_to_values(r, self.get_rebound(), self.get_cmp());
    }

    /// Tries to convert a given [RangeBounds] instance to the internal range type.
    /// Returns None if the conversion process fails or the range produced is invalid.
    pub fn add_range(&mut self, r: &impl RangeBounds<T>) -> Option<(usize, &R)> {
        match self.rebound(r) {
            Some(src) => self.add_tuple(src),
            None => None,
        }
    }

    /// Tries to convert a tuple to the internal range type and add it.
    /// Returns None if the conversion process fails or the resulting range is invalid.
    pub fn add_tuple(&mut self, src: (T, T)) -> Option<(usize, &R)> {
        return self.add_from_tuple(src);
    }

    /// Returns a mutable ref to the internal instance of [IncDecCpCmp].
    pub fn get_cmp_mut(&mut self) -> &mut C {
        return &mut self.cmp;
    }

    /// Returns a ref to the internal instance of [IncDecCpCmp].
    pub fn get_cmp(&self) -> &C {
        return &self.cmp;
    }

    /// Returns a ref to the internal rebound value.
    pub fn get_rebound(&self) -> &V {
        return &self.rebound;
    }

    /// Returns a ref to the internal step value.
    pub fn get_step(&self) -> &V {
        return &self.step;
    }

    /// Updates the internal rebound value.
    pub fn set_rebound(&mut self, rebound: V) {
        self.rebound = rebound;
    }

    /// Updates the internal step value.
    pub fn set_step(&mut self, step: V) {
        self.step = step;
    }
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>> IntoIterator
    for Intersector<T, V, C, R, F>
{
    type Item = R;

    type IntoIter = OverlapIter<T, V, C, R, F>;

    fn into_iter(self) -> Self::IntoIter {
        return OverlapIter::new(self.list, self.step, self.cmp, self.factory);
    }
}
