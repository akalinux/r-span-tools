use crate::{RangeSet};
use crate::utils::{first_range_begin_end,next_range_begin_end};
use crate::types::{RangeAddSubValue};
use std::mem;

pub struct SpanIter<'a, T: RangeAddSubValue, R: RangeSet<T>,F>
where 
F: FnMut(T,T) ->R {
    src: &'a mut [R],
    next: Option<R>,
    step: T,
    new_span: F
}

impl<'a, T: RangeAddSubValue, R: RangeSet<T>,F > SpanIter<'a, T, R,F>
where 
F: FnMut(T,T)->R {
    pub fn new(src: &'a mut [R], step: T, mut new_span: F) -> Self {
        let mut next: Option<R> = None;
        if let Some((begin, end)) = first_range_begin_end(src) {
            next = Some(new_span(begin,end));
        }
        Self { src, next, step, new_span }
    }

    pub fn update_column(&mut self, span: R, idx: usize)  -> Result<(),&'static str>{
        if self.src.len()==0 {
            return Err(&"Iterator is empty")
        } else if idx > self.src.len() -1 {
            return Err(&"idx: is out of bounds");
        }
        *&mut self.src[idx] = span;
        return Ok(());
    }
}

impl<'a, T: RangeAddSubValue, R: RangeSet<T>,F> Iterator for SpanIter<'a, T, R,F> 
    where
    F: FnMut(T,T)->R
{
    type Item = R;
    fn next(&mut self) -> Option<Self::Item> {
        let mut next: Option<R> = None;
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
                    if let Some((begin, end)) = next_range_begin_end(&start.clone(), self.src) {
                        next = Some((self.new_span)(begin,end))
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
