use std::cmp::Ordering;
use std::mem;
use std::ops::{Add, Deref};
use std::panic::{UnwindSafe, catch_unwind};
pub enum SpanRelation {
    Before,
    Overlap,
    After,
    None,
}

pub trait CoreValue: Deref + Clone + PartialOrd {}
impl<T: Deref + Clone + PartialOrd<Self>> CoreValue for T {}

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

pub fn first_range_begin_end<T: CoreValue, R>(src: &dyn AsRef<[R]>) -> Option<(T, T)>
where
    R: Deref<Target = dyn RangeSet<T>>,
{
    let mut begin: Option<&T> = None;
    let mut end: Option<&T> = None;

    let list = src.as_ref();
    for span in list {
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

pub fn next_range_begin_end<T: CoreValue, R>(begin: T, src: &dyn AsRef<[R]>) -> Option<(T, T)>
where
    R: Deref<Target = dyn RangeSet<T>> 
{
    let list = src.as_ref();
    let mut target: Option<&T> = None;
    let mut alt: Option<&T> = None;
    for check in list {
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

                for check in list {
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

/*
pub trait Core<V,B>
 where
    B: Deref<Target=dyn RangeSet<V>> + Borrow<dyn RangeSet<V>>,
   {
    fn lt(&self,a: &V,b: &V)->bool;
    fn next_end(&self, current: &V) ->Option<V>;
    fn new_span(&self, a: &V, b: &V) ->B;

    //fn build_core(&self)->Self;

    fn span_contains(&self, check: &dyn RangeSet<V>, value: &V) -> bool {
        !(self.lt(value,check.get_begin()) || self.lt(check.get_end(),value))
    }

    fn span_contains_begin_or_end(&self, outer: &dyn RangeSet<V>, inner: &dyn RangeSet<V>) ->bool {
        self.span_contains(outer, inner.get_begin()) || self.span_contains(outer, inner.get_end())
    }

    fn spans_overlap(&self, a: &dyn RangeSet<V>, b: &dyn RangeSet<V>) ->bool {
        self.span_contains_begin_or_end(a, b) || self.span_contains_begin_or_end(b, a)
    }

    fn span_relation(&self, point: &dyn RangeSet<V>,check: &dyn RangeSet<V> ) ->SpanRelation {
        if self.lt( check.get_end(),point.get_begin()) {
            return SpanRelation::Before;
        } else if self.lt(point.get_end(),check.get_begin()) {
            return SpanRelation::After;
        }

        return SpanRelation::Overlap;

    }

    fn cmp_spans(&self,a:&dyn RangeSet<V>, b: &dyn RangeSet<V>) ->Ordering {

        if self.lt(b.get_begin(),a.get_begin()) {
            return Ordering::Greater;
        } else if self.lt(a.get_begin(),b.get_begin()) {
            return Ordering::Less;

            // anything below this point both begin values are the same
        } else if self.lt(a.get_end(),b.get_end()) {
            return Ordering::Greater;
        } else if self.lt(b.get_end(),a.get_end()) {
            return Ordering::Less;
        }
        // if we get here, begin and end are equal
        return Ordering::Equal;
    }



    fn get_first_span(&self, src: &dyn AsRef<[B]>) -> Option<B> {
        let list=src.as_ref();
        match list.get(0) {
            Some(first)=>{
                let mut begin=first.get_begin();
                let mut end =first.get_end();
                for i in 1..list.len()  {
                    let next=&list[i];
                    let mut cmp=next.get_begin();
                    if self.lt(cmp,begin) {
                        begin=cmp
                    }
                    cmp=next.get_end();
                    if self.lt(cmp,end) {
                        end=cmp
                    }
                }
                return Some(self.new_span(begin,end));
            }
            None=>None
        }
    }

    fn get_next_span(&self,start: &dyn RangeSet<V>, src: &dyn AsRef<[B]>) ->Option<B> {
        let list=src.as_ref();
        if let Some(begin)=self.next_end(start.get_end()) {
        let mut target: Option<&V>=None;
        let mut alt: Option<&V>=None;
        for check in list.into_iter() {
            if self.span_contains(check.borrow(), &begin) {
                let test=check.get_end();
                match target {
                    Some(cmp)=>{
                        if self.lt(test,cmp) {
                            target=Some(test)
                        }
                    }
                    _=>target=Some(test)
                }
            } else {
                let start=check.get_begin();
                if self.lt(&begin,start) {
                    match alt {
                        Some(cmp)=>{
                            if self.lt(start,cmp) {
                                alt=Some(start)
                            }
                        },
                        _=>alt=Some(start)
                    }
                }
            }
        }
        match target {
            Some(end)=>Some(self.new_span(&begin,end)),
            _=>{
                match alt {
                    Some(begin)=>{
                        target=None;

                        for check in list.into_iter() {
                            if self.span_contains(check.borrow(), begin) {
                                let start=check.get_begin();
                                let end=check.get_end();

                                match target {
                                    Some(cmp)=> {
                                        if self.lt(begin,start) && self.lt(start,cmp) {
                                            target=Some(start)
                                        } else if self.lt(end,cmp) {
                                            target=Some(end)
                                        }
                                    }
                                    _=>target=Some(if self.lt(begin,start) { start } else { end})
                                }
                            } else {
                                let start=check.get_begin();
                                if self.lt(begin, start) {
                                    match target {
                                        Some(cmp)=>{
                                            if self.lt(start,cmp) {
                                                target=Some(start)
                                            }
                                        },
                                        _=>target=Some(start)
                                    }
                                }
                            }
                        }
                        match target {
                            Some(end)=>return Some(self.new_span(begin,end)),
                            _=>return None
                        }
                    },
                    _=>return None
                }
            }
        }
        } else {
           return None
        }
    }

}

impl<T> Span<T> {
    pub fn new(begin: T, end: T) ->Self {
        Self { begin: begin, end: end, }
    }
}

pub struct SpanIter<C: Core<V,B>,V,B>
where
    B: Deref<Target=dyn RangeSet<V>> + Borrow<dyn RangeSet<V>>,
    {
    list: Vec<B>,
    core: C,
    next: Option<B>,
}

impl<C: Core<V,B>,V,B> SpanIter<C,V,B>
  where
    B: Deref<Target=dyn RangeSet<V>> + Borrow<dyn RangeSet<V>>,
  {
    pub fn new(core: C, list: Vec<B>) ->Self {

        let next=core.get_first_span(&list);
        return Self {
            list: list,
            core: core,
            next: next,
        }
    }

    pub fn update_cell(&mut self,i: usize, value: B) {
        self.list[i]=value;

    }
}



impl<C: Core<V,B>,V,B>  Iterator for  SpanIter<C,V,B>
where
    B: Deref<Target=dyn RangeSet<V>> + Borrow<dyn RangeSet<V>> ,
 {
    type Item = B;
    fn next(&mut self) -> Option<Self::Item> {
        let mut target:Option<B>=None;
        {
            let next=&self.next;
            match next {
                Some(check)=>target=self.core.get_next_span(check.borrow(), &self.list),
                _=>(),
            }
        }
        match target {
            Some(span)=>mem::replace(&mut self.next, Some(span)),
            _=>None,
         }
    }

}

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn new_span() {
        let span=Span::new(0, 1);
        assert_eq!(span.get_begin(),&0);
        assert_eq!(span.get_end(),&1);
    }

    struct Tools { }

    type S=Box<dyn RangeSet<i32>>;
    impl  Core<i32,S> for Tools {
        fn new_span(&self, begin: &i32,end: &i32) ->S {
            let span: Span<i32>=Span::new(begin+0,end+0);
            return Box::new(span);
        }

        fn lt(&self, a:&i32,b:&i32) ->bool {
            return a<b
        }

        fn next_end(&self, current: &i32) ->Option<i32> {
            return current.checked_add(1);
        }

    }

    fn build_core() ->Tools {
        return Tools {};
    }

    #[test]
    fn basic_core_tests() {
        let core=build_core();

        let span=core.new_span(&1, &2);

        assert_eq!(span.get_begin(),&1);
        assert_eq!(span.get_end(),&2);

        // positive contains value test(s)
        assert!(core.span_contains(span.borrow(), &1));
        assert!(core.span_contains(span.borrow(), &2));

        // negative contains value test(s)
        assert!(!core.span_contains(span.borrow(), &3));
        assert!(!core.span_contains(span.borrow(), &0));


        // positive contains begin or end test(s)
        assert!(core.span_contains_begin_or_end(span.borrow(), span.borrow()));

        // negative contains begin or end test(s)
        let before=core.new_span(&-1, &0);
        let after=core.new_span(&3, &3);

        assert!(!core.span_contains_begin_or_end(span.borrow(), before.borrow()));
        assert!(!core.span_contains_begin_or_end(span.borrow(), after.borrow()));

        let all=core.new_span(&-2, &4);
        assert!(!core.span_contains_begin_or_end(span.borrow(), all.borrow()));


        // positive overlap tests
        assert!(core.spans_overlap(all.borrow(), span.borrow()));
        assert!(core.spans_overlap(span.borrow(), all.borrow()));

        assert!(!core.spans_overlap(before.borrow(), after.borrow()));


    }

    #[test]
    fn relation_tests() {

        let core=build_core();
        let before=core.new_span(&-1, &0);
        let after=core.new_span(&3, &3);
        let all=core.new_span(&-2, &4);
        assert!(matches!(core.span_relation(before.borrow(), after.borrow()),SpanRelation::After));
        assert!(matches!(core.span_relation(after.borrow(), before.borrow()),SpanRelation::Before));
        assert!(matches!(core.span_relation(all.borrow(), before.borrow()),SpanRelation::Overlap));
        assert!(matches!(core.span_relation(all.borrow(), after.borrow()),SpanRelation::Overlap));
    }

    #[test]
    fn first_span_tests() {
        let core=build_core();
        // positive testing
        let mut list=vec![
            core.new_span(&4, &5),
            core.new_span(&0, &3),
            core.new_span(&1, &2),
        ];

        if let Some(span)=core.get_first_span(&list) {
            check_span(span.borrow(), 0, 2);
        }

        // negative testing
        list.clear();
        assert!(matches!(core.get_first_span(&list),None));
    }

    fn check_span(check: &dyn RangeSet<i32>,a: i32,b: i32) {

        print!("    Expected: {}->{}, Got: {}->{}\n",a,b,check.get_begin(),check.get_end());
        assert_eq!(check.get_begin(),&a);
        assert_eq!(check.get_end(),&b);
    }
    #[test]
    fn next_span_tests() {
        let core=build_core();
        // positive testing
        let mut list=vec![
            core.new_span(&4, &5),
            core.new_span(&4, &6),
            core.new_span(&0, &3),
            core.new_span(&1, &2),
            // gap 1
            core.new_span(&8, &11),
            // gap 2
            core.new_span(&13, &22),
            core.new_span(&15, &19),
        ];

        let mut point=core.new_span(&0, &2);

        let mut checkset=vec![
            (3,3),
            (4,5),
            (6,6),
            (8,11),
            (13,15),
            (16,19),
            (20,22),
        ];

        for (a,b) in checkset {
            if let Some(next)=core.get_next_span(point.borrow(), &list) {
                check_span(next.borrow(), a, b);
                point=next;
            }
        }

        assert!(matches!(core.get_next_span(point.borrow(), &list),None));

        // we really only have 1 negative test..  IE empty list!
        list.clear();
        assert!(matches!(core.get_next_span(point.borrow(), &list),None));


        // validate smallest default gap in reversal of
        point=core.new_span(&8, &11);
        list=vec![
            // reversing  the order of the gap for coverage
            core.new_span(&15, &19),
            core.new_span(&13, &22),
            // order should never mater
            core.new_span(&8, &11),
        ];

        checkset=vec![
            (13,15),
            (16,19),
            (20,22),
        ];

        for (a,b) in checkset {
            if let Some(next)=core.get_next_span(point.borrow(), &list) {
                check_span(next.borrow(), a, b);
                point=next;
            }
        }
        assert!(matches!(core.get_next_span(point.borrow(), &list),None));

    }

}
    */
