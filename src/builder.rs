use crate::GetBeginEnd;
use std::{cmp::Ordering, ops::Bound};
mod tests;

/// This enum is used to represent positional relationships in 3 states
///  - before a range
///  - overlap with a range
///  - after a range
pub enum RangeRelation<T> {
    /// Range a is before range b
    Before,
    /// Range a and b overlap
    Overlap(T),
    /// Range a is after range b
    After,
}

/// The **Blanket Implementation** of [crate::IncDecCpCmp].  
///
/// Acts as the general proxy layer for creating and comparing values  of ranges.
/// Note that incrementing and decrementing are of 2 differnt types, but do not have to be.
///
/// The following types implemented for [crate::BlanketIncDecCpCmp]
///
///  - Unsigned Int: u8, u16, u32, u64, u128, usize
///  - Signed Int: i8, i16, i32, i64, i128, isize);
///  - Float: f32, f64
pub struct BlanketIncDecCpCmp {}

impl BlanketIncDecCpCmp {
    /// General constructor.
    pub fn new() -> Self {
        Self {}
    }
}

/// **Increment, Decrement, Copy, Compare Values and Ranges**
///
/// This is the base trait used to represent range manipulation by [crate].  
/// In general this library implements the proxy or wrapper approach to range and value manipulation as apposed to
/// manipulation by value.  This means the trait implementation for values does not need to be implemented on the generic type.
/// Since the operations are not implemented by the values, this frees us from the worry of conflicting trait resoltion.   This
/// also means we can quickly implement how the values and ranges are manipualted for the same data for a different task.
///
/// Example Implementation
///
/// ```
/// use common_range_tools::IncDecCpCmp;
///
/// struct MyTrait {}
/// impl IncDecCpCmp<i32,i32> for MyTrait {
///
///     fn dec(&self, a: &i32, b:&i32) ->Option<i32> {
///         if *b<=0 { return None}
///         return a.clone().checked_sub(b.clone());
///     }
///
///     fn min(&self) ->i32 { <i32>::MIN }
///     fn max(&self) ->i32 { <i32>::MAX }
///
///     fn inc(&self, a: &i32, b: &i32) -> Option<i32> {
///         if *b<=0 { return None}
///         return a.clone().checked_add(b.clone())
///     }
///
///     fn cp(&self,v: &i32) ->i32 { return v.clone() }
///
///     fn lt(&self,a:&i32,b: &i32) ->bool {  return a<b  }
/// }
/// ```
///
pub trait IncDecCpCmp<T, V> {
    //. Should return a clone or copy of &T.
    fn cp(&self, v: &T) -> T;

    /// Should safely increment a by b.  The value should always go up.. if not then it should return None.
    fn inc(&self, a: &T, b: &V) -> Option<T>;

    /// Should safely decrement a by b.  The value should always go down... if not then it should return None.
    fn dec(&self, a: &T, b: &V) -> Option<T>;

    /// Returns true if a < b.
    fn lt(&self, a: &T, b: &T) -> bool;

    /// Returns the minimum begin value we will accept when converting from [`std::ops::Bound::Unbounded`].
    fn min(&self) -> T;

    /// Returns the maximum end value we will accept when converting from [`std::ops::Bound::Unbounded`].
    fn max(&self) -> T;

    /// Returns true if a gt b.
    fn gt(&self, a: &T, b: &T) -> bool {
        return self.lt(b, a);
    }

    /// Returns true if a eq b.
    fn eq(&self, a: &T, b: &T) -> bool {
        return !self.lt(a, b) && !self.lt(b, a);
    }

    /// Returns true if a ne b.
    fn ne(&self, a: &T, b: &T) -> bool {
        return self.lt(a, b) || self.lt(b, a);
    }

    /// Returns true if a le b.
    fn le(&self, a: &T, b: &T) -> bool {
        return self.lt(a, b) || !self.lt(b, a);
    }

