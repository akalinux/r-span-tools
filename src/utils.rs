use std::cmp::Ordering;

use crate::{RangeSet, types::RangeValue};

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

#[cfg(test)]
mod tests {
    use crate::utils::{first_range_begin_end, next_range_begin_end};
    use crate::{Mrs, RangeSet, partial_cmp};

    #[test]
    fn test_first_range() {
        // Empty set test
        assert!(matches!(first_range_begin_end::<i32, Mrs<i32>>(&[]), None));

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
            assert_eq!(next_range_begin_end(&point, &check), Some((a, b)),);
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

        for (i, good) in vec![
            Mrs::new(8, 11),
            Mrs::new(8, 9),
            Mrs::new(13, 22),
            Mrs::new(15, 19),
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(check[i].get_begin(), good.get_begin());
            assert_eq!(check[i].get_end(), good.get_end());
        }
    }
}
