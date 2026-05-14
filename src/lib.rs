//! # R Span Tools
//!
//! `r_span_tools` is a library that, can be used to find all common intersection values for generic typs.
use std::cmp::Ordering;
use std::mem;

pub enum RangeRelation {
    Before,
    Overlap,
    After,
}

/// Trait representing incrementing or decrementing via a checked value.  It is always assumed
/// that, self.checked_inc(rhs) Some(Self) will always return a larger value than either self or rhs.
/// Likewise it is always assumed that self.checked_dec(rhs) Some(Self) will always return a value smaller
/// than self.
///
/// # Examples
///
/// When imported the trait is added to integer and floating point primitives.
/// This example demonstrates the feature behavior using i32.
/// 
/// ```
/// use r_span_tools::SafeIncDec;
///
/// fn main() {
///    // Increment examples
///    assert!( matches!(1.checked_inc(2), Some(3) ));      // Number went up by 2!
///    assert!( matches!(0.checked_inc(0), None ));         // Number did not go up
///    assert!( matches!(0.checked_inc(-2), None ));        // Number did not go up
///    assert!( matches!(i32::MAX.checked_inc(1), None ));  // Catch overflow
///
///    // Decrement examples
///    assert!( matches!(1.checked_dec(2), Some(-1) ));     // Number went down by 2!
///    assert!( matches!(0.checked_dec(0), None ));         // Number did not go down
///    assert!( matches!(0.checked_dec(-2), None ));        // Number did not go down
///    assert!( matches!(i32::MIN.checked_dec(1), None ));  // Catch undeflow
/// }
///
/// ```
///
/// ## Implementation Example
///
/// This example shows how to safely grow or shrik a struct called `MilkSupply`.
///
/// Note: Incrementing by a negative number will result in None and decrementing by a
/// negative number will result in None.
///
/// ```
/// use r_span_tools::SafeIncDec;
///
/// #[derive(Debug, Copy, Clone, PartialEq)]
/// struct MilkSupply { hundreths: i64 }
///
/// impl SafeIncDec for MilkSupply {
///    fn checked_inc(self,rhs: Self) ->Option<Self> {
///      // if we add the number must always go up!
///      if self.hundreths==0 && rhs.hundreths==0 || rhs.hundreths <0 { return None }
///      // check for overflow
///      let next=self.hundreths.checked_add(rhs.hundreths);
///      match next {
///         Some(hundreths)=>Some(MilkSupply { hundreths } ),
///         None=>None,
///      }
///    }
///
///    fn checked_dec(self,rhs: Self) ->Option<Self> {
///      // if we subtract the number must always go down!
///      if self.hundreths==0 && rhs.hundreths==0 ||  rhs.hundreths <0{ return None }
///      let next=self.hundreths.checked_sub(rhs.hundreths);
///      match next {
///         Some(hundreths)=>Some(MilkSupply { hundreths } ),
///         None=>None,
///      }
///    }
/// }
///
/// ```
pub trait SafeIncDec: Sized {
    /// Should capture overflow and the returned Self should be: gt self &&  ltrhs.
    fn checked_inc(self, rhs: Self) -> Option<Self>;
    /// Should capture overflow and the returned Self should be: tt self && lt rhs.
    fn checked_dec(self, rhs: Self) -> Option<Self>;
}

