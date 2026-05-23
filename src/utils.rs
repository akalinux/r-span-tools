use crate::{GetBeginEnd, builder::IncDecCpCmpTrait};
use std::ops::{Bound, RangeBounds};

/// **Range to value Conversion**
///
/// This method takes a [std::ops::RangeBounds] and returns the calculted values for the type.
///
/// For conversion of start values
///   - [std::ops::Bound::Unbounded] becomes $t::MIN
///   - [std::ops::Bound::Included] value is not changed
///   - [std::ops::Bound::Excluded] value is incremented
///
/// For conversion of end values
///   - [std::ops::Bound::Unbounded] becomes $t::MAX
///   - [std::ops::Bound::Included] value is not changed
///   - [std::ops::Bound::Excluded] value is decremented
///
/// See [crate::IncDecCpCmpTrait] for more details.
///
/// Example of range to number conversion.
///
/// ```
/// use common_range_tools::{BlanketIncDecCpCmp,range_bounds_to_values};
/// use std::ops::{Bound, RangeBounds};
///
/// // demo range boundry container.
/// struct Rd<T> {
///     a: Bound<T>,
///     z: Bound<T>,
/// }
///
/// // example impl
/// impl<T> RangeBounds<T> for Rd<T> {
///     fn start_bound(&self) -> Bound<&T> {
///         match &self.a {
///             Bound::Excluded(a) => Bound::Excluded(a),
///             Bound::Included(a) => Bound::Included(a),
///             Bound::Unbounded => Bound::Unbounded,
///         }
///     }
///     fn end_bound(&self) -> Bound<&T> {
///         match &self.z {
///             Bound::Excluded(z) => Bound::Excluded(z),
///             Bound::Included(z) => Bound::Included(z),
///             Bound::Unbounded => Bound::Unbounded,
///         }
///     }
/// }
///
/// fn main() {
///     let t = BlanketIncDecCpCmp::new();
///     assert_eq!(
///         range_bounds_to_values(
///             &Rd {
///                 // will become 1
///                 a: Bound::Excluded(0),
///                 // will become 2
///                 z: Bound::Excluded(3),
///             },
///             &1,
///             &t
///         ),
///         (1, 2)
///     );
///     assert_eq!(
///         range_bounds_to_values(
///             &Rd {
///                 // litteral
///                 a: Bound::Included(0),
///                 // litteral
///                 z: Bound::Included(3),
///             },
///             &1,
///             &t
///         ),
///         (0, 3)
///     );
///     assert_eq!(
///         range_bounds_to_values(
///             &Rd {
///                 // Converts to i32::MIN
///                 a: Bound::Unbounded,
///                 // Converts to i32::MAX
///                 z: Bound::Unbounded,
///             },
///             &1,
///             &t,
///         ),
///         (i32::MIN, i32::MAX)
///     );
///     assert_eq!(
///         range_bounds_to_values(
///             &Rd {
///                 // litteral
///                 a: Bound::Included(0),
///                 // will become 2
///                 z: Bound::Excluded(3),
///             },
///             &1,
///             &t
///         ),
///         (0, 2)
///     );
///     assert_eq!(
///         range_bounds_to_values(
///             &Rd {
///                 // will beocme 1
///                 a: Bound::Excluded(0),
///                 // litteral
///                 z: Bound::Included(3),
///             },
///             &1,
///             &t
///         ),
///         (1, 3)
///     );
///     assert_eq!(
///         range_bounds_to_values(
///             &Rd {
///                 // becomes 1
///                 a: Bound::Excluded(0),
///                 // becomes i32::MAX
///                 z: Bound::Unbounded,
///             },
///             &1,
///             &t
///         ),
///         (1, i32::MAX)
///     );
///     assert_eq!(
///         range_bounds_to_values(
///             &Rd {
///                 // litteral
///                 a: Bound::Included(0),
///                 // becomes i32::MAX
///                 z: Bound::Unbounded,
///             },
///             &1,
///             &t
///         ),
///         (0, i32::MAX)
///     );
///     assert_eq!(
///         range_bounds_to_values(
///             &Rd {
///                 // becomes i32::MIN
///                 a: Bound::Unbounded,
///                 // litteral
///                 z: Bound::Included(0),
///             },
///             &1,
///             &t
///         ),
///         (i32::MIN, 0)
///     );
///     assert_eq!(
///         range_bounds_to_values(
///             &Rd {
///                 // becomes i32::MIN
///                 a: Bound::Unbounded,
///                 // becomes -1
///                 z: Bound::Excluded(0),
///             },
///             &1,
///             &t
///         ),
///         (i32::MIN, -1)
///     );
/// }
/// ```
pub fn range_bounds_to_values<T, V>(
    range: &impl RangeBounds<T>,
    rebound: &V,
    cmp: &impl IncDecCpCmpTrait<T, V>,
) -> (T, T) {
    let a;
    let b;
    match range.start_bound() {
        Bound::Included(begin) => a = Some(cmp.cp(begin)),
        Bound::Excluded(begin) => a = cmp.inc(begin, rebound),
        Bound::Unbounded => a = Some(cmp.min()),
    }
    match range.end_bound() {
        Bound::Included(end) => b = Some(cmp.cp(end)),
        Bound::Excluded(end) => b = cmp.dec(end, rebound),
        Bound::Unbounded => b = Some(cmp.max()),
    }
    if let Some(a) = a
        && let Some(b) = b
    {
        return (a, b);
    }
    // required for type completeness.. but the code never gets here
    return (cmp.min(), cmp.max());
}

