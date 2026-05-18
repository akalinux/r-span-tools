use std::cmp::Ordering;

use crate::{GetBeginEnd, RangeSet, types::RangeValue};

pub struct BlanketIncDecCpCmp {}

impl BlanketIncDecCpCmp {
    pub fn new() -> Self {
        Self {}
    }
}

/// Acts as the general helper trait for creating and comparing values outside of ranges.
/// Note that incrementing and decrementing are of 2 differnt types, but do not have to be.
///
/// ## Blanket implementation
///
/// ```
/// use common_range_tools::utils::{BlanketIncDecCpCmp, IncDecCpCmpTrait};
///
/// fn main() {
///     
///     let l=BlanketIncDecCpCmp::new();
///     // i32 Increment examples
///     assert_eq!(l.inc(&1, &2), Some(3)); // Number went up by 2!
///     assert_eq!(l.inc(&1, &0), None); // Number did not go up
///     assert_eq!(l.inc(&0, &-2), None); // Number did not go up
///     assert_eq!(l.inc(&i32::MAX, &1), None); // Catch overflow
///
///     // i32 Decrement examples
///     assert_eq!(l.dec(&1, &2), Some(-1)); // Number went down by 2!
///     assert_eq!(l.dec(&0, &0), None); // Number did not go down
///     assert_eq!(l.dec(&0, &-2), None); // Number did not go down
///     assert_eq!(l.dec(&i32::MIN, &1), None); // Catch undeflow
///
///     // u32 Increment examples
///     assert_eq!(l.inc(&1, &2), Some(3)); // Number went up by 2!
///     assert_eq!(l.inc(&0, &0), None); // Number did not go up
///     assert_eq!(l.inc(&u32::MAX, &1), None); // Catch overflow
///
///     // i32 Decrement examples
///     assert_eq!(l.dec(&3_u32, &2), Some(1)); // Number went down by 2!
///     assert_eq!(l.dec(&3_u32, &0), None); // Number did not go down
///     assert_eq!(l.dec(&u32::MIN, &1), None); // Catch undeflow
///
///     // f32 Increment examples
///     assert_eq!(l.inc(&0.2, &0.5), Some(1.5));
///     assert_eq!(l.inc(&1.7, &-0.5), None);
///     assert_eq!(l.inc(&f32::INFINITY, &0.5), None);
///     assert_eq!(l.inc(&f32::INFINITY, &f32::INFINITY), None);
///     assert_eq!(l.inc(&1.0, &f32::INFINITY), Some(f32::INFINITY));
///     assert_eq!(l.inc(&1.0, &f32::NEG_INFINITY), None);
///
///     // f32 Decrement examples
///     assert_eq!(l.dec(&0.5, &0.5), Some(-0.5));
///     assert_eq!(l.dec(&1.7, &-0.5), None);
///     assert_eq!(l.dec(&f32::INFINITY, &0.5), None);
///     assert_eq!(l.dec(&f32::INFINITY, &f32::INFINITY), None);
///     assert_eq!(l.dec(&1.0, &f32::INFINITY), Some(f32::NEG_INFINITY));
///     assert_eq!(l.dec(&1.0, &f32::NEG_INFINITY), None);
///
///     // positive compare examples
///     assert!(l.lt(&1, &2));
///     assert!(l.le(&1, &2));
///     assert!(l.eq(&2, &2));
///     assert!(l.gt(&4, &3));
///     assert!(l.ge(&4, &3));
///     assert!(l.ne(&4, &3));
///
///     // negative compare examples
///     assert!(!l.ne(&4, &4));
///     assert!(!l.eq(&3, &4));
///     assert!(!l.le(&6, &5));
///     assert!(!l.ge(&4, &5));
///     assert!(!l.lt(&4, &3));
///     assert!(!l.gt(&4, &5));
///
///     // Contains Examples
///     assert!(l.contains(&1, &3, &1));
///     assert!(!l.contains(&1, &2, &0));
///     assert!(!l.contains(&0, &0, &1));
///     assert!(!l.contains(&0, &0, &2));
///
///     // Overlap Examples
///     assert!(l.overlap(&1, &2, &0, &1));
///     assert!(!l.overlap(&1, &2, &0, &0));
///
/// }
/// ```
pub trait IncDecCpCmpTrait<T, V> {
    //. Should return a clone or copy of &T.
    fn cp(&self, v: &T) -> T;

    /// Should safely increment a by b.  The value should always go up.. if not then it should return None.
    fn inc(&self, a: &T, b: &V) -> Option<T>;

