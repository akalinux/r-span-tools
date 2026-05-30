use crate::{GetBeginEnd, builder::IncDecCpCmp};
use std::{cmp::Ordering, ops::RangeBounds};

/// This enum is used to represent positional relationships in 3 states
///  - before a range
///  - overlap with a range
///  - after a range
pub enum RangeRelation<B, O, A> {
    /// Range a is before range b
    Before(B),
    /// Range a and b overlap
    Overlap(O),
    /// Range a is after range b
    After(A),
}

/// Compares the positional relationship between a and b.
///
/// - [`crate::RangeRelation::Before`] a is before b.
/// - [`crate::RangeRelation::After`] a is after b.
/// - [`crate::RangeRelation::Overlap`] a and b overlap to some degree.
pub fn range_relation<T, V, R: GetBeginEnd<T>, C: IncDecCpCmp<T, V>>(
    a: &R,
    b: &R,
    t: &C,
) -> RangeRelation<(), (), ()> {
    if t.lt(a.get_end(), b.get_begin()) {
        return RangeRelation::Before(());
    } else if t.lt(b.get_end(), a.get_begin()) {
        return RangeRelation::After(());
    }

    return RangeRelation::Overlap(());
}

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
        if cmp.is_invalid_set(&begin, &end) {
            return None;
        }
        return Some((begin, end));
    } else {
        return None;
    }
}

/// Compares range a and b and returns the **Forward Consolidation Order** [std::cmp::Ordering] value.
///
/// The sort order is meant to represent **Forward Consolidation Order** not tradtional range sort order.
/// **Forward Consolidation Order** is represented as earliest largest ranges first.
///
/// Put another way:
/// - GetBeginEnd.get_begin() asc
/// - GetBeginEnd.get_end() desc
///
pub fn sort_forward<T, V, R: GetBeginEnd<T>, C: IncDecCpCmp<T, V>>(
    a: &R,
    b: &R,
    t: &C,
) -> Ordering {
    if t.lt(b.get_begin(), a.get_begin()) {
        return Ordering::Greater;
    } else if t.lt(a.get_begin(), b.get_begin()) {
        return Ordering::Less;

    // anything below this point both begin values are the same
    } else if t.lt(a.get_end(), b.get_end()) {
        return Ordering::Greater;
    } else if t.lt(b.get_end(), a.get_end()) {
        return Ordering::Less;
    }
    // if we get here, begin and end are equal
    return Ordering::Equal;
}

/// Compares range a and b and returns the **Reverse Consolidation Order** [std::cmp::Ordering] value.
///
/// The sort order is meant to represent **Reverse Consolidation Order** not tradtional range sort order.
/// **Reverse Consolidation Order** is represented as latest largest ranges first.
///
/// Put another way:
/// - GetBeginEnd.get_end() desc
/// - GetBeginEnd.get_begin() asc
///
pub fn sort_reverse<T, V, R: GetBeginEnd<T>, C: IncDecCpCmp<T, V>>(
    a: &R,
    b: &R,
    t: &C,
) -> Ordering {
    if t.lt(a.get_end(), b.get_end()) {
        return Ordering::Greater;
    } else if t.lt(b.get_end(), a.get_end()) {
        return Ordering::Less;
    } else if t.lt(b.get_begin(), a.get_begin()) {
        return Ordering::Greater;
    } else if t.lt(a.get_begin(), b.get_begin()) {
        return Ordering::Less;
    }

    // anything below this point both begin values are the same

    // if we get here, begin and end are equal
    return Ordering::Equal;
}

fn contains<T, V, R: GetBeginEnd<T>, C: IncDecCpCmp<T, V>>(check: &R, value: &T, t: &C) -> bool {
    return t.contains(check.get_begin(), check.get_end(), value);
}

pub fn next_smallest_range<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    begin: &T,
    end: &T,
    src: &[R],
    t: &C,
) -> (T, T) {
    let mut target: Option<&T> = None;

    for r in src {
        let (start, finish) = r.to_tuple_ref();
        if t.is_invalid_set(start, finish) || !t.overlap(begin, end, start, finish) {
            continue;
        }
        let mut min = finish;
        if t.lt(begin, start) {
            min = start;
        }
        match target {
            Some(cmp) => {
                if t.lt(min, cmp) {
                    target = Some(min)
                }
            }
            None => target = Some(min),
        }
    }

    match target {
        Some(end) => (t.cp(begin), t.cp(end)),
        None => (t.cp(begin), t.cp(end)),
    }
}

