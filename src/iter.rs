use crate::types::RangeAddSubValue;
use crate::utils::{first_range_begin_end, next_range_begin_end};
use crate::{
    GetBeginEnd, IncDecCpCmpTrait, Mrs, RangeSet, first_range_begin_end_idcc,
    next_range_begin_end_idcc,
};
use std::cell::RefCell;
use std::mem;
use std::ops::{Bound, RangeBounds};
use std::rc::Rc;
pub struct OverlapIDCC<T, V, C: IncDecCpCmpTrait<T, V>, R: GetBeginEnd<T>> {
    cols: Rc<RefCell<[R]>>,
    cmp: Rc<C>,
    next: Option<(T, T)>,
    step: Rc<V>,
    _marker: std::marker::PhantomData<(T, V)>,
}

impl<T, V, C: IncDecCpCmpTrait<T, V>, R: GetBeginEnd<T>> OverlapIDCC<T, V, C, R> {
    pub fn new(cols: &Rc<RefCell<[R]>>, step: &Rc<V>, cmp: &Rc<C>) -> Self {
        Self {
            cols: Rc::clone(cols),
            cmp: Rc::clone(cmp),
            step: Rc::clone(step),
            next: first_range_begin_end_idcc(&*cols.borrow(), cmp.as_ref()),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T, V, C: IncDecCpCmpTrait<T, V>, R: GetBeginEnd<T>> Iterator for OverlapIDCC<T, V, C, R> {
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        let mut target: Option<(T, T)> = None;
        if let Some((_, finish)) = &self.next {
            if let Some(begin) = self.cmp.inc(finish, self.step.as_ref()) {
                target = next_range_begin_end_idcc(&begin, &*self.cols.borrow(), self.cmp.as_ref())
            }
        }
        return mem::replace(&mut self.next, target);
    }
}
pub struct OverlapIter<'a, T: RangeAddSubValue, R: RangeSet<T>> {
    src: &'a mut [R],
    next: Option<(T, T)>,
    step: T,
}

impl<'a, T: RangeAddSubValue, R: RangeSet<T>> OverlapIter<'a, T, R> {
    pub fn new(src: &'a mut [R], step: T) -> Self {
        let next = first_range_begin_end(src);
        Self { src, next, step }
    }

    pub fn update_column(&mut self, span: R, idx: usize) -> Result<(), &'static str> {
        if self.src.is_empty() {
            return Err(&"Iterator is empty");
        } else if idx > self.src.len() - 1 {
            return Err(&"idx: is out of bounds");
        }
        *&mut self.src[idx] = span;
        return Ok(());
    }
}

impl<'a, T: RangeAddSubValue, R: RangeSet<T>> Iterator for OverlapIter<'a, T, R> {
    type Item = (T, T);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((_, end)) = &self.next {
            if let Some(begin) = end.checked_inc(&self.step) {
                return mem::replace(&mut self.next, next_range_begin_end(&begin, &self.src));
            }
        }
        return None;
    }
}

pub struct RangeOverlapIter<T: RangeAddSubValue> {
    next: Option<(Bound<T>, Bound<T>)>,
    list: Vec<Mrs<T>>,
    step: T,
}

