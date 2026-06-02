use crate::builder::IncDecCpCmp;
use crate::{
    AnyIncDecCpCmp, ConsolidateMrsP, CpCmp, DefaultValues, GetBeginEnd, GetBeginEndOption, MrsP,
    NumberIncDecCpCmp, RangeRelation, RiFactory, consolidate, first_range_begin_end,
    last_range_begin_end, next_range_begin_end, previous_range_begin_end, range_bounds_to_values,
    range_relation,
};

use std::marker::PhantomData;
use std::mem;
use std::ops::RangeInclusive;
use std::ops::{Add, RangeBounds, Sub};

// Represents the consolidation order.
pub enum ConsolidationOrder {
    /// Flags an object stating data is expected in the order provided by [crate::sort_forward].
    Forward,

    /// Flags an object stating data is expected in the order provided by [crate::sort_reverse].
    Reverse,
}

impl ConsolidationOrder {
    /// Filters instances of [RangeRelation] for validity against the given [ConsolidationOrder].
    /// When an invalid direction is detected a None is returned.
    ///
    /// There are 2 valid directions for consolidation
    ///  - Forward: see [crate::sort_forward].
    ///  - Reverse: see [crate::sort_reverse]
    ///
    /// Invalid state for: [ConsolidationOrder::Forward]
    ///   - [RangeRelation::After] is not valid.
    ///
    /// Invalid states for: [ConsolidationOrder::Reverse]
    ///   - [RangeRelation::Before] is not valid.
    pub fn check_direction<T>(&self, state: &RangeRelation<T>) -> Result<(), &'static str> {
        match state {
            RangeRelation::Last(_) | RangeRelation::Overlap(_) => Ok(()),
            RangeRelation::After(_) => match self {
                Self::Forward => {
                    Err("Out of Forward Sequence, Expected: Before|Last|Overlap, got: After")
                }
                Self::Reverse => Ok(()),
            },
            RangeRelation::Before(_) => match self {
                Self::Forward => Ok(()),
                Self::Reverse => {
                    Err("Out of Forward Sequence, Expected: After|Last|Overlap, got: Before")
                }
            },
        }
    }
}
pub struct Consolidate<
    T,
    R: GetBeginEnd<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = R>,
    C: CpCmp<T>,
> {
    iter: I,
    last: Option<(R, Vec<(usize, R)>)>,
    cmp: C,
    facotry: F,
    offset: usize,
    _p: PhantomData<T>,
}
impl<T, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>, I: Iterator<Item = R>, C: CpCmp<T>>
    Consolidate<T, R, F, I, C>
{
    pub fn new(iter: I, cmp: C, factory: F) -> Self {
        return Self {
            iter,
            last: None,
            cmp: cmp,
            facotry: factory,
            offset: 0,
            _p: PhantomData,
        };
    }
}

impl<T, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>, I: Iterator<Item = R>, C: CpCmp<T>>
    Consolidate<T, R, F, I, C>
{
    pub fn to_consolidate_proxy(
        self,
        order: ConsolidationOrder,
    ) -> ConsolidateChecker<T, R, F, I, C> {
        return ConsolidateChecker {
            order,
            iter: self,
            _p: PhantomData,
        };
    }
}

pub struct ConsolidateChecker<
    T,
    R: GetBeginEnd<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = R>,
    C: CpCmp<T>,
> {
    order: ConsolidationOrder,
    iter: Consolidate<T, R, F, I, C>,
    _p: PhantomData<(T, R, F, C, I)>,
}

impl<T, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>, I: Iterator<Item = R>, C: CpCmp<T>> Iterator
    for ConsolidateChecker<T, R, F, I, C>
{
    type Item = Result<ConsolidateMrsP<T, R>, (&'static str, RangeRelation<(R, Vec<(usize, R)>)>)>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(r) = self.iter.next() {
            match self.order.check_direction(&r) {
                Ok(()) => {
                    let src = r.unwrap();
                    return Some(Ok(ConsolidateMrsP {
                        r: src.0,
                        src: src.1,
                        _t: PhantomData,
                    }));
                }
                Err(msg) => {
                    return Some(Err((msg, r)));
                }
            }
        }
        return None;
    }
}

impl<T, I: Iterator<Item = RangeInclusive<T>>>
    Consolidate<T, RangeInclusive<T>, RiFactory<T>, I, NumberIncDecCpCmp<T>>
where
    NumberIncDecCpCmp<T>: DefaultValues<T, T>,
    T: Copy + Clone,
{
    pub fn num(iter: I, cmp: NumberIncDecCpCmp<T>, factory: RiFactory<T>) -> Self {
        return Self::new(iter, cmp, factory);
    }
}

impl<T, I: Iterator<Item = RangeInclusive<T>>>
    Consolidate<T, RangeInclusive<T>, RiFactory<T>, I, NumberIncDecCpCmp<T>>