/// Computes the first common (begin: T, end: T) values for a list of [`crate::GetBeginEnd``].
///
pub fn first_range_begin_end<T, V, C: IncDecCpCmpTrait<T, V>, R: GetBeginEnd<T>>(
    src: &[R],
    t: &C,
) -> Option<(T, T)> {
    let mut begin: Option<&T> = None;
    let mut end: Option<&T> = None;

    for span in src {
        if t.is_invalid_range(span) {
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

pub fn next_range_begin_end<T, V, C: IncDecCpCmpTrait<T, V>, R: GetBeginEnd<T>>(
    begin: &T,
    src: &[R],
    t: &C,
) -> Option<(T, T)> {
    let mut target: Option<&T> = None;
    let mut alt: Option<&T> = None;
    for check in src.iter() {
        if t.is_invalid_range(check) {
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
                    if t.is_invalid_range(check) {
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
    use std::ops::RangeBounds;

    use crate::{
        GetBeginEnd, Mrs,
        builder::{BlanketIncDecCpCmp, IncDecCpCmpTrait, RangeRelation},
        first_range_begin_end, next_range_begin_end, range_bounds_to_values,
    };

    #[test]
    fn test_first_range() {
        let t = BlanketIncDecCpCmp::new();

        // Empty set test
        assert_eq!(
            first_range_begin_end::<i32, i32, BlanketIncDecCpCmp, Mrs<i32>>(&[], &t),
            None
        );

        assert_eq!(
            first_range_begin_end::<i32, i32, BlanketIncDecCpCmp, Mrs<i32>>(&[Mrs::new(0, -1)], &t),
            None
        );

        assert_eq!(
            first_range_begin_end(
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
        let mut point = 23;

        let t = BlanketIncDecCpCmp::new();
        assert_eq!(next_range_begin_end(&point, &check, &t), None,);

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
                next_range_begin_end(&point, &check, &t),
                Some((a.clone(), b.clone()))
            );
            point = b + 1;
        }
        assert_eq!(next_range_begin_end(&23, &check, &t), None,);
    }

    #[test]
    fn sort_func_tests() {
        let correct = vec![
            Mrs::new(8, 11),
            Mrs::new(8, 9),
            Mrs::new(13, 22),
            Mrs::new(15, 19),
        ];
        let t = BlanketIncDecCpCmp::new();
        let mut check = vec![
            // reversing  the order of the gap for coverage
            Mrs::new(15, 19),
            Mrs::new(13, 22),
            // order should never mater
            Mrs::new(8, 11),
            Mrs::new(8, 9),
        ];
        check.sort_by(|a, b| t.sortfn(a, b));

        for (i, good) in correct.iter().enumerate() {
            assert_eq!(check[i].get_begin(), good.get_begin());
            assert_eq!(check[i].get_end(), good.get_end());
        }
    }
    #[test]
    fn overlap_check() {
        let t = BlanketIncDecCpCmp::new();
        assert!(matches!(
            t.range_relation(&Mrs::new(1, 2), &Mrs::new(1, 2)),
            RangeRelation::Overlap(())
        ));
        assert!(matches!(
            t.range_relation(&Mrs::new(1, 1), &Mrs::new(1, 2)),
            RangeRelation::Overlap(())
        ));
        assert!(matches!(
            t.range_relation(&Mrs::new(2, 2), &Mrs::new(1, 2)),
            RangeRelation::Overlap(())
        ));
        assert!(matches!(
            t.range_relation(&Mrs::new(0, 0), &Mrs::new(1, 2)),
            RangeRelation::Before
        ));
        assert!(matches!(
            t.range_relation(&Mrs::new(3, 4), &Mrs::new(1, 2)),
            RangeRelation::After
        ));
    }

    use std::ops::Bound;
    struct Rd<T> {
        a: Bound<T>,
        z: Bound<T>,
    }

    impl<T> RangeBounds<T> for Rd<T> {
        fn start_bound(&self) -> Bound<&T> {
            match &self.a {
                Bound::Excluded(a) => Bound::Excluded(a),
                Bound::Included(a) => Bound::Included(a),
                Bound::Unbounded => Bound::Unbounded,
            }
        }

        fn end_bound(&self) -> Bound<&T> {
            match &self.z {
                Bound::Excluded(z) => Bound::Excluded(z),
                Bound::Included(z) => Bound::Included(z),
                Bound::Unbounded => Bound::Unbounded,
            }
        }
    }

    #[test]
    fn range_conversion() {
        let t = BlanketIncDecCpCmp::new();

        assert_eq!(
            range_bounds_to_values(
                &Rd {
                    a: Bound::Excluded(0),
                    z: Bound::Excluded(3),
                },
                &1,
                &t
            ),
            (1, 2)
        );
        assert_eq!(
            range_bounds_to_values(
                &Rd {
                    a: Bound::Included(0),
                    z: Bound::Included(3),
                },
                &1,
                &t
            ),
            (0, 3)
        );
        assert_eq!(
            range_bounds_to_values(
                &Rd {
                    a: Bound::Unbounded,
                    z: Bound::Unbounded,
                },
                &1,
                &t,
            ),
            (i32::MIN, i32::MAX)
        );
        assert_eq!(
            range_bounds_to_values(
                &Rd {
                    a: Bound::Included(0),
                    z: Bound::Excluded(3),
                },
                &1,
                &t
            ),
            (0, 2)
        );

        assert_eq!(
            range_bounds_to_values(
                &Rd {
                    a: Bound::Excluded(0),
                    z: Bound::Included(3),
                },
                &1,
                &t
            ),
            (1, 3)
        );
        assert_eq!(
            range_bounds_to_values(
                &Rd {
                    a: Bound::Excluded(0),
                    z: Bound::Unbounded,
                },
                &1,
                &t
            ),
            (1, i32::MAX)
        );

        assert_eq!(
            range_bounds_to_values(
                &Rd {
                    a: Bound::Included(0),
                    z: Bound::Unbounded,
                },
                &1,
                &t
            ),
            (0, i32::MAX)
        );

        assert_eq!(
            range_bounds_to_values(
                &Rd {
                    a: Bound::Unbounded,
                    z: Bound::Included(0),
                },
                &1,
                &t
            ),
            (i32::MIN, 0)
        );

        assert_eq!(
            range_bounds_to_values(
                &Rd {
                    a: Bound::Unbounded,
                    z: Bound::Excluded(0),
                },
                &1,
                &t
            ),
            (i32::MIN, -1)
        );
    }
}
