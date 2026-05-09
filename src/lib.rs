
use std::mem;
use std::ops::Deref;
use std::borrow::Borrow;
use std::cmp::Ordering;

pub enum SpanRelation {
    Before,
    Overlap,
    After,
}
pub struct Span<T> {
    begin: T,
    end: T,
}


pub trait SpanSet<T> {
    fn get_begin(&self) -> &T;
    fn get_end(&self) -> &T;
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

    fn span_relation(&self, point: &dyn SpanSet<V>,check: &dyn SpanSet<V> ) ->SpanRelation {
        if self.lt( check.get_end(),point.get_begin()) {
            return SpanRelation::Before;
        } else if self.lt(point.get_end(),check.get_begin()) {
            return SpanRelation::After;
        } 

        return SpanRelation::Overlap;

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

    fn next_span(&self,start: &dyn SpanSet<V>, list: &Vec<B>) ->Option<B> {
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
                let end=check.get_end();
                if self.lt(&begin,end) {
                    match alt {
                        Some(cmp)=>{
                            if self.lt(end,cmp) {
                                alt=Some(end)
                            }
                        },
                        _=>alt=Some(end)
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
                                let end=check.get_end();
                                match target {
                                    Some(cmp)=> {
                                        if self.lt(end,cmp) {
                                            target=Some(end)
                                        }
                                    }
                                    _=>target=Some(end)
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

pub struct SpanItr<C: Core<V,B>,V,B> 
where 
  B: Deref<Target=dyn SpanSet<V>>+ Borrow<dyn SpanSet<V>>
    {
    list: Vec<B>,
    core: C,
    next: Option<B>,
}

impl<C: Core<V,B>,V,B> SpanItr<C,V,B> 
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

impl<C: Core<V,B>,V,B>  Iterator for  SpanItr<C,V,B>
where 
    B: Deref<Target=dyn SpanSet<V>> + Borrow<dyn SpanSet<V>> +SpanSet<V>,
 {
    type Item = B;
    fn next(&mut self) -> Option<Self::Item> {
        let mut target:Option<B>=None;
        {
            let next=&self.next;
            match next {
                Some(check)=>target=self.core.next_span(check, &self.list),
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

        // positive test
        assert!(core.span_contains(span.borrow(), &1));
        assert!(core.span_contains(span.borrow(), &2));

        // negative test
        assert!(!core.span_contains(span.borrow(), &3));
        assert!(!core.span_contains(span.borrow(), &0));

        assert!(core.span_contains_begin_or_end(span.borrow(), span.borrow()));

        let before=core.new_span(&-1, &0);
        let after=core.new_span(&3, &3);

        assert!(!core.span_contains_begin_or_end(span.borrow(), before.borrow()));
        assert!(!core.span_contains_begin_or_end(span.borrow(), after.borrow()));

        let all=core.new_span(&-2, &4);
        assert!(!core.span_contains_begin_or_end(span.borrow(), all.borrow()));

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

}