use crate::{GetBeginEnd, builder::IncDecCpCmp};
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
/// See [crate::IncDecCpCmp] for more details.
///
/// Example of range to number conversion.
///
pub fn range_bounds_to_values<T, V>(
    range: &impl RangeBounds<T>,
    rebound: &V,
    cmp: &impl IncDecCpCmp<T, V>,
) -> Option<(T, T)> {
    if let Some(begin) = cmp.rebound_start(range.start_bound(), rebound)
        && let Some(end) = cmp.rebound_end(range.end_bound(), rebound)
    {
        return Some((begin, end));
    }

    return None;
}

fn range_init<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    src: &[R],
    lt: &impl Fn(&T, &T) -> bool,
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
                if lt(cmp, check) {
                    begin = Some(cmp)
                }
            }
            None => begin = Some(cmp),
        }
        cmp = span.get_end();
        match end {
            Some(check) => {
                if lt(cmp, check) {
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

/// Computes the first common (begin: T, end: T) values for a list of [crate::GetBeginEnd].
pub fn first_range_begin_end<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    src: &[R],
    t: &C,
) -> Option<(T, T)> {
    return range_init(src, &|a: &T, b: &T| t.lt(a, b), t);
}

/// Computes the last common (begin: T, end: T) values for a list of [crate::GetBeginEnd].
pub fn last_range_begin_end<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    src: &[R],
    t: &C,
) -> Option<(T, T)> {
    return range_init(src, &|a: &T, b: &T| t.gt(a, b), t);
}

pub fn next_range_begin_end<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
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

mod tests;