    /// Returns true if a ge b.
    fn ge(&self, a: &T, b: &T) -> bool {
        return self.lt(b, a) || !self.lt(a, b);
    }

    // Returns true if a and b contain c.
    fn contains(&self, a: &T, b: &T, c: &T) -> bool {
        return !(self.lt(c, a) || self.lt(b, c));
    }

    /// Returns true if any of the following are true
    /// - a and b contain c
    /// - a and b contain d
    /// - c and d contain a
    /// - c and d contain b
    fn overlap(&self, a: &T, b: &T, c: &T, d: &T) -> bool {
        return self.contains(a, b, c)
            || self.contains(a, b, d)
            || self.contains(c, d, a)
            || self.contains(c, d, b);
    }

    /// Compares range a and b and returns the correct [std::cmp::Ordering] value.
    ///
    /// The sort order is meant to represent consolidation order not tradtional range sort order.
    /// Consolidation order is represented as earliest largest ranges first.
    ///
    /// Put another way:
    /// - GetBeginEnd.get_begin() asc
    /// - GetBeginEnd.get_end() desc
    ///
    fn sort_forward<R: GetBeginEnd<T>>(&self, a: &R, b: &R) -> Ordering {
        if self.lt(b.get_begin(), a.get_begin()) {
            return Ordering::Greater;
        } else if self.lt(a.get_begin(), b.get_begin()) {
            return Ordering::Less;

        // anything below this point both begin values are the same
        } else if self.lt(a.get_end(), b.get_end()) {
            return Ordering::Greater;
        } else if self.lt(b.get_end(), a.get_end()) {
            return Ordering::Less;
        }
        // if we get here, begin and end are equal
        return Ordering::Equal;
    }

    /// Returns true if b lt a.
    fn is_invalid_set(&self, a: &T, b: &T) -> bool {
        return self.lt(b, a);
    }

    /// Returns true if the [crate::GetBeginEnd] contains an invalid set.
    fn is_invalid_range<R: GetBeginEnd<T>>(&self, check: &R) -> bool {
        return self.is_invalid_set(check.get_begin(), check.get_end());
    }

    /// Compares the positional relationship between a and b.
    ///
    /// - [`crate::RangeRelation::Before`] a is before b.
    /// - [`crate::RangeRelation::After`] a is after b.
    /// - [`crate::RangeRelation::Overlap`] a and b overlap to some degree.
    fn range_relation<R: GetBeginEnd<T>>(&self, a: &R, b: &R) -> RangeRelation<()> {
        if self.lt(a.get_end(), b.get_begin()) {
            return RangeRelation::Before;
        } else if self.lt(b.get_end(), a.get_begin()) {
            return RangeRelation::After;
        }

        return RangeRelation::Overlap(());
    }

    /// Returns the raw adjusted start value.
    ///   - [std::ops::Bound::Unbounded] becomes self.min()
    ///   - [std::ops::Bound::Included] value is not changed
    ///   - [std::ops::Bound::Excluded] value is incremented
    fn rebound_start(&self, start: Bound<&T>, rebound: &V) -> Option<T> {
        match start {
            Bound::Included(begin) => Some(self.cp(begin)),
            Bound::Excluded(begin) => self.inc(begin, rebound),
            Bound::Unbounded => Some(self.min()),
        }
    }

    /// Returns the raw adjusted end value.
    ///   - [std::ops::Bound::Unbounded] becomes
    ///   - [std::ops::Bound::Included] value is not changed
    ///   - [std::ops::Bound::Excluded] value is decremented
    fn rebound_end(&self, end: Bound<&T>, rebound: &V) -> Option<T> {
        match end {
            Bound::Included(end) => Some(self.cp(end)),
            Bound::Excluded(end) => self.dec(end, rebound),
            Bound::Unbounded => Some(self.max()),
        }
    }
}

