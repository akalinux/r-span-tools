
use std::mem;
use std::ops::{Deref, RangeInclusive};
use std::borrow::Borrow;
use std::cmp::Ordering;
pub enum RangeRelation {
    Before,
    Overlap,
    After,
    Empty,
}
pub struct Span<T> {
    begin: T,
    end: T,
}

pub trait SpanSet<T> {
    fn get_begin(&self) -> &T;
    fn get_end(&self) -> &T;
    fn try_rangeinclusive(&self)->Option<RangeInclusive<T>> {
        return None
    }

}
pub trait CmpClone: PartialOrd +Clone  {}
impl<C: PartialOrd + Clone> CmpClone for C {}

impl <T: CmpClone> SpanSet<T> for RangeInclusive<T> {
    fn get_begin(&self) -> &T {
        self.start()
    }

    fn get_end(&self) -> &T {
        self.end()
    }

    fn try_rangeinclusive(&self)->Option<RangeInclusive<T>> {
        return Some(self.clone())
    }
}

fn new_range<T: CmpClone>(start: &T, end: &T) -> RangeInclusive<T> {
    start.clone()..=end.clone()
}

fn lt<T: CmpClone>(a: &T, b: &T) ->bool {
    return a<b;
}

struct RangeVecIterBuilder<'a,T: CmpClone> {
    list: &'a Vec<RangeInclusive<T>>,
}

pub trait SpanSetIter<T> {
    fn iter(&mut self) -> impl Iterator<Item = Box<dyn SpanSet<T>>>;
}

impl<'a,T:CmpClone> RangeVecIterBuilder<'a,T> {
    fn new(list: &'a Vec<RangeInclusive<T>>) ->Self {
        Self { list }
    }
}

impl<'a,T:CmpClone +'static > SpanSetIter<T> for RangeVecIterBuilder<'a,T> {
    fn iter(&mut self) ->impl Iterator<Item=Box<dyn SpanSet<T>>>{
      
        return RangeIter::new(&self.list);
        
    }
}

struct RangeIter<'a,T: CmpClone> {
    list: &'a Vec<RangeInclusive<T>>,
    pos: usize,
}

impl <'a,T: CmpClone> RangeIter<'a,T> {
    fn new(list: &'a Vec<RangeInclusive<T>>) ->Self {
        return Self {
            list: list,
            pos:0,
        }
    }
}

impl<'a,T> Iterator for RangeIter<'a,T> 
  where T: CmpClone +'static
  {
    type Item=Box<dyn SpanSet<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.list.len()!=0 && self.list.len() > self.pos {
            let next=&self.list[self.pos];
            let res=Box::new(next.clone());
            self.pos+=1;
            return Some(res);
        }
        return None;
    }

}


fn get_first_range<T: CmpClone>(list: &Vec<RangeInclusive<T>>) -> Option<impl SpanSet<T>> {
    let mut start: Option<(usize,&RangeInclusive<T>)>=None;
    for (idx,span) in list.iter().enumerate() {
        if !span.is_empty() {
            start=Some((idx,span));
            break;
        }
    }
    match start {
        Some((idx,first))=>{
            let mut begin=first.start();
            let mut end =first.end();
            for i in idx..list.len()  {
                let next=&list[i];
                let mut cmp=next.start();
                if cmp < begin {
                    begin=cmp
                }
                cmp=next.end();
                if cmp < end {
                    end=cmp
                }
            }
            return Some(new_range(begin,end));
        }
        None=>None
    }
}


pub trait NextRangeBegin<C> {
    fn begin(&mut self, end:&C) ->C;
}

#[cfg(test)]
mod range_tests {
    use crate::{NextRangeBegin, new_range};

    struct NextBegin {}

    impl NextRangeBegin<i32> for NextBegin {
        fn begin(&mut self, end: &i32) ->i32 {
            return end+1;
        }
    }

    #[test]
    fn test_next_begin() {
        let mut n=NextBegin{};
        assert_eq!(n.begin(&1),2);
    }

    #[test]
    fn test_new_range() {
        let r=new_range(&1,&2);
        assert_eq!(r,1..=2);
    }
}

impl<T> SpanSet<T> for Span<T> {
    fn get_begin(&self) -> &T {
        &self.begin
    }
    fn get_end(&self) -> &T {
        &self.end
    }
}

