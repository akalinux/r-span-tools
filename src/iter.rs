use crate::{Span,RangeSet};
use crate::utils::{first_range_begin_end,next_range_begin_end};
use crate::types::{RangeAddSubValue};
use std::mem;



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