    /// Should safely decrement a by b.  The value should always go down... if not then it should return None.
    fn dec(&self, a: &T, b: &V) -> Option<T>;

    /// Should return true if a < b.
    fn lt(&self, a: &T, b: &T) -> bool;

    // Should return the minimum value we will accept.
    fn min(&self) -> T;

    // Should return the maximum value we will accept.
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
    fn sortfn<R: GetBeginEnd<T>>(&self, a: &R, b: &R) -> Ordering {
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
}

macro_rules! impl_inc_dec_cp_cmp_trait_i {
    ($($t:ty),*) => {
        $(
            impl IncDecCpCmpTrait<$t,$t> for BlanketIncDecCpCmp {
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
        )*
    };
}

macro_rules! impl_inc_dec_cp_cmp_trait_u {
    ($($t:ty),*) => {
        $(
            impl IncDecCpCmpTrait<$t,$t> for BlanketIncDecCpCmp {
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
        )*
    };
}

macro_rules! impl_inc_dec_cp_cmp_trait_f {
    ($($t:ty),*) => {
        $(
            impl IncDecCpCmpTrait<$t,$t> for BlanketIncDecCpCmp {
                fn dec(&self, a: &$t, b:&$t) ->Option<$t> {
                    let floor=a.floor();
                    let res=floor - *b;
                    if res.is_nan() || res >=floor { None } else { Some(res) }
                }

                fn min(&self) ->$t { <$t>::MIN }
                fn max(&self) ->$t { <$t>::MAX }

                fn inc(&self, a: &$t, b: &$t) -> Option<$t> {
                    let ceil=a.ceil();
                    let res=ceil + *b;
                    if res.is_nan() || res <=ceil { None } else { Some(res) }
                }

                fn cp(&self,v: &$t) ->$t {
                    return v.clone();
                }

                fn lt(&self,a:&$t,b: &$t) ->bool {
                    return a<b;
                }
            }
        )*
    };
}

impl_inc_dec_cp_cmp_trait_u!(u8, u16, u32, u64, u128, usize);
impl_inc_dec_cp_cmp_trait_i!(i8, i16, i32, i64, i128, isize);
impl_inc_dec_cp_cmp_trait_f!(f32, f64);

#[cfg(test)]
mod lattice_tests {

    use crate::utils::{BlanketIncDecCpCmp, IncDecCpCmpTrait};
    #[test]
    fn inc_dec_behavior() {
        let l = BlanketIncDecCpCmp::new();

        // i32 Increment examples
        assert_eq!(l.inc(&1, &2), Some(3)); // Number went up by 2!
        assert_eq!(l.inc(&1, &0), None); // Number did not go up
        assert_eq!(l.inc(&0, &-2), None); // Number did not go up
        assert_eq!(l.inc(&i32::MAX, &1), None); // Catch overflow

        // i32 Decrement examples
        assert_eq!(l.dec(&1, &2), Some(-1)); // Number went down by 2!
        assert_eq!(l.dec(&0, &0), None); // Number did not go down
        assert_eq!(l.dec(&0, &-2), None); // Number did not go down
        assert_eq!(l.dec(&i32::MIN, &1), None); // Catch undeflow

        // u32 Increment examples
        assert_eq!(l.inc(&1, &2), Some(3)); // Number went up by 2!
        assert_eq!(l.inc(&0, &0), None); // Number did not go up
        assert_eq!(l.inc(&u32::MAX, &1), None); // Catch overflow

        // i32 Decrement examples
        assert_eq!(l.dec(&3_u32, &2), Some(1)); // Number went down by 2!
        assert_eq!(l.dec(&3_u32, &0), None); // Number did not go down
        assert_eq!(l.dec(&u32::MIN, &1), None); // Catch undeflow

        // f32 Increment examples
        assert_eq!(l.inc(&0.2, &0.5), Some(1.5));
        assert_eq!(l.inc(&1.7, &-0.5), None);
        assert_eq!(l.inc(&f32::INFINITY, &0.5), None);
        assert_eq!(l.inc(&f32::INFINITY, &f32::INFINITY), None);
        assert_eq!(l.inc(&1.0, &f32::INFINITY), Some(f32::INFINITY));
        assert_eq!(l.inc(&1.0, &f32::NEG_INFINITY), None);

        // f32 Decrement examples
        assert_eq!(l.dec(&0.5, &0.5), Some(-0.5));
        assert_eq!(l.dec(&1.7, &-0.5), None);
        assert_eq!(l.dec(&f32::INFINITY, &0.5), None);
        assert_eq!(l.dec(&f32::INFINITY, &f32::INFINITY), None);
        assert_eq!(l.dec(&1.0, &f32::INFINITY), Some(f32::NEG_INFINITY));
        assert_eq!(l.dec(&1.0, &f32::NEG_INFINITY), None);
    }

    #[test]
    fn inc_dec_compare_all() {
        let l = BlanketIncDecCpCmp::new();

        // positive examples
        assert!(l.lt(&1, &2));
        assert!(l.le(&1, &2));
        assert!(l.eq(&2, &2));
        assert!(l.gt(&4, &3));
        assert!(l.ge(&4, &3));
        assert!(l.ne(&4, &3));

        // negative examples
        assert!(!l.ne(&4, &4));
        assert!(!l.eq(&3, &4));
        assert!(!l.le(&6, &5));
        assert!(!l.ge(&4, &5));
        assert!(!l.lt(&4, &3));
        assert!(!l.gt(&4, &5));

        // Contains Examples
        assert!(l.contains(&1, &3, &1));
        assert!(!l.contains(&1, &2, &0));
        assert!(!l.contains(&0, &0, &1));
        assert!(!l.contains(&0, &0, &2));

        // Overlap Examples
        assert!(l.overlap(&1, &2, &0, &1));
        assert!(!l.overlap(&1, &2, &0, &0));
    }
}

/// Static sort method implementing the optimal order for consolidating overlapping RangeSet instances togeather.
///
/// Order Provided:
/// - self.get_begin() in descending order
/// - self.get_end() in ascending order
///
/// Example
///
/// ```
/// use common_range_tools::utils::partial_cmp;
/// use common_range_tools::{Mrs,RangeSet};
/// fn main() {
///    let mut check = vec![
///      Mrs::new(15, 19),
///      Mrs::new(13, 22),
///      Mrs::new(8, 11),
///      Mrs::new(8, 9),
///    ];
///    check.sort_by(partial_cmp);
///
///    for (i, good) in vec![
///      Mrs::new(8, 11),
///      Mrs::new(8, 9),
///      Mrs::new(13, 22),
///      Mrs::new(15, 19),
///    ].iter().enumerate() {
///      assert_eq!(check[i].get_begin(), good.get_begin());
///      assert_eq!(check[i].get_end(), good.get_end());
///    }
/// }
/// ```
pub fn partial_cmp<T: RangeValue, R: RangeSet<T>>(a: &R, b: &R) -> Ordering {
    if b.get_begin() < a.get_begin() {
        return Ordering::Greater;
    } else if a.get_begin() < b.get_begin() {
        return Ordering::Less;

    // anything below this point both begin values are the same
    } else if a.get_end() < b.get_end() {
        return Ordering::Greater;
    } else if b.get_end() < a.get_end() {
        return Ordering::Less;
    }
    // if we get here, begin and end are equal
    return Ordering::Equal;
}

fn begin_end_invalid<T, V, C: IncDecCpCmpTrait<T, V>, R: GetBeginEnd<T>>(
    span: &R,
    cmp: &C,
) -> bool {
    return cmp.lt(span.get_end(), span.get_begin());
}

pub fn first_range_begin_end_idcc<T, V, C: IncDecCpCmpTrait<T, V>, R: GetBeginEnd<T>>(
    src: &[R],
    t: &C,
) -> Option<(T, T)> {
    let mut begin: Option<&T> = None;
    let mut end: Option<&T> = None;

    for span in src {
        if begin_end_invalid(span, t) {
            continue;
        }
        let mut cmp = span.get_begin();
        match begin {
            Some(check) => {
                if t.lt(cmp, check) {
                    begin = Some(cmp)
                }
            }
            None => begin = Some(cmp),
        }
        cmp = span.get_end();
        match end {
            Some(check) => {
                if t.lt(cmp, check) {
                    end = Some(cmp)
                }
            }
            None => end = Some(cmp),
        }
    }

    match begin {
        Some(begin) => match end {
            Some(end) => {
                return Some((t.cp(begin), t.cp(end)));
            }
            None => return None,
        },
        None => return None,
    }
}

pub fn first_range_begin_end<T: RangeValue, R: RangeSet<T>>(src: &[R]) -> Option<(T, T)> {
    let mut begin: Option<&T> = None;
    let mut end: Option<&T> = None;

    for span in src {
        if span.is_invalid() {
            continue;
        }
        let mut cmp = span.get_begin();
        match begin {
            Some(check) => {
                if cmp < check {
                    begin = Some(cmp)
                }
            }
            None => begin = Some(cmp),
        }
        cmp = span.get_end();
        match end {
            Some(check) => {
                if cmp < check {
                    end = Some(cmp)
                }
            }
            None => end = Some(cmp),
        }
    }

    match begin {
        Some(begin) => match end {
            Some(end) => {
                return Some((begin.clone(), end.clone()));
            }
            None => return None,
        },
        None => return None,
    }
}

pub fn next_range_begin_end<T: RangeValue, R: RangeSet<T>>(begin: &T, src: &[R]) -> Option<(T, T)> {
    let mut target: Option<&T> = None;
    let mut alt: Option<&T> = None;
    for check in src.iter() {
        if check.is_invalid() {
            continue;
        }
        if check.contains_value(begin) {
            let test = check.get_end();
            match target {
                Some(cmp) => {
                    if test < cmp {
                        target = Some(test)
                    }
                }
                _ => target = Some(test),
            }
        } else {
            let start = check.get_begin();
            if begin < start {
                match alt {
                    Some(cmp) => {
                        if start < cmp {
                            alt = Some(start)
                        }
                    }
                    _ => alt = Some(start),
                }
            }
        }
    }
    match target {
        Some(end) => Some((begin.clone(), end.clone())),
        _ => match alt {
            Some(begin) => {
                target = None;

                for check in src {
                    if check.is_invalid() {
                        continue;
                    }
                    let start = check.get_begin();
                    if check.contains_value(begin) {
                        let end = check.get_end();

                        match target {
                            Some(cmp) => {
                                if begin < start && start < cmp {
                                    target = Some(start)
                                } else if end < cmp {
                                    target = Some(end)
                                }
                            }
                            _ => target = Some(if begin < start { start } else { end }),
                        }
                    } else {
                        if begin < start {
                            match target {
                                Some(cmp) => {
                                    if start < cmp {
                                        target = Some(start)
                                    }
                                }
                                _ => target = Some(start),
                            }
                        }
                    }
                }
                match target {
                    Some(end) => return Some((begin.clone(), end.clone())),
                    _ => return None,
                }
            }
            _ => return None,
        },
    }
}

pub fn next_range_begin_end_idcc<T, V, C: IncDecCpCmpTrait<T, V>, R: GetBeginEnd<T>>(
    begin: &T,
    src: &[R],
    t: &C,
) -> Option<(T, T)> {
    let mut target: Option<&T> = None;
    let mut alt: Option<&T> = None;
    for check in src.iter() {
        if begin_end_invalid(check, t) {
            continue;
        }
        if t.contains(check.get_begin(), check.get_end(), begin) {
            let test = check.get_end();
            match target {
                Some(cmp) => {
                    if t.lt(test, cmp) {
                        target = Some(test)
                    }
                }
                _ => target = Some(test),
            }
        } else {
            let start = check.get_begin();
            if t.lt(begin, start) {
                match alt {
                    Some(cmp) => {
                        if t.lt(start, cmp) {
                            alt = Some(start)
                        }
                    }
                    _ => alt = Some(start),
                }
            }
        }
    }
    match target {
        Some(end) => Some((t.cp(begin), t.cp(end))),
        _ => match alt {
            Some(begin) => {
                target = None;

                for check in src {
                    if begin_end_invalid(check, t) {
                        continue;
                    }
                    let start = check.get_begin();
                    if t.contains(check.get_begin(), check.get_end(), begin) {
                        let end = check.get_end();

                        match target {
                            Some(cmp) => {
                                if t.lt(begin, start) && t.lt(start, cmp) {
                                    target = Some(start)
                                } else if t.lt(end, cmp) {
                                    target = Some(end)
                                }
                            }
                            _ => target = Some(if t.lt(begin, start) { start } else { end }),
                        }
                    } else {
                        if t.lt(begin, start) {
                            match target {
                                Some(cmp) => {
                                    if t.lt(start, cmp) {
                                        target = Some(start)
                                    }
                                }
                                _ => target = Some(start),
                            }
                        }
                    }
                }
                match target {
                    Some(end) => return Some((t.cp(begin), t.cp(end))),
                    _ => return None,
                }
            }
            _ => return None,
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{first_range_begin_end, next_range_begin_end};
    use crate::{
        BlanketIncDecCpCmp, GetBeginEnd, IncDecCpCmpTrait, Mrs, RangeSet,
        first_range_begin_end_idcc, next_range_begin_end_idcc, partial_cmp,
    };

    #[test]
    fn test_first_range() {
        let t = BlanketIncDecCpCmp::new();

        // Empty set test
        assert!(matches!(first_range_begin_end::<i32, Mrs<i32>>(&[]), None));
        assert_eq!(
            first_range_begin_end_idcc::<i32, i32, BlanketIncDecCpCmp, Mrs<i32>>(&[], &t),
            None
        );

        assert_eq!(
            first_range_begin_end_idcc::<i32, i32, BlanketIncDecCpCmp, Mrs<i32>>(
                &[Mrs::new(0, -1)],
                &t
            ),
            None
        );

        // Invalid range test
        assert!(matches!(
            first_range_begin_end::<i32, Mrs<i32>>(&[Mrs::new(0, -1)]),
            None
        ));

        assert!(matches!(
            first_range_begin_end(&[
                Mrs::new(5, 7),
                Mrs::new(0, 2),
                Mrs::new(0, 1),
                Mrs::new(0, 0),
                Mrs::new(2, -1), // this should be invalid
            ]),
            Some((0, 0))
        ));

        assert_eq!(
            first_range_begin_end_idcc(
                &[
                    Mrs::new(5, 7),
                    Mrs::new(0, 2),
                    Mrs::new(0, 1),
                    Mrs::new(0, 0),
                    Mrs::new(2, -1), // this should be invalid
                ],
                &t
            ),
            Some((0, 0))
        );
    }

    #[test]
    fn test_next_span() {
        let mut checkset = vec![
            (3, 3),
            (4, 5),
            (6, 6),
            (8, 11),
            (13, 15),
            (16, 19),
            (20, 22),
        ];

        let mut check = vec![
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
        let mut point = 3;
        for (a, b) in checkset {
            assert_eq!(next_range_begin_end(&point, &check), Some((a, b)),);
            point = b + 1;
        }
        assert_eq!(next_range_begin_end(&point, &check), None,);

        let t = BlanketIncDecCpCmp::new();
        assert_eq!(next_range_begin_end_idcc(&point, &check, &t), None,);

        checkset = vec![(8, 11), (13, 15), (16, 19), (20, 22)];

        // validate smallest default gap in reversal of
        point = 7;
        check = vec![
            // reversing  the order of the gap for coverage
            Mrs::new(15, 19),
            Mrs::new(13, 22),
            // order should never mater
            Mrs::new(8, 11),
        ];
        for (a, b) in checkset {
            assert_eq!(
                next_range_begin_end(&point, &check),
                Some((a.clone(), b.clone())),
            );
            assert_eq!(
                next_range_begin_end_idcc(&point, &check, &t),
                Some((a.clone(), b.clone()))
            );
            point = b + 1;
        }
        assert_eq!(next_range_begin_end(&23, &check), None,);
    }

    #[test]
    fn sort_func_tests() {
        let mut check = vec![
            // reversing  the order of the gap for coverage
            Mrs::new(15, 19),
            Mrs::new(13, 22),
            // order should never mater
            Mrs::new(8, 11),
            Mrs::new(8, 9),
        ];
        check.sort_by(partial_cmp);

        let correct = vec![
            Mrs::new(8, 11),
            Mrs::new(8, 9),
            Mrs::new(13, 22),
            Mrs::new(15, 19),
        ];
        for (i, good) in correct.iter().enumerate() {
            assert_eq!(
                <Mrs<i32> as RangeSet<i32>>::get_begin(&check[i]),
                <Mrs<i32> as RangeSet<i32>>::get_begin(good),
            );
            assert_eq!(
                <Mrs<i32> as RangeSet<i32>>::get_end(&check[i]),
                <Mrs<i32> as RangeSet<i32>>::get_end(good),
            );
        }
        let t = BlanketIncDecCpCmp::new();
        check = vec![
            // reversing  the order of the gap for coverage
            Mrs::new(15, 19),
            Mrs::new(13, 22),
            // order should never mater
            Mrs::new(8, 11),
            Mrs::new(8, 9),
        ];
        check.sort_by(|a, b| t.sortfn(a, b));

        for (i, good) in correct.iter().enumerate() {
            assert_eq!(
                <Mrs<i32> as GetBeginEnd<i32>>::get_begin(&check[i]),
                <Mrs<i32> as GetBeginEnd<i32>>::get_begin(good),
            );
            assert_eq!(
                <Mrs<i32> as GetBeginEnd<i32>>::get_end(&check[i]),
                <Mrs<i32> as GetBeginEnd<i32>>::get_end(good),
            );
        }
    }
}
