use crate::{GetBeginEnd, builder::IncDecCpCmpTrait};
use std::ops::RangeBounds;

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
pub fn range_bounds_to_values<T, V>(
    range: &impl RangeBounds<T>,
    rebound: &V,
    cmp: &impl IncDecCpCmpTrait<T, V>,
) -> Option<(T, T)> {
    if let Some(begin) = cmp.rebound_start(range.start_bound(), rebound)
        && let Some(end) = cmp.rebound_end(range.end_bound(), rebound)
    {
        return Some((begin, end));
    }

    return None;
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

    use crate::{
        Mrs, builder::BlanketIncDecCpCmp, first_range_begin_end, next_range_begin_end,
        range_bounds_to_values,
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
    fn range_conversion() {
        let t = BlanketIncDecCpCmp::new();

        assert_eq!(range_bounds_to_values(&(1..=2), &1, &t), Some((1, 2)));
    }
}
