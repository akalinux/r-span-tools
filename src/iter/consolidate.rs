pub mod checker;

use std::{
    marker::PhantomData,
    ops::{Add, RangeInclusive, Sub},
};

use crate::utils::consolidate;
use crate::{
    AnyIncDecCpCmp, CpCmp, DefaultValues, GetBeginEnd, GetBeginEndOption, NumberIncDecCpCmp,
    RangeRelation, RiFactory, iter::consolidate::checker::ConsolidateChecker,
};

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
    ///  - Forward: see [crate::sort_forward]
    ///  - Reverse: see [crate::sort_reverse]
    ///
    /// Invalid state for: [ConsolidationOrder::Forward]
    ///   - [RangeRelation::After] is not valid.
    ///   - [RangeRelation::Invalid] is not valid.
    ///
    /// Invalid states for: [ConsolidationOrder::Reverse]
    ///   - [RangeRelation::Before] is not valid.
    ///   - [RangeRelation::Invalid] is not valid.
    pub fn check_direction<T>(&self, state: &RangeRelation<T>) -> Result<(), &'static str> {
        match state {
            RangeRelation::Invalid(_) => Err("Range Compare contained Invalid range(s)"),
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
            RangeRelation::Invalid(_) => false,
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
    S: GetBeginEnd<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = S>,
    C: CpCmp<T>,
> {
    iter: I,
    last: Option<(R, Vec<(usize, S)>)>,
    cmp: C,
    facotry: F,
    offset: usize,
    _p: PhantomData<(T, S)>,
}
impl<
    T,
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = S>,
    C: CpCmp<T>,
> Consolidate<T, R, S, F, I, C>
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

impl<
    T,
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = S>,
    C: CpCmp<T>,
> Consolidate<T, R, S, F, I, C>
{
    pub fn to_consolidate_checker(
        self,
        order: ConsolidationOrder,
    ) -> ConsolidateChecker<T, R, S, F, I, C> {
        return ConsolidateChecker::new(order, self);
    }
}

impl<T, I: Iterator<Item = RangeInclusive<T>>>
    Consolidate<T, RangeInclusive<T>, RangeInclusive<T>, RiFactory<T>, I, NumberIncDecCpCmp<T>>
where
    NumberIncDecCpCmp<T>: DefaultValues<T, T>,
    T: Copy + Clone,
{
    pub fn num(iter: I, cmp: NumberIncDecCpCmp<T>, factory: RiFactory<T>) -> Self {
        return Self::new(iter, cmp, factory);
    }
}

impl<T, I: Iterator<Item = RangeInclusive<T>>>
    Consolidate<T, RangeInclusive<T>, RangeInclusive<T>, RiFactory<T>, I, NumberIncDecCpCmp<T>>
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

impl<R: GetBeginEnd<T>, S: GetBeginEnd<T>, T, I: Iterator<Item = S>, F: GetBeginEndOption<T, R>>
    Consolidate<T, R, S, F, I, AnyIncDecCpCmp<T>>
where
    T: PartialOrd + Clone + Copy,
{
    pub fn any(iter: I, cmp: AnyIncDecCpCmp<T>, factory: F) -> Self {
        return Self::new(iter, cmp, factory);
    }
}

impl<T, I: Iterator<Item = RangeInclusive<T>>>
    Consolidate<T, RangeInclusive<T>, RangeInclusive<T>, RiFactory<T>, I, AnyIncDecCpCmp<T>>
where
    T: PartialOrd + Clone + Copy + Add<T, Output = T> + Sub<T, Output = T>,
{
    pub fn any_defaults(iter: I, min: T, max: T) -> Self {
        return Self::any(iter, AnyIncDecCpCmp::new(min, max), RiFactory::new());
    }
}

impl<
    T,
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = S>,
    C: CpCmp<T>,
> Iterator for Consolidate<T, R, S, F, I, C>
{
    type Item = RangeRelation<(R, Vec<(usize, S)>)>;
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
