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
    } else {
        return None;
    }
}

fn contains<T, V, R: GetBeginEnd<T>, C: IncDecCpCmp<T, V>>(check: &R, value: &T, t: &C) -> bool {
    return t.contains(check.get_begin(), check.get_end(), value);
}

pub(crate) fn next_smallest_range<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    begin: &T,
    end: &T,
    valid: &[&R],
    t: &C,
) -> (T, T) {
    let mut target: Option<&T> = None;

    for r in valid {
        let (start, finish) = r.to_tuple_ref();
        if !t.overlap(begin, end, start, finish) {
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

pub(crate) fn previous_smallest_range<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    begin: &T,
    end: &T,
    valid: &[&R],
    t: &C,
) -> (T, T) {
    let mut target: Option<&T> = None;

    for r in valid {
        let (start, finish) = r.to_tuple_ref();
        if !t.overlap(begin, end, start, finish) {
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
) -> Option<(&'r T, &'r T, Vec<&'r R>)> {
    let mut check: Option<(&T, &T)> = None;

    let mut valid = Vec::new();
    for span in src {
        if t.is_invalid_set(span.get_begin(), span.get_end()) {
            continue;
        }
        valid.push(span);
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
        return Some((begin, end, valid));
    }

    return None;
}
pub fn first_range_begin_end<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    src: &[R],
    t: &C,
) -> Option<(T, T)> {
    let check = min_max(src, t);

    if let Some((begin, end, valid)) = check {
        return Some(next_smallest_range(begin, end, &valid, t));
    }

    return None;
}

pub fn last_range_begin_end<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    src: &[R],
    t: &C,
) -> Option<(T, T)> {
    let check = min_max(src, t);
    if let Some((begin, end, valid)) = check {
        return Some(previous_smallest_range(begin, end, &valid, t));
    }

    return None;
}

pub fn next_range_begin_end<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    begin: &T,
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
        return Some(next_smallest_range(begin, end, &valid, t));
    } else if let Some((begin, end)) = alt {
        return Some(next_smallest_range(begin, end, &valid, t));
    }
    return None;
}

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
        return Some(previous_smallest_range(begin, end, &valid, t));
    } else if let Some((begin, end)) = alt {
        return Some(previous_smallest_range(begin, end, &valid, t));
    }
    return None;
}

mod tests;
