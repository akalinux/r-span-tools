use std::ops::RangeBounds;

use crate::{
    Consolidate, ConsolidateMrsP, ConsolidationOrder, GetBeginEnd, GetBeginEndOption, IncDecCpCmp,
    grow,
};
pub mod column;

pub struct ConsolidateChecker<
    T,
    V,
    R: GetBeginEnd<T>,
    S: RangeBounds<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = S>,
    C: IncDecCpCmp<T, V>,
> {
    order: ConsolidationOrder,
    iter: Consolidate<T, V, R, S, F, I, C>,
}

impl<
    T,
    V,
    R: GetBeginEnd<T>,
    S: RangeBounds<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = S>,
    C: IncDecCpCmp<T, V>,
> ConsolidateChecker<T, V, R, S, F, I, C>
{
    pub fn new(order: ConsolidationOrder, iter: Consolidate<T, V, R, S, F, I, C>) -> Self {
        return Self { order, iter };
    }
    /// Returns the internal [ConsolidationOrder].
    pub fn get_order(&self) -> ConsolidationOrder {
        return self.order;
    }

    /// Returns the [CpCmp] instance from the internal iterator.
    pub fn get_cmp(&self) -> &C {
        return self.iter.get_cmp();
    }
}

impl<
    T,
    V,
    R: GetBeginEnd<T>,
    S: RangeBounds<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = S>,
    C: IncDecCpCmp<T, V>,
> Iterator for ConsolidateChecker<T, V, R, S, F, I, C>
{
    type Item = Result<ConsolidateMrsP<T, R, S>, (&'static str, ConsolidateMrsP<T, R, S>)>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(r) = self.iter.next() {
            match self.order.check_direction(&r) {
                Ok(()) => {
                    return Some(Ok(ConsolidateMrsP::new(r.unwrap())));
                }
                Err(msg) => {
                    // it is not possible to get here.. without a next from the consolidation iterator.
                    let n = self.iter.next().unwrap();
                    // get our root cause
                    let cmp = self.iter.get_cmp();
                    let f = self.iter.get_factory();
                    let (first, mut dst) = r.unwrap();
                    let (second, mut src) = n.unwrap();
                    let r = grow(&first, &second, cmp, f);
                    dst.append(&mut src);
                    return Some(Err((msg, ConsolidateMrsP::new((r, dst)))));
                }
            }
        }
        return None;
    }
}
