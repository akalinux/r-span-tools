use std::cmp::Ordering;

#[doc(inline)]
use crate::{RangeSet, types::RangeValue};

/// Static sort method.
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
        let mut cmp = span.get_begin();
        if let Some(check) = begin
            && cmp < check
        {
            begin = Some(cmp)
        }
        cmp = span.get_end();
        if let Some(check) = end
            && cmp < check
        {
            end = Some(cmp)
        }
    }

    match begin {
        Some(begin) => match end {
            Some(end) => Some((begin.clone(), end.clone())),
            _ => None,
        },
        _ => None,
    }
}

pub fn next_range_begin_end<T: RangeValue, R: RangeSet<T>>(begin: T, src: &[R]) -> Option<(T, T)> {
    let mut target: Option<&T> = None;
    let mut alt: Option<&T> = None;
    for check in src {
        if check.contains_value(&begin) {
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
            if &begin < start {
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
                    if check.contains_value(begin) {
                        let start = check.get_begin();
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
                        let start = check.get_begin();
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
