use crate::RangeSet;
use crate::types::RangeAddSubValue;
use crate::utils::{first_range_begin_end, next_range_begin_end};
use std::mem;

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
        if self.src.len() == 0 {
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
            if let Some(begin) = end.clone().checked_inc(self.step.clone()) {
                return mem::replace(&mut self.next, next_range_begin_end(&begin, &self.src));
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