fn flstart<T: RangeAddSubValue>(lstart: &mut Option<T>, start: T, begin: &mut Option<T>) {
    match &begin {
        Some(cmp) => {
            if cmp < &start {
                *begin = Some(cmp.clone());
            }
        }
        _ => *begin = Some(start.clone()),
    }
    lstart.replace(start);
}
fn flfinish<T: RangeAddSubValue>(lfinish: &mut Option<T>, finish: T, end: &mut Option<T>) {
    match &end {
        Some(cmp) => {
            if cmp > &finish {
                *end = Some(cmp.clone());
            }
        }
        _ => *end = Some(finish.clone()),
    }
    lfinish.replace(finish);
}
impl<T: RangeAddSubValue> RangeOverlapIter<T> {
    pub fn new<S: RangeBounds<T>>(src: &[S], step: T) -> Self {
        let mut list: Vec<Mrs<T>> = Vec::new();
        let mut next: Option<(Bound<T>, Bound<T>)> = None;

        let mut begin: Option<T> = None;
        let mut end: Option<T> = None;
        let mut state: u8 = 0;

        for range in src {
            let mut lstart: Option<T> = None;
            let mut lfinish: Option<T> = None;
            if state & 1 != 1 {
                match range.start_bound() {
                    Bound::Included(start) => flstart(&mut lstart, start.clone(), &mut begin),
                    Bound::Excluded(start) => {
                        if let Some(start) = start.checked_dec(&step) {
                            flstart(&mut lstart, start, &mut begin)
                        }
                    }
                    Bound::Unbounded => state |= 1,
                }
            }
            if state & 2 != 2 {
                match range.end_bound() {
                    Bound::Included(finish) => flfinish(&mut lfinish, finish.clone(), &mut end),
                    Bound::Excluded(finish) => {
                        if let Some(finish) = finish.checked_inc(&step) {
                            flfinish(&mut lfinish, finish, &mut end)
                        }
                    }
                    Bound::Unbounded => {
                        state |= 2;
                    }
                }
            }
            if state == 3 {
                break;
            } else if state == 0
                && let Some(start) = lstart
                && let Some(finish) = lfinish
            {
                list.push(Mrs::new(start.clone(), finish.clone()))
            }
        }

        if state == 3 {
            next = Some((Bound::Unbounded, Bound::Unbounded));
        } else if state == 1 {
            match end {
                Some(finish) => next = Some((Bound::Unbounded, Bound::Included(finish))),
                _ => next = None,
            }
        } else if state == 2 {
            match begin {
                Some(start) => next = Some((Bound::Included(start), Bound::Unbounded)),
                _ => next = None,
            }
        } else if let Some((begin, end)) = first_range_begin_end(&list) {
            next = Some((Bound::Included(begin), Bound::Included(end)));
        }

        Self { next, list, step }
    }
}

impl<T: RangeAddSubValue> Iterator for RangeOverlapIter<T> {
    type Item = (Bound<T>, Bound<T>);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((begin, end)) = &self.next {
            if let Bound::Included(_) = begin
                && let Bound::Included(end) = end
            {
                match end.checked_inc(&self.step) {
                    Some(new_begin) => match next_range_begin_end(&new_begin, &self.list) {
                        Some((a, b)) => {
                            return mem::replace(
                                &mut self.next,
                                Some((Bound::Included(a), Bound::Included(b))),
                            );
                        }
                        None => return mem::replace(&mut self.next, None),
                    },
                    None => return mem::replace(&mut self.next, None),
                }
            } else {
                return mem::replace(&mut self.next, None);
            }
        }
        return None;
    }
}

#[cfg(test)]
mod test_overlap_iter {

    use crate::{Mrs, OverlapIter};

    #[test]
    fn test_overlap_iter() {
        let res: Vec<_> = OverlapIter::new(
            &mut [
                Mrs::new(4, 5),
                Mrs::new(4, 6),
                Mrs::new(0, 3),
                Mrs::new(1, 2),
                // gap 1 is 7-7
                Mrs::new(8, 11),
                // gap 2 is 12-12
                Mrs::new(13, 22),
                Mrs::new(15, 19),
            ],
            1,
        )
        .collect();
        assert_eq!(
            res,
            vec![
                (0, 2),
                (3, 3),
                (4, 5),
                (6, 6),
                (8, 11),
                (13, 15),
                (16, 19),
                (20, 22),
            ]
        )
    }

    /*
    use std::ops::RangeInclusive;
    fn poc() {
        let list: Vec<RangeInclusive<i32>> = vec![1..=11];
    }
    */
}