/// **Default values**
///
/// Implemenations of this trait drive the internals used for [crate::iter::Intersector::defaults].
pub trait DefaultValues<T, V>: IncDecCpCmp<T, V> {
    /// Returns the default value use for progressing a begin or end value of a range.
    fn default_step(&self) -> V;

    /// Returns the value used to adjust a start or end value in the context of [std::ops::range::Bound::Excluded].
    fn default_rebound(&self) -> V;
}

macro_rules! impl_inc_dec_cp_cmp_trait_i {
    ($($t:ty),*) => {
        $(
            impl IncDecCpCmp<$t,$t> for BlanketIncDecCpCmp {
                fn dec(&self, a: &$t, b:&$t) ->Option<$t> {
                    if *b<=0 { return None}
                    return a.clone().checked_sub(b.clone());
                }

                fn min(&self) ->$t { <$t>::MIN }
                fn max(&self) ->$t { <$t>::MAX }

                fn inc(&self, a: &$t, b: &$t) -> Option<$t> {
                    if *b<=0 { return None}
                    return a.clone().checked_add(b.clone())
                }

                fn cp(&self,v: &$t) ->$t {
                    return v.clone();
                }

                fn lt(&self,a:&$t,b: &$t) ->bool {
                    return a<b;
                }
            }

            impl DefaultValues<$t,$t> for BlanketIncDecCpCmp {
                fn default_step(&self) ->$t { return 1}
                fn default_rebound(&self) ->$t { return 1}
            }
        )*
    };
}

macro_rules! impl_inc_dec_cp_cmp_trait_u {
    ($($t:ty),*) => {
        $(
            impl IncDecCpCmp<$t,$t> for BlanketIncDecCpCmp {
                fn dec(&self, a: &$t, b:&$t) ->Option<$t> {
                    if *b==0 { return None}
                    return a.clone().checked_sub(b.clone());
                }
                fn min(&self) ->$t { <$t>::MIN }
                fn max(&self) ->$t { <$t>::MAX }

                fn inc(&self, a: &$t, b: &$t) -> Option<$t> {
                    if *b==0 { return None}
                    return a.clone().checked_add(b.clone())
                }

                fn cp(&self,v: &$t) ->$t {
                    return v.clone();
                }

                fn lt(&self,a:&$t,b: &$t) ->bool {
                    return a<b;
                }
            }

            impl DefaultValues<$t,$t> for BlanketIncDecCpCmp {
                fn default_step(&self) ->$t { return 1}
                fn default_rebound(&self) ->$t { return 1}
            }
        )*
    };
}

macro_rules! impl_inc_dec_cp_cmp_trait_f {
    ($($t:ty),*) => {
        $(
            impl IncDecCpCmp<$t,$t> for BlanketIncDecCpCmp {
                fn dec(&self, a: &$t, b:&$t) ->Option<$t> {
                    let res=a - b;
                    if res.is_nan() || res >=*a { None } else { Some(res) }
                }

                fn min(&self) ->$t { <$t>::MIN }
                fn max(&self) ->$t { <$t>::MAX }

                fn inc(&self, a: &$t, b: &$t) -> Option<$t> {
                    let res=a + b;
                    if res.is_nan() || res <=*a { None } else { Some(res) }
                }

                fn cp(&self,v: &$t) ->$t {
                    return v.clone();
                }

                fn lt(&self,a:&$t,b: &$t) ->bool {
                    return a<b;
                }
            }

            impl DefaultValues<$t,$t> for BlanketIncDecCpCmp {
                fn default_step(&self) ->$t { return 1.0}
                fn default_rebound(&self) ->$t { return 1.0 }
            }
        )*
    };
}

impl_inc_dec_cp_cmp_trait_u!(u8, u16, u32, u64, u128, usize);
impl_inc_dec_cp_cmp_trait_i!(i8, i16, i32, i64, i128, isize);
impl_inc_dec_cp_cmp_trait_f!(f32, f64);