#[macro_export]
macro_rules! impl_checked_inc_sub_u {
    ($($t:ty),*) => {
        $(
            impl SafeIncDec for $t {
                fn checked_dec(self, rhs: Self) ->Option<Self> {
                    if rhs==0  { return None }
                    return self.checked_sub(rhs);
                }
                fn checked_inc(self, rhs: Self) -> Option<Self> {
                    if rhs==0 { return None }
                    return self.checked_add(rhs)
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! impl_checked_inc_sub_i {
    ($($t:ty),*) => {
        $(
            impl SafeIncDec for $t {
                fn checked_dec(self, rhs: Self) ->Option<Self> {
                    if rhs<=0 { return None}
                    return self.checked_sub(rhs);
                }
                fn checked_inc(self, rhs: Self) -> Option<Self> {
                    if rhs<=0 { return None}
                    return self.checked_add(rhs)
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! impl_checked_inc_sub_f {
    ($($t:ty),*) => {
        $(
            impl SafeIncDec for $t {
                fn checked_dec(self, rhs: Self) ->Option<Self> {
                    let res=self - rhs;
                    if res.is_nan() || res.is_infinite() || res >=self || res >=rhs { None } else { Some(res) }
                }
                fn checked_inc(self, rhs: Self) -> Option<Self> {
                    let res=self + rhs;
                    if res.is_nan() || res.is_infinite() || res <=self || res<=rhs { None } else { Some(res) }
                }
            }
        )*
    };
}

impl_checked_inc_sub_u!(u8, u16, u32, u64, u128, usize);
impl_checked_inc_sub_i!(i8, i16, i32, i64, i128, isize);
impl_checked_inc_sub_f!(f32, f64);

pub trait RangeValue: Clone + PartialOrd {}
impl<T: Clone + PartialOrd<Self>> RangeValue for T {}

pub trait RangeAddSubValue: RangeValue + SafeIncDec {}
impl<T: RangeValue + SafeIncDec> RangeAddSubValue for T {}

pub trait RangeSet<T: RangeValue> {
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

pub struct Span<T: RangeValue> {
    begin: T,
    end: T,
}

impl<T: RangeValue> RangeSet<T> for Span<T> {
    fn get_begin(&self) -> &T {
        &self.begin
    }

    fn get_end(&self) -> &T {
        &self.end
    }
}

impl<T: RangeAddSubValue> Span<T> {
    pub fn new(begin: T, end: T) -> Self {
        return Span { begin, end };
    }
}

pub struct SpanIter<'a, T: RangeAddSubValue, R: RangeSet<T>> {
    src: &'a mut [R],
    next: Option<Span<T>>,
    step: T,
}

impl<'a, T: RangeAddSubValue, R: RangeSet<T>> SpanIter<'a, T, R> {
    pub fn new(src: &'a mut [R], step: T) -> Self {
        let mut next: Option<Span<T>> = None;
        if let Some((begin, end)) = first_range_begin_end(src) {
            next = Some(Span { begin, end })
        }
        Self { src, next, step }
    }

    pub fn update_column(&mut self, span: R, idx: usize) {
        if idx > self.src.len() {
            return;
        }
        *&mut self.src[idx] = span;
    }
}

impl<'a, T: RangeAddSubValue, R: RangeSet<T>> Iterator for SpanIter<'a, T, R> {
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
                let check = span.get_end().clone();
                if let Some(start) = check.checked_inc(self.step.clone()) {
                    if let Some((begin, end)) = next_range_begin_end(start.clone(), self.src) {
                        next = Some(Span { begin, end })
                    }
                }
            }
        }
        match next {
            Some(span) => mem::replace(&mut self.next, Some(span)),
            None => mem::replace(&mut self.next, None),
        }
    }
}

#[cfg(test)]
mod span_tests {
    use crate::SafeIncDec;

    #[test]
    fn test_safe_add_sub_doc_example() {
        assert!(matches!(1.checked_inc(2), Some(3))); // Number went up by 2!
        assert!(matches!(0.checked_inc(0), None)); // Number did not go up
        assert!(matches!(0.checked_inc(-2), None)); // Number did not go up
        assert!(matches!(i32::MAX.checked_inc(1), None)); // Catch overflow

        // Decrement examples
        assert!(matches!(1.checked_dec(2), Some(-1))); // Number went down by 2!
        assert!(matches!(0.checked_dec(0), None)); // Number did not go down
        assert!(matches!(0.checked_dec(-2), None)); // Number did not go down
        assert!(matches!(i32::MIN.checked_dec(1), None)); // Catch undeflow
    }
    #[test]
    fn test_add_sub() {
        // int positive test
        let mut i: Option<u8> = 1.checked_inc(2);
        assert!(matches!(i, Some(3)));
        i = 1.checked_dec(1);
        assert!(matches!(i, Some(0)));

        // negative test
        for (a, b) in [(255, 1), (0, 0)] {
            i = a.checked_inc(b);
            assert!(matches!(i, None));
        }
        for (a, b) in [(0, 1), (0, 0)] {
            i = a.checked_dec(b);
            assert!(matches!(i, None));
        }

        // float tests
        let mut f: Option<f32> = 1.0.checked_inc(1.0);
        if let Some(c) = f {
            assert!(c > 1.0)
        } else {
            assert!(false);
        }
        f = 1.0.checked_dec(1.0);
        if let Some(c) = f {
            assert!(c < 1.0)
        } else {
            assert!(false);
        }

        for (a, b) in [(f32::INFINITY, 1.0), (0.0, 0.0)] {
            f = a.checked_inc(b);
            assert!(matches!(f, None));
        }

        let mut u: Option<i8> = 1.checked_inc(2);
        assert!(matches!(u, Some(3)));
        u = 1.checked_dec(1);
        assert!(matches!(u, Some(0)));
        u = (-1).checked_dec(1);
        assert!(matches!(u, Some(-2)));

        // negative test
        for (a, b) in [(127, 1), (0, 0)] {
            u = a.checked_inc(b);
            assert!(matches!(u, None));
        }
        for (a, b) in [(-128, 1), (0, 0), (1, -1)] {
            u = a.checked_dec(b);
            assert!(matches!(u, None));
        }
    }
}
