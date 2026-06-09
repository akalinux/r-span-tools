use crate::{CpCmp, GetBeginEnd, GetBeginEndOption, builder::IncDecCpCmp};
use std::{cmp::Ordering, mem, ops::RangeBounds};

/// This enum is used to represent positional relationships in 3 states
///  - before a range
///  - overlap with a range
///  - after a range
///
/// The additional states, represent the initialization of the set.
///  - empty or no data
///  - last or final
pub enum RangeRelation<T> {
    /// Range a is before range b
    Before(T),

    /// Range a and b overlap
    Overlap(T),

    /// Range a is after range b
    After(T),

    /// Represents final set of data
    Last(T),

    /// Denotes a was not valid
    Invalid(T),
}

impl<T> RangeRelation<T> {
    /// Unwraps the Value.
    pub fn unwrap(self) -> T {
        match self {
            RangeRelation::After(v)
            | RangeRelation::Before(v)
            | RangeRelation::Last(v)
            | RangeRelation::Invalid(v)
            | RangeRelation::Overlap(v) => return v,
        }
    }
    /// Unwraps the state of [RangeRelation::Invalid] or panics!.
    pub fn invalid(self) -> T {
        match self {
            RangeRelation::Invalid(data) => data,
            _ => panic!("Not Last!"),
        }
    }

    /// Unwraps the state of [RangeRelation::Last] or panics!
    pub fn last(self) -> T {
        match self {
            RangeRelation::Last(data) => data,
            _ => panic!("Not Last!"),
        }
    }

    /// Unwraps the state of [RangeRelation::Before] or panics!
    pub fn before(self) -> T {
        match self {
            RangeRelation::Before(data) => data,
            _ => panic!("Not Before!"),
        }
    }

    /// Unwraps the state of [RangeRelation::Overlap] or panics!
    pub fn overlap(self) -> T {
        match self {
            RangeRelation::Overlap(data) => data,
            _ => panic!("Not Overlap!"),
        }
    }

    /// Unwraps the state of [RangeRelation::After] or panics!
    pub fn after(self) -> T {
        match self {
            RangeRelation::After(data) => data,
            _ => panic!("Not After!"),
        }
    }

    /// Returns true if [RangeRelation::Last].
    pub fn is_last(&self) -> bool {
        match self {
            RangeRelation::Last(_) => true,
            _ => false,
        }
    }

    /// Returns true if [RangeRelation::Invalid].
    pub fn is_invalid(&self) -> bool {
        match self {
            RangeRelation::Invalid(_) => true,
            _ => false,
        }
    }

    /// Returns true if [RangeRelation::Overlap].
    pub fn is_overlap(&self) -> bool {
        match self {
            RangeRelation::Overlap(_) => true,
            _ => false,
        }
    }

    /// Returns true if [RangeRelation::Before].
    pub fn is_before(&self) -> bool {
        match self {
            RangeRelation::Before(_) => true,
            _ => false,
        }
    }

