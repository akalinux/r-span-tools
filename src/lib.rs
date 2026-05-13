use std::cmp::Ordering;
use std::mem;
use std::ops::{Add, Deref};
use std::panic::{UnwindSafe, catch_unwind};

pub enum RangeRelation {
    Before,
    Overlap,
    After,
}

pub trait CoreValue: Clone + PartialOrd {}
impl<T: Clone + PartialOrd<Self>> CoreValue for T {}

pub trait CoreAddValue: CoreValue + Add<Self, Output = Self> + UnwindSafe {}
impl<T: CoreValue + Add<T, Output = T> + UnwindSafe> CoreAddValue for T {}

pub fn safe_add_value<T: CoreAddValue>(a: &T, b: &T) -> Option<T> {
    let x = a.clone();
    let y = b.clone();

    let result = catch_unwind(|| x + y);

    match result {
        Ok(begin) => Some(begin),
        Err(_) => None,
    }
}

pub trait RangeSet<T: CoreValue> {
    fn get_begin(&self) -> &T;
    fn get_end(&self) -> &T;

    fn contains_value(&self, value: &T) -> bool {
        !(value < self.get_begin() || value > self.get_end())
    }

    fn contains(&self, check: &dyn RangeSet<T>) -> bool {
        return self.contains_value(check.get_begin()) || self.contains_value(check.get_end());
    }

    fn overlap(&self, check: &dyn RangeSet<T>) -> bool {
        return self.contains(check)
            || check.contains_value(&self.get_begin())
            || check.contains_value(&self.get_end());
    }

    fn is_empty(&self) -> bool {
        return self.get_begin() > self.get_end();
    }

    /// Provides positional relationship of range to self
    fn range_relation(&self, range: &dyn RangeSet<T>) -> RangeRelation {
        if range.get_end() < self.get_begin() {
            return RangeRelation::Before;
        } else if self.get_end() < range.get_begin() {
            return RangeRelation::After;
        }
        return RangeRelation::Overlap;
    }
}

/// Static sort method for SpanSet<T>.
pub fn partial_cmp<T: CoreValue>(a: &dyn RangeSet<T>, b: &dyn RangeSet<T>) -> Ordering {
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

pub fn first_range_begin_end<T: CoreValue, R>(src: &[R]) -> Option<(T, T)>
where
    R: Deref<Target = dyn RangeSet<T>>,
{
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

pub fn next_range_begin_end<T: CoreValue, R>(begin: T, src: &[R]) -> Option<(T, T)>
where
    R: Deref<Target = dyn RangeSet<T>>,
{
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

pub struct Span<T: CoreValue> {
    begin: T,
    end: T,
}

impl<T: CoreValue> RangeSet<T> for Span<T> {
    fn get_begin(&self) -> &T {
        &self.begin
    }

    fn get_end(&self) -> &T {
        &self.end
    }
}

impl<T: CoreAddValue> Span<T> {
    pub fn new(begin: T, end: T) -> Self {
        return Span { begin, end };
    }
}

pub struct SpanIter<'a, T: CoreAddValue, R>
where
    R: Deref<Target = dyn RangeSet<T>>,
{
    src: &'a mut [R],
    next: Option<Span<T>>,
    step: T,
}

impl<'a, T: CoreAddValue, R> SpanIter<'a, T, R>
where
    R: Deref<Target = dyn RangeSet<T>>,
{
    pub fn new(src: &'a mut [R], step: T) -> Self {
        let mut next: Option<Span<T>> = None;
        if let Some((begin, end)) = first_range_begin_end(src) {
            next = Some(Span { begin, end })
        }
        Self { src, next, step }
    }

    pub fn update_column(&mut self, span: R, idx: usize) {
        if idx> self.src.len() {
            return;
        }
        *&mut self.src[idx]=span;
    }
}

impl<'a, T: CoreAddValue, R> Iterator for SpanIter<'a, T, R>
where
    R: Deref<Target = dyn RangeSet<T>>,
{
    type Item = Span<T>;
    fn next(&mut self) -> Option<Span<T>> {
        let mut next: Option<Span<T>> = None;
        {
            let mut current = None;
            {
                let check = &self.next;
                match check {
                    Some(span) => current = Some(span),
                    _ => (),
                }
            }
            if let Some(span) = current {
                if let Some(start) = safe_add_value(span.get_end(), &self.step) {
                    if let Some((begin, end)) = next_range_begin_end(start, self.src) {
                        next= Some(Span { begin,end })
                    }
                }
            }
        }
        match next {
            Some(span)=> mem::replace(&mut self.next, Some(span)),
            None=>mem::replace(&mut self.next, None)
        }
    }
}

#[cfg(test)]
mod span_tests {
    use crate::safe_add_value;

    #[test]
    fn test_add() {
        // positive test
        let mut nv: Option<u8> = safe_add_value(&1, &2);
        assert!(matches!(nv, Some(3)));

        // negative test
        nv = safe_add_value(&255, &1);
        assert!(matches!(nv, None));
    }
}
