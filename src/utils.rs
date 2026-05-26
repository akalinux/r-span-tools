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

/*
fn is_begin_gap<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    valid: &[&R],
    end: &T,
    step: &V,
    t: &C,
) -> bool {
    let mut total: usize = 0;
    if let Some(begin) = t.inc(end, step) {
        for r in valid {
            let (a, b) = r.to_tuple_ref();
            if t.contains(a, b, &begin) {
                total += 1;
                if total > 1 {
                    return false;
                }
            }
        }
    }
    return true;
}
*/

fn is_end_gap<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    valid: &[&R],
    begin: &T,
    step: &V,
    t: &C,
) -> bool {
    if let Some(end) = t.dec(begin, step) {
        let mut total: usize = 0;

        for r in valid {
            let (a, b) = r.to_tuple_ref();
            if t.contains(a, b, &end) {
                total += 1;
                if total > 1 {
                    return false;
                }
            }
        }
    }
    return true;
}

fn rebound_next<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    valid: Vec<&R>,
    begin: &T,
    end: &T,
    step: &V,
    t: &C,
    force: bool,
) -> Option<(T, T)> {
    let (start, finish, total, state) = next_smallest_range(begin, end, &valid, t);

    if force {
        if let Some(new_end) = t.dec(&finish, step) {
            if t.is_invalid_set(&start, &new_end) {
                return Some((start, finish));
            }
            return Some((start, new_end));
        }
    }
    if total > 1 {
        if state == 3 {
            let end = t.cp(&start);
            return Some((start, end));
        } else if !(state & 1 == 1) {
            if state == 0
                && let Some(new_end) = t.dec(&finish, step)
                && !t.is_invalid_set(&start, &new_end)
            {
                return Some((start, new_end));
            }
        }
        return Some((start, t.cp(end)));
    } else {
        return Some((start, finish));
    }
}

fn rebound_previous<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    valid: Vec<&R>,
    begin: &T,
    end: &T,
    step: &V,
    t: &C,
    force: bool,
) -> Option<(T, T)> {
    let (start, finish, total, state) = previous_smallest_range(begin, end, &valid, t);

    if force {
        if is_end_gap(&valid, &start, step, t) {
            return Some((start, finish));
        }
        if let Some(new_start) = t.inc(&start, step) {
            if !t.is_invalid_set(&new_start, &finish) {
                return Some((new_start, finish));
            }
        }
    } else if total > 1 && state < 4 {
        if state == 3 {
            let begin = t.cp(&finish);
            return Some((begin, finish));
        } else if state != 1 {
            if state == 2 && is_end_gap(&valid, &finish, step, t) {
                return Some((start, finish));
            }
            if let Some(new_start) = t.inc(&start, step) {
                if !t.is_invalid_set(&new_start, &finish) {
                    return Some((new_start, finish));
                }
            }
        } else if state == 1 {
            return Some((start, finish));
        } else {
            return None;
        }
    }
    return Some((start, finish));
}

pub(crate) fn next_smallest_range<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    begin: &T,
    end: &T,
    valid: &[&R],
    t: &C,
) -> (T, T, usize, u8) {
    let mut target: Option<(&T, bool, bool)> = None;

    let mut total: usize = 0;
    for r in valid {
        let (start, finish) = r.to_tuple_ref();
        if !t.overlap(begin, end, start, finish) {
            continue;
        }
        total += 1;
        let mut min = finish;
        if t.lt(begin, start) {
            min = start;
        }
        match target {
            Some((cmp, _, _)) => {
                if t.lt(min, cmp) {
                    target = Some((min, t.eq(begin, start), t.eq(end, finish)))
                }
            }
            None => target = Some((min, t.eq(begin, start), t.eq(end, finish))),
        }
    }

    match target {
        Some((end, is_a, is_b)) => (
            t.cp(begin),
            t.cp(end),
            total,
            (is_a as u8) | ((is_b as u8) << 1),
        ),
        None => (t.cp(begin), t.cp(end), total, 4),
    }
}

pub(crate) fn previous_smallest_range<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    begin: &T,
    end: &T,
    valid: &[&R],
    t: &C,
) -> (T, T, usize, u8) {
    let mut target: Option<(&T, bool, bool)> = None;

    let mut total: usize = 0;
    for r in valid {
        let (start, finish) = r.to_tuple_ref();
        if !t.overlap(begin, end, start, finish) {
            continue;
        }
        total += 1;
        let mut min = start;
        if t.lt(finish, end) {
            min = finish;
        }
        match target {
            Some((cmp, _, _)) => {
                if t.lt(cmp, min) {
                    target = Some((min, t.eq(begin, start), t.eq(end, finish)))
                }
            }
            None => target = Some((min, t.eq(begin, start), t.eq(end, finish))),
        }
    }

    match target {
        Some((begin, is_a, is_b)) => (
            t.cp(begin),
            t.cp(end),
            total,
            (is_a as u8) | ((is_b as u8) << 1),
        ),
        None => (t.cp(begin), t.cp(end), total, 4),
    }
}

pub fn first_range_begin_end<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    src: &[R],
    step: &V,
    t: &C,
) -> Option<(T, T)> {
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
        return rebound_next(valid, begin, end, step, t, true);
    }

    return None;
}

pub fn last_range_begin_end<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    src: &[R],
    step: &V,
    t: &C,
) -> Option<(T, T)> {
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
        return rebound_previous(valid, begin, end, step, t, true);
    }

    return None;
}

pub fn next_range_begin_end<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    begin: &T,
    src: &[R],
    step: &V,
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
        return rebound_next(valid, begin, end, step, t, false);
    } else if let Some((begin, end)) = alt {
        return rebound_next(valid, begin, end, step, t, false);
    }
    return None;
}

pub fn previous_range_begin_end<T, V, C: IncDecCpCmp<T, V>, R: GetBeginEnd<T>>(
    end: &T,
    src: &[R],
    step: &V,
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
        return rebound_previous(valid, begin, end, step, t, false);
    } else if let Some((begin, end)) = alt {
        return rebound_previous(valid, begin, end, step, t, false);
    }
    return None;
}

mod tests;
