use crate::{
    Consolidate, ConsolidateMrsP, ConsolidationOrder, CpCmp, GetBeginEnd, GetBeginEndOption,
    RangeRelation,
};
pub mod column;
use std::marker::PhantomData;

pub struct ConsolidateChecker<
    T,
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = S>,
    C: CpCmp<T>,
> {
    order: ConsolidationOrder,
    iter: Consolidate<T, R, S, F, I, C>,
}

impl<
    T,
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = S>,
    C: CpCmp<T>,
> ConsolidateChecker<T, R, S, F, I, C>
{
    pub fn new(order: ConsolidationOrder, iter: Consolidate<T, R, S, F, I, C>) -> Self {
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
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = S>,
    C: CpCmp<T>,
> Iterator for ConsolidateChecker<T, R, S, F, I, C>
{
    type Item =
        Result<ConsolidateMrsP<T, R, S>, (&'static str, RangeRelation<(R, Vec<(usize, S)>)>)>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(r) = self.iter.next() {
            if r.is_invalid() {
                return Some(Err(("Invalid Range Found in iterator", r)));
            } else {
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
        }
        return None;
    }
}