pub fn previous_smallest_range<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    begin: &T,
    end: &T,
    src: &[R],
    t: &C,
) -> (T, T) {
    let mut target: Option<&T> = None;

    for r in src {
        let (start, finish) = r.to_tuple_ref();
        if t.is_invalid_set(start, finish) || !t.overlap(begin, end, start, finish) {
            continue;
        }
        let mut min = start;
        if t.lt(finish, end) {
            min = finish;
        }
        match target {
            Some(cmp) => {
                if t.lt(cmp, min) {
                    target = Some(min)
                }
            }
            None => target = Some(min),
        }
    }

    match target {
        Some(begin) => (t.cp(begin), t.cp(end)),
        None => (t.cp(begin), t.cp(end)),
    }
}

pub(crate) fn min_max<'r, T, V, R: GetBeginEnd<T>, C: IncDecCpCmp<T, V>>(
    src: &'r [R],
    t: &C,
) -> Option<(&'r T, &'r T)> {
    let mut check: Option<(&T, &T)> = None;

    for span in src {
        if t.is_invalid_set(span.get_begin(), span.get_end()) {
            continue;
        }
        let (start, finish) = span.to_tuple_ref();
        match check {
            Some((begin, end)) => {
                let mut a = begin;
                let mut z = end;
                if t.lt(end, finish) {
                    z = finish;
                }
                if t.lt(start, begin) {
                    a = start;
                }
                check = Some((a, z))
            }
            _ => check = Some((start, finish)),
        }
    }
    if let Some((begin, end)) = check {
        return Some((begin, end));
    }

    return None;
}

/// Looks for the first most range, if found returns an Option<(T,T)>.
pub fn first_range_begin_end<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    src: &[R],
    t: &C,
) -> Option<(T, T)> {
    let check = min_max(src, t);

    if let Some((begin, end)) = check {
        return Some(next_smallest_range(begin, end, src, t));
    }

    return None;
}

/// Looks for the last most range, if found returns an Option<(T,T)>.
pub fn last_range_begin_end<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    src: &[R],
    t: &C,
) -> Option<(T, T)> {
    let check = min_max(src, t);
    if let Some((begin, end)) = check {
        return Some(previous_smallest_range(begin, end, src, t));
    }

    return None;
}

/// Searches for the next smallest range valid range of (T,T) overlaps with begin.
/// If no range overlaps with end, it finds the next smallest range after begin.
/// Returns None when no matches were found.
pub fn next_range_begin_end<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    begin: &T,
    src: &[R],
    t: &C,
) -> Option<(T, T)> {
    let mut target: Option<&T> = None;
    let mut alt: Option<(&T, &T)> = None;
    for check in src {
        if t.is_invalid_set(check.get_begin(), check.get_end()) {
            continue;
        }
        let (start, finish) = check.to_tuple_ref();
        if contains(check, begin, t) {
            match target {
                Some(cmp) => {
                    if t.lt(finish, cmp) {
                        target = Some(finish)
                    }
                }
                _ => target = Some(finish),
            }
        } else {
            if t.lt(begin, start) {
                match alt {
                    Some((cmp, _)) => {
                        if t.lt(start, cmp) {
                            alt = Some((start, finish))
                        }
                    }
                    _ => alt = Some((start, finish)),
                }
            }
        }
    }
    if let Some(end) = target {
        return Some(next_smallest_range(begin, end, src, t));
    } else if let Some((begin, end)) = alt {
        return Some(next_smallest_range(begin, end, src, t));
    }
    return None;
}

/// Searches for the previous smallest range valid range of (T,T) overlaps with end.
/// If no range overlaps with end, it finds the previous smallest range before begin.
/// Returns None when no matches were found.
pub fn previous_range_begin_end<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    end: &T,
    src: &[R],
    t: &C,
) -> Option<(T, T)> {
    let mut target: Option<&T> = None;
    let mut alt: Option<(&T, &T)> = None;
    let mut valid = Vec::new();
    for check in src {
        if t.is_invalid_set(check.get_begin(), check.get_end()) {
            continue;
        }
        valid.push(check);
        let (start, finish) = check.to_tuple_ref();
        if contains(check, end, t) {
            match target {
                Some(cmp) => {
                    if t.lt(start, cmp) {
                        target = Some(start)
                    }
                }
                _ => target = Some(start),
            }
        } else {
            if t.lt(finish, end) {
                match alt {
                    Some((x, y)) => {
                        let mut a = x;
                        let mut b = y;
                        if t.lt(y, finish) {
                            b = finish
                        }
                        if t.lt(start, a) {
                            a = start
                        }
                        alt = Some((a, b));
                    }
                    _ => alt = Some((start, finish)),
                }
            }
        }
    }
    if let Some(begin) = target {
        return Some(previous_smallest_range(begin, end, src, t));
    } else if let Some((begin, end)) = alt {
        return Some(previous_smallest_range(begin, end, src, t));
    }
    return None;
}
