use crate::ConsolidationOrder::{Forward, Reverse};
use crate::builder::IncDecCpCmp;
use crate::{
    AnyIncDecCpCmp, ConsolidateMrsP, CpCmp, DefaultValues, GetBeginEnd, GetBeginEndOption, MrsP,
    NumberIncDecCpCmp, RangeRelation, RiFactory, consolidate, first_range_begin_end,
    last_range_begin_end, next_range_begin_end, previous_range_begin_end, range_bounds_to_values,
    range_relation,
};

use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;
use std::ops::RangeInclusive;
use std::ops::{Add, RangeBounds, Sub};
use std::rc::Rc;

/// Represents the consolidation order.
#[derive(Clone, Copy)]
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

    /// Cheks if the next range would be weanted.
    /// Returns true if yes, false if no.
    pub fn wants_next<T>(&self, r: &RangeRelation<T>) -> bool {
        match r {
            RangeRelation::Last(_) | RangeRelation::Overlap(_) => return true,
            RangeRelation::After(_) => match self {
                Self::Forward => return false,
                Self::Reverse => return true,
            },
            RangeRelation::Before(_) => match self {
                Self::Forward => return true,
                Self::Reverse => return false,
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

    /// Returns a ref to the internal [CpCmp] instance.
    pub fn get_cmp(&self) -> &C {
        return &self.cmp;
    }
}

impl<T, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>, I: Iterator<Item = R>, C: CpCmp<T>>
    Consolidate<T, R, F, I, C>
{
    pub fn to_consolidate_checker(
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

impl<T, R: GetBeginEnd<T>, F: GetBeginEndOption<T, R>, I: Iterator<Item = R>, C: CpCmp<T>>
    ConsolidateChecker<T, R, F, I, C>
{
    /// Returns the internal [ConsolidationOrder].
    pub fn get_order(&self) -> ConsolidationOrder {
        return self.order;
    }

    /// Returns the [CpCmp] instance from the internal iterator.
    pub fn get_cmp(&self) -> &C {
        return self.iter.get_cmp();
    }
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

    /// Tries to copy the src ref range via the internals.
    /// Returns None if it fails.
    pub fn copy_range<U: GetBeginEnd<T>>(&self, src: &U) -> Option<R> {
        let a = self.cmp.cp(src.get_begin());
        let z = self.cmp.cp(src.get_end());
        return self.factory.factory(Some((a, z)));
    }

    /// Updates the internal column to the new [GetBeginEnd] instance.
    /// Returns [Result::Err]f the range is invalid or if the index point does not exist.
    pub fn update_column(&mut self, idx: usize, range: R) -> Result<(), &'static str> {
        if let Some(col) = self.src.get_mut(idx) {
            if self.cmp.is_invalid_set(range.get_begin(), range.get_end()) {
                return Err("Invalid Range");
            }
            *col = range;
            return Ok(());
        }
        return Err("No such Column");
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
pub struct ColumnStater<
    T,
    V,
    R: GetBeginEnd<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = R>,
    C: IncDecCpCmp<T, V>,
> {
    column: Option<usize>,
    checker: RefCell<ConsolidateChecker<T, R, F, I, C>>,
    rows: RefCell<
        Vec<Rc<Result<ConsolidateMrsP<T, R>, (&'static str, RangeRelation<(R, Vec<(usize, R)>)>)>>>,
    >,
    _t: PhantomData<(T, F, I, C, V)>,
}

impl<
    'r,
    T,
    V,
    R: GetBeginEnd<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = R>,
    C: IncDecCpCmp<T, V>,
> ColumnStater<T, V, R, F, I, C>
{
    pub fn new(
        mut checker: ConsolidateChecker<T, R, F, I, C>,
        isec: &mut Intersector<T, V, C, R, F>,
    ) -> Self {
        if let Some(res) = checker.next() {
            let mut column: Option<usize> = None;
            let rows = Vec::new();
            if let Ok(range) = &res
                && let Some((idx, _)) = isec.add_from_tuple_ref(range.to_tuple_ref())
            {
                column = Some(idx);
            }

            return Self::builder(column, checker, rows);
        } else {
            return Self::builder(None, checker, Vec::new());
        }
    }
    fn builder(
        column: Option<usize>,
        checker: ConsolidateChecker<T, R, F, I, C>,
        rows: Vec<
            Rc<Result<ConsolidateMrsP<T, R>, (&'static str, RangeRelation<(R, Vec<(usize, R)>)>)>>,
        >,
    ) -> Self {
        return Self {
            column: column,
            checker: RefCell::new(checker),
            rows: RefCell::new(rows),
            _t: PhantomData,
        };
    }

    pub fn get_rows<'a>(
        &mut self,
        intersection: &R,
        iter: OverlapIter<T, V, C, R, F>,
    ) -> Result<
        Vec<Rc<Result<ConsolidateMrsP<T, R>, (&'static str, RangeRelation<(R, Vec<(usize, R)>)>)>>>,
        &'static str,
    > {
        let mut results = Vec::new();
        let order = self.checker.borrow().get_order();
        if self.column.is_none() {
            return Err("No Column id");
        }

        let rows = self.rows.replace(Vec::new());
        let mut todo = false;
        let checker = self.checker.borrow();
        for r in rows.into_iter() {
            match r.as_ref() {
                Err((e, _)) => {
                    self.rows.borrow_mut().push(Rc::clone(&r));
                    return Err(e);
                }
                Ok(res) => {
                    let relation = range_relation(res, intersection, checker.get_cmp());
                    if order.wants_next(&relation) {
                        // we do not need to update our column!
                        self.rows.borrow_mut().push(Rc::clone(&r));
                        results.push(Rc::clone(&r));
                        // we need to update our column!
                        match order {
                            Forward => {
                                todo = checker.get_cmp().gt(intersection.get_end(), res.get_end())
                            }
                            Reverse => {
                                todo = checker
                                    .get_cmp()
                                    .lt(intersection.get_begin(), res.get_begin())
                            }
                        }
                    } // range is dropped here, if it is not in scope!
                }
            }
        }
        if todo {
            loop {
                let next = self.checker.borrow_mut().next();
                if next.is_none() {
                    return Ok(results);
                }
                let r: Result<ConsolidateMrsP<T, R>, (&str, RangeRelation<(R, Vec<(usize, R)>)>)> =
                    next.unwrap();

                match r {
                    Err(res) => {
                        let end = Err(res.0);
                        self.rows.borrow_mut().push(Rc::new(Err(res)));
                        return end;
                    }
                    Ok(res) => {
                        let relation = range_relation(&res, intersection, checker.get_cmp());
                        let target = iter.copy_range(&res);
                        if order.wants_next(&relation) {
                            let r = Rc::new(Ok(res));

                            // we do not need to update our column!
                            self.rows.borrow_mut().push(Rc::clone(&r));
                            results.push(Rc::clone(&r));
                            if let Ok(res) = r.as_ref() {
                                // we need to update our column!
                                match order {
                                    Forward => {
                                        if checker
                                            .get_cmp()
                                            .lt(intersection.get_end(), res.get_end())
                                        {
                                            break;
                                        }
                                    }
                                    Reverse => {
                                        if checker
                                            .get_cmp()
                                            .gt(intersection.get_begin(), res.get_begin())
                                        {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        return Ok(results);
    }

    pub fn status(&self) -> Result<(), &'static str> {
        if self.column.is_none() {
            return Err("No Column id");
        } else {
            for r in self.rows.borrow().iter().rev() {
                if let Err((msg, _)) = r.as_ref() {
                    return Err(msg);
                }
            }
        }
        return Ok(());
    }
    pub fn is_err(&self) -> bool {
        if self.column.is_none() {
            return true;
        } else {
            for r in self.rows.borrow().iter().rev() {
                if r.is_err() {
                    return true;
                }
            }
        }
        return false;
    }
}

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