    /// Returns true if [RangeRelation::After].
    pub fn is_after(&self) -> bool {
        match self {
            RangeRelation::After(_) => true,
            _ => false,
        }
    }
}
/// Compares the positional relationship between a and b.
///
/// - [RangeRelation::Invalid] a was not a valid range.
/// - [RangeRelation::Before] a is before b.
/// - [RangeRelation::After] a is after b.
/// - [RangeRelation::Overlap] a and b overlap to some degree.
pub fn range_relation<T, R: GetBeginEnd<T>, N: GetBeginEnd<T>, C: CpCmp<T>>(
    a: &R,
    b: &N,
    t: &C,
) -> RangeRelation<()> {
    if t.is_invalid_set(a.get_begin(), a.get_end()) {
        return RangeRelation::Invalid(());
    } else if t.lt(a.get_end(), b.get_begin()) {
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
/// See [IncDecCpCmp] for more details.
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
pub fn sort_forward<T, R: GetBeginEnd<T>, C: CpCmp<T>>(a: &R, b: &R, t: &C) -> Ordering {
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
pub fn sort_reverse<T, R: GetBeginEnd<T>, C: CpCmp<T>>(a: &R, b: &R, t: &C) -> Ordering {
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

fn contains<T, R: GetBeginEnd<T>, C: CpCmp<T>>(check: &R, value: &T, t: &C) -> bool {
    return t.contains(check.get_begin(), check.get_end(), value);
}

pub fn next_smallest_range<T, C: CpCmp<T>, R: GetBeginEnd<T>>(
    begin: &T,
    end: &T,
    src: &[R],
    t: &C,
) -> (T, T) {
    let mut target = end;

    for r in src {
        let (start, finish) = r.to_tuple_ref();
        if t.is_invalid_set(start, finish) || !t.overlap(begin, end, start, finish) {
            continue;
        }
        let mut min = finish;
        if t.lt(begin, start) {
            min = start;
        }
        if t.lt(min, target) {
            target = min
        }
    }

    return (t.cp(begin), t.cp(target));
}

pub fn previous_smallest_range<T, C: CpCmp<T>, R: GetBeginEnd<T>>(
    begin: &T,
    end: &T,
    src: &[R],
    t: &C,
) -> (T, T) {
    let mut target = begin;

    for r in src {
        let (start, finish) = r.to_tuple_ref();
        if t.is_invalid_set(start, finish) || !t.overlap(begin, end, start, finish) {
            continue;
        }
        let mut min = start;
        if t.lt(finish, end) {
            min = finish;
        }
        if t.lt(target, min) {
            target = min;
        }
    }

    return (t.cp(target), t.cp(end));
}

pub(crate) fn min_max<'r, T, R: GetBeginEnd<T>, C: CpCmp<T>>(
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
pub fn first_range_begin_end<T, C: CpCmp<T>, R: GetBeginEnd<T>>(
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
pub fn last_range_begin_end<T, C: CpCmp<T>, R: GetBeginEnd<T>>(src: &[R], t: &C) -> Option<(T, T)> {
    let check = min_max(src, t);
    if let Some((begin, end)) = check {
        return Some(previous_smallest_range(begin, end, src, t));
    }

    return None;
}

/// Searches for the next smallest range valid range of (T,T) overlaps with begin.
/// If no range overlaps with end, it finds the next smallest range after begin.
/// Returns None when no matches were found.
pub fn next_range_begin_end<T, C: CpCmp<T>, R: GetBeginEnd<T>>(
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
pub fn previous_range_begin_end<T, C: CpCmp<T>, R: GetBeginEnd<T>>(
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

pub fn grow<
    T,
    Q: GetBeginEnd<T>,
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
    C: CpCmp<T>,
    F: GetBeginEndOption<T, Q>,
>(
    x: &R,
    y: &S,
    t: &C,
    f: &F,
) -> Q {
    let a;
    if t.lt(x.get_begin(), y.get_begin()) {
        a = t.cp(x.get_begin())
    } else {
        a = t.cp(y.get_begin())
    }
    let z;
    if t.lt(x.get_end(), y.get_end()) {
        z = t.cp(y.get_end());
    } else {
        z = t.cp(x.get_end());
    }
    return f.new_range((a, z));
}

pub fn consolidate<
    T,
    R: GetBeginEnd<T>,
    S: GetBeginEnd<T>,
    C: CpCmp<T>,
    F: GetBeginEndOption<T, R>,
    I: Iterator<Item = S>,
>(
    last: &mut Option<(R, Vec<(usize, S)>)>,
    iter: &mut I,
    t: &C,
    f: &F,
    mut offset: usize,
) -> (usize, Option<RangeRelation<(R, Vec<(usize, S)>)>>) {
    // this value will become our next offset when we are called again!
    for range in iter {
        let mut ar = (
            f.new_range(t.cp_tpl_ref(range.to_tuple_ref())),
            vec![(offset, range)],
        );

        offset += 1;

        let nv = mem::replace(last, None);
        if let Some((src, mut list)) = nv {
            match range_relation(&src, &ar.0, t) {
                RangeRelation::Overlap(_) => {
                    let nr = grow(&src, &ar.0, t, f);
                    list.append(&mut ar.1);
                    if t.is_invalid_set(nr.get_begin(), nr.get_end()) {
                        *last = None;
                        return (offset, Some(RangeRelation::Invalid((nr, list))));
                    } else {
                        *last = Some((nr, list));
                    }
                }
                RangeRelation::Before(_) => {
                    *last = Some(ar);
                    return (offset, Some(RangeRelation::Before((src, list))));
                }
                RangeRelation::After(_) => {
                    *last = Some(ar);
                    return (offset, Some(RangeRelation::After((src, list))));
                }
                RangeRelation::Invalid(_) => {
                    *last = Some(ar);
                    return (offset, Some(RangeRelation::Invalid((src, list))));
                }
                _ => {}
            }
        } else {
            *last = Some(ar);
        }
    }
    if last.is_some() {
        let res = mem::replace(last, None);
        return (offset, Some(RangeRelation::Last(res.unwrap())));
    }
    return (offset, None);
}