pub trait Core<V,B> 
 where 
    B: Deref<Target=dyn SpanSet<V>> + Borrow<dyn SpanSet<V>>,
   {
    fn lt(&self,a: &V,b: &V)->bool;
    fn next_value(&self, current: &V) ->V;
    fn new_span(&self, a: &V, b: &V) ->B;
    
    fn build_core(&self)->Self;


    fn span_contains(&self, check: &dyn SpanSet<V>, value: &V) -> bool {
        !(self.lt(value,check.get_begin()) || self.lt(check.get_end(),value))
    }

    fn span_contains_begin_or_end(&self, outer: &dyn SpanSet<V>, inner: &dyn SpanSet<V>) ->bool {
        self.span_contains(outer, inner.get_begin()) || self.span_contains(outer, inner.get_end())
    }

    fn spans_overlap(&self, a: &dyn SpanSet<V>, b: &dyn SpanSet<V>) ->bool {
        self.span_contains_begin_or_end(a, b) || self.span_contains_begin_or_end(b, a)
    }

    fn span_relation(&self, point: &dyn SpanSet<V>,check: &dyn SpanSet<V> ) ->RangeRelation {
        if self.lt( check.get_end(),point.get_begin()) {
            return RangeRelation::Before;
        } else if self.lt(point.get_end(),check.get_begin()) {
            return RangeRelation::After;
        } 

        return RangeRelation::Overlap;

    }

    fn cmp_spans(&self,a:&dyn SpanSet<V>, b: &dyn SpanSet<V>) ->Ordering {

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

    fn cmp_spans_fn(&self) ->impl FnMut(&dyn SpanSet<V>,&dyn SpanSet<V>) ->Ordering {
        return |a,b|self.cmp_spans(a, b)
    }

    fn cmp_spans_vec_fn(&self) ->impl FnMut(&B,&B) ->Ordering {
        let mut cmp=self.cmp_spans_fn();
        return move |a,b| cmp(a.borrow(),b.borrow())
    }

        
    fn get_first_span(&self, list: &Vec<B>) -> Option<B> {
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

    fn get_next_span(&self,start: &dyn SpanSet<V>, list: &Vec<B>) ->Option<B> {
        let begin=self.next_value(start.get_end());
        let mut target: Option<&V>=None;
        let mut alt: Option<&V>=None;
        for check in list {
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

                        for check in list {
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
                            Some(end)=>{
                                Some(self.new_span(begin,end))},
                            _=>None
                        }
                    },
                    _=>None
                }
            }
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
    B: Deref<Target=dyn SpanSet<V>> + Borrow<dyn SpanSet<V>>,
    {
    list: Vec<B>,
    core: C,
    next: Option<B>,
}

impl<C: Core<V,B>,V,B> SpanIter<C,V,B> 
  where 
    B: Deref<Target=dyn SpanSet<V>> + Borrow<dyn SpanSet<V>>,
  {
    pub fn new(core: C, list:Vec<B>) ->Self {
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
    B: Deref<Target=dyn SpanSet<V>> + Borrow<dyn SpanSet<V>> ,
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
mod span_tests {
    use super::*;

    #[test]
    fn new_span() {
        let span=Span::new(0, 1);
        assert_eq!(span.get_begin(),&0);
        assert_eq!(span.get_end(),&1);
    }

    struct Tools { }

    type S=Box<dyn SpanSet<i32>>;
    impl  Core<i32,S> for Tools {
        fn new_span(&self, begin: &i32,end: &i32) ->S {
            let span: Span<i32>=Span::new(begin+0,end+0);
            return Box::new(span);
        }

        fn lt(&self, a:&i32,b:&i32) ->bool {
            return a<b
        }

        fn next_value(&self, current: &i32) ->i32 {
            return current +1;
        }
        fn build_core(&self) ->Self {
            return Self{}
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
        assert!(matches!(core.span_relation(before.borrow(), after.borrow()),RangeRelation::After));
        assert!(matches!(core.span_relation(after.borrow(), before.borrow()),RangeRelation::Before));
        assert!(matches!(core.span_relation(all.borrow(), before.borrow()),RangeRelation::Overlap));
        assert!(matches!(core.span_relation(all.borrow(), after.borrow()),RangeRelation::Overlap));
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

    fn check_span(check: &dyn SpanSet<i32>,a: i32,b: i32) {

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