where
    NumberIncDecCpCmp<T>: DefaultValues<T, T>,
    T: Copy + Clone,
{
    pub fn num_defaults(iter: I) -> Self {
        let cmp = NumberIncDecCpCmp::<T>::defaults();
        let factory = RiFactory::<T>::new();
        return Self::num(iter, cmp, factory);
    }
}

impl<R: GetBeginEnd<T>, T, I: Iterator<Item = R>, F: GetBeginEndOption<T, R>>
    Consolidate<T, R, F, I, AnyIncDecCpCmp<T>>
where
    T: PartialOrd + Clone + Copy,
{
    pub fn any(iter: I, cmp: AnyIncDecCpCmp<T>, factory: F) -> Self {
        return Self::new(iter, cmp, factory);
    }
}

impl<T, I: Iterator<Item = RangeInclusive<T>>>
    Consolidate<T, RangeInclusive<T>, RiFactory<T>, I, AnyIncDecCpCmp<T>>
where
    T: PartialOrd + Clone + Copy + Add<T, Output = T> + Sub<T, Output = T>,
{
    pub fn any_defaults(iter: I, min: T, max: T) -> Self {
        return Self::any(iter, AnyIncDecCpCmp::new(min, max), RiFactory::new());
    }
}

impl<T, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>, I: Iterator<Item = R>, C: CpCmp<T>> Iterator
    for Consolidate<T, R, F, I, C>
{
    type Item = RangeRelation<(R, Vec<(usize, R)>)>;
    fn next(&mut self) -> Option<Self::Item> {
        let next;
        (self.offset, next) = consolidate(
            &mut self.last,
            &mut self.iter,
            &self.cmp,
            &self.facotry,
            self.offset,
        );

        return next;
    }
}
pub struct OverlapIter<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>> {
    src: Vec<R>,
    step: V,
    cmp: C,
    next: Option<R>,
    back: Option<R>,
    factory: F,
    _marker: PhantomData<(T, R)>,
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
            factory,
            _marker: PhantomData,
        }
    }
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>> Iterator
    for OverlapIter<T, V, C, R, F>
{
    type Item = R;
    fn next(&mut self) -> Option<Self::Item> {
        let mut next = None;
        if let Some(n) = &self.next {
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
        return mem::replace(&mut self.next, next);
    }
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>> DoubleEndedIterator
    for OverlapIter<T, V, C, R, F>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut back = None;
        if let Some(b) = &self.back
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
        return mem::replace(&mut self.back, back);
    }
}

/// This object acts as a conversion tool for accumulating instances of [std::ops::RangeBounds] and converting them to an internal representation.
/// The Internal representation is used by the [std::iter::IntoIterator] instance that is created to find the most common intersections.
/// This implementation is meant to be generic and can be easily tailored to any data type or structure that requires computing intersections.
pub struct Intersector<T, V, C: IncDecCpCmp<T, V>, R, B> {
    list: Vec<R>,
    step: V,
    rebound: V,
    cmp: C,
    factory: B,
    _r: PhantomData<(T, R)>,
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, B: GetBeginEndOption<T, R>>
    Intersector<T, V, C, R, B>
{
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

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, B: GetBeginEndOption<T, R>>
    Intersector<T, V, C, R, B>
{
    pub fn add_from_tuple(&mut self, src: (T, T)) -> Option<(usize, &R)> {
        if self.cmp.is_invalid_set(&src.0, &src.1) {
            return None;
        }
        match self.factory.factory(Some(src)) {
            Some(mrs) => {
                self.list.push(mrs);
                let id = self.list.len() - 1;
                return Some((id, &self.list[id]));
            }
            None => None,
        }
    }

    pub fn get_rebound(&self) -> &V {
        return &self.rebound;
    }

    pub fn get_step(&self) -> &V {
        return &self.step;
    }

    pub fn set_rebound(&mut self, rebound: V) {
        self.rebound = rebound;
    }

    pub fn set_step(&mut self, step: V) {
        self.step = step;
    }
    pub fn get_cmp(&self) -> &C {
        return &self.cmp;
    }

    pub fn get_cmp_mut(&mut self) -> &mut C {
        return &mut self.cmp;
    }

    pub fn rebound(&self, r: &impl RangeBounds<T>) -> Option<(T, T)> {
        return range_bounds_to_values(r, self.get_rebound(), self.get_cmp());
    }

    pub fn add_range(&mut self, r: &impl RangeBounds<T>) -> Option<(usize, &R)> {
        match self.rebound(r) {
            Some(src) => self.add_tuple(src),
            None => None,
        }
    }
    pub fn add_tuple(&mut self, src: (T, T)) -> Option<(usize, &R)> {
        return self.add_from_tuple(src);
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
