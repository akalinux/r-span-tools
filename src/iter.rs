use crate::builder::IncDecCpCmp;
use crate::{
    AnyIncDecCpCmp, CpCmp, DefaultValues, GetBeginEnd, GetBeginEndOption, MrsP, NumberIncDecCpCmp,
    RangeRelation, RiFactory, consolidate, first_range_begin_end, last_range_begin_end,
    next_range_begin_end, previous_range_begin_end, range_bounds_to_values, range_relation,
};

use std::borrow::Borrow;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;
use std::ops::RangeInclusive;
use std::ops::{Add, RangeBounds, Sub};
use std::rc::Rc;

// Represents the consolidation order.
pub enum ConsolidationOrder {
    /// Flags an object stating data is expected in the order provided by [crate::sort_forward].
    Forward,

    /// Flags an object stating data is expected in the order provided by [crate::sort_reverse].
    Reverse,
}

impl ConsolidationOrder {
    /// Filters instances of [crate::RangeRelation] for validity against the given [crate::ConsolidationOrder].
    /// When an invalid direction is detected a None is returned.
    ///
    /// There are 2 valid directions for consolidation
    ///  - Forward: see [crate::sort_forward].
    ///  - Reverse: see [crate::sort_reverse]
    ///
    /// Invalid state for: [crate::ConsolidationOrder::Forward]
    ///   - [crate::RangeRelation::After] is not valid.
    ///
    /// Invalid states for: [crate::ConsolidationOrder::Reverse]
    ///   - [crate::RangeRelation::Before] is not valid.
    pub fn check_direction<T>(&self, state: Option<RangeRelation<T>>) -> Option<T> {
        if state.is_none() {
            return None;
        }
        match state.unwrap() {
            RangeRelation::Last(v) | RangeRelation::Overlap(v) => Some(v),
            RangeRelation::After(v) => match self {
                Self::Forward => None,
                Self::Reverse => Some(v),
            },
            RangeRelation::Before(v) => match self {
                Self::Forward => Some(v),
                Self::Reverse => None,
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
    X,
    Y,
> where
    X: Borrow<F>,
    Y: Borrow<C>,
{
    iter: I,
    last: Option<(R, Vec<(usize, R)>)>,
    cmp: Y,
    facotry: X,
    offset: usize,
    order: ConsolidationOrder,
    _p: PhantomData<(F, T, C)>,
}
impl<T, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>, I: Iterator<Item = R>, C: CpCmp<T>, X, Y>
    Consolidate<T, R, F, I, C, X, Y>
where
    X: Borrow<F>,
    Y: Borrow<C>,
{
    pub fn new(order: ConsolidationOrder, iter: I, cmp: Y, factory: X) -> Self {
        return Self {
            iter,
            last: None,
            cmp: cmp,
            facotry: factory,
            order,
            offset: 0,
            _p: PhantomData,
        };
    }
}
impl<Y, X, T, I: Iterator<Item = RangeInclusive<T>>>
    Consolidate<T, RangeInclusive<T>, RiFactory<T>, I, NumberIncDecCpCmp<T>, Y, X>
where
    NumberIncDecCpCmp<T>: DefaultValues<T, T>,
    X: Borrow<NumberIncDecCpCmp<T>>,
    Y: Borrow<RiFactory<T>>,
    T: Copy + Clone,
{
    pub fn num(order: ConsolidationOrder, iter: I, cmp: X, factory: Y) -> Self {
        return Self::new(order, iter, cmp, factory);
    }
}

impl<T, I: Iterator<Item = RangeInclusive<T>>>
    Consolidate<
        T,
        RangeInclusive<T>,
        RiFactory<T>,
        I,
        NumberIncDecCpCmp<T>,
        Rc<RiFactory<T>>,
        Rc<NumberIncDecCpCmp<T>>,
    >
where
    NumberIncDecCpCmp<T>: DefaultValues<T, T>,
    T: Copy + Clone,
{
    pub fn num_defaults(iter: I) -> Self {
        let cmp = Rc::new(NumberIncDecCpCmp::<T>::defaults());
        let factory = Rc::new(RiFactory::<T>::new());
        return Self::new(ConsolidationOrder::Forward, iter, cmp, factory);
    }
}

impl<Y, X, R: GetBeginEnd<T>, T, I: Iterator<Item = R>, F: GetBeginEndOption<T, R>>
    Consolidate<T, R, F, I, AnyIncDecCpCmp<T>, X, Y>
where
    T: PartialOrd + Clone + Copy,
    X: Borrow<F>,
    Y: Borrow<AnyIncDecCpCmp<T>>,
{
    pub fn any(order: ConsolidationOrder, iter: I, cmp: Y, factory: X) -> Self {
        return Self::new(order, iter, cmp, factory);
    }
}

impl<T, I: Iterator<Item = RangeInclusive<T>>>
    Consolidate<
        T,
        RangeInclusive<T>,
        RiFactory<T>,
        I,
        AnyIncDecCpCmp<T>,
        Rc<RiFactory<T>>,
        Rc<AnyIncDecCpCmp<T>>,
    >
where
    T: PartialOrd + Clone + Copy + Add<T, Output = T> + Sub<T, Output = T>,
{
    pub fn any_defaults(iter: I, min: T, max: T) -> Self {
        return Self::any(
            ConsolidationOrder::Forward,
            iter,
            Rc::new(AnyIncDecCpCmp::new(min, max)),
            Rc::new(RiFactory::new()),
        );
    }
}

impl<T, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>, I: Iterator<Item = R>, X, Y, C: CpCmp<T>>
    Iterator for Consolidate<T, R, F, I, C, X, Y>
where
    X: Borrow<F>,
    Y: Borrow<C>,
{
    type Item = (R, Vec<(usize, R)>);
    fn next(&mut self) -> Option<Self::Item> {
        let t = self.cmp.borrow();
        let f = self.facotry.borrow();
        let iter = &mut self.iter;
        let next;
        (self.offset, next) = consolidate(&mut self.last, iter, t, f, self.offset);

        return self.order.check_direction(next);
    }
}
pub struct OverlapIter<
    T,
    V,
    C: IncDecCpCmp<T, V>,
    R: GetBeginEnd<T>,
    L,
    X,
    B: GetBeginEndOption<T, R>,
    Y,
> where
    L: Borrow<RefCell<Vec<R>>>,
    X: Borrow<C>,
{
    src: L,
    step: V,
    cmp: X,
    next: Option<R>,
    back: Option<R>,
    factory: Y,
    _marker: PhantomData<(T, R, C, B)>,
}

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, L, X, B: GetBeginEndOption<T, R>, Y>
    OverlapIter<T, V, C, R, L, X, B, Y>
where
    L: Borrow<RefCell<Vec<R>>>,
    X: Borrow<C>,
    Y: Borrow<B>,
{
    /// Creates a new [crate::OverlapIter] from the slice of R.
    pub fn new(src: L, step: V, cmp: X, factory: Y) -> Self {
        let next = factory
            .borrow()
            .factory(first_range_begin_end(&*src.borrow().borrow(), cmp.borrow()));
        let back = factory
            .borrow()
            .factory(last_range_begin_end(&*src.borrow().borrow(), cmp.borrow()));
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

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, L, X, B: GetBeginEndOption<T, R>, Y> Iterator
    for OverlapIter<T, V, C, R, L, X, B, Y>
where
    L: Borrow<RefCell<Vec<R>>>,
    X: Borrow<C>,
    Y: Borrow<B>,
{
    type Item = R;
    fn next(&mut self) -> Option<Self::Item> {
        let mut next = None;
        if let Some(n) = &self.next {
            match &self.back {
                Some(b) => match range_relation(n, b, self.cmp.borrow()) {
                    RangeRelation::Overlap(_) => {
                        if let Some(begin) = self.cmp.borrow().inc(n.get_end(), &self.step) {
                            next = self.factory.borrow().factory(next_range_begin_end(
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
                                self.cmp.borrow(),
                            ));
                        }
                    }
                    RangeRelation::Before(_) => {
                        if let Some(begin) = self.cmp.borrow().inc(n.get_end(), &self.step) {
                            next = self.factory.borrow().factory(next_range_begin_end(
                                &begin,
                                &*self.src.borrow().borrow(),
                                self.cmp.borrow(),
                            ));
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

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, L, X, B: GetBeginEndOption<T, R>, Y>
    DoubleEndedIterator for OverlapIter<T, V, C, R, L, X, B, Y>
where
    L: Borrow<RefCell<Vec<R>>>,
    X: Borrow<C>,
    Y: Borrow<B>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut back = None;
        if let Some(b) = &self.back
            && let Some(n) = &self.next
        {
            match range_relation(b, n, self.cmp.borrow()) {
                RangeRelation::Overlap(_) => {
                    if let Some(end) = self.cmp.borrow().dec(b.get_begin(), &self.step) {
                        back = self.factory.borrow().factory(previous_range_begin_end(
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
                            self.cmp.borrow(),
                        ));
                    }
                }
                RangeRelation::After(_) => {
                    if let Some(end) = self.cmp.borrow().dec(b.get_begin(), &self.step) {
                        back = self.factory.borrow().factory(previous_range_begin_end(
                            &end,
                            &*self.src.borrow().borrow(),
                            self.cmp.borrow(),
                        ));
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

impl<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>, B: GetBeginEndOption<T, R>> IntoIterator
    for Intersector<T, V, C, R, B>
{
    type Item = R;

    type IntoIter = OverlapIter<T, V, C, R, Rc<RefCell<Vec<R>>>, Rc<C>, B, Rc<B>>;

    fn into_iter(self) -> Self::IntoIter {
        return OverlapIter::new(
            Rc::new(RefCell::new(self.list)),
            self.step,
            Rc::new(self.cmp),
            Rc::new(self.factory),
        );
    }
}
