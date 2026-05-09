

use std::ops::Deref;
use std::borrow::Borrow;
use std::cmp::Ordering;

pub enum SpanPosition {
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

pub trait SpanTool<T,B> 
 where 
    B: Deref<Target=dyn SpanSet<T>> + Borrow<dyn SpanSet<T>>
   {
    fn lt(&self,a: &T,b: &T)->bool;
    fn next_value(&self, current: &T) ->T;
    fn new_span(&self, a: &T, b: &T) ->B;


    fn span_contains(&self, check: &dyn SpanSet<T>, value: &T) -> bool {
        !(self.lt(value,check.get_begin()) || self.lt(check.get_end(),value))
    }

    fn contains_span(&self, a: &dyn SpanSet<T>, b: &dyn SpanSet<T>) ->bool {
        self.span_contains(a, b.get_begin()) || self.span_contains(a, b.get_end())
    }

    fn spans_overlap(&self, a: &dyn SpanSet<T>, b: &dyn SpanSet<T>) ->bool {
        self.contains_span(a, b) || self.contains_span(b, a)
    }

    fn cmp_spans(&self,a:&dyn SpanSet<T>, b: &dyn SpanSet<T>) ->Ordering {

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

    fn cmp_spans_fn(&self) ->impl FnMut(&dyn SpanSet<T>,&dyn SpanSet<T>) ->Ordering {
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

    fn next_span(&self,start: &dyn SpanSet<T>, list: &Vec<B>) ->Option<B> {
        let begin=self.next_value(start.get_end());
        let mut target: Option<&T>=None;
        let mut alt: Option<&T>=None;
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




#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn new_span() {
        let span=Span::new(0, 1);
        assert_eq!(span.get_begin(),&0);
        assert_eq!(span.get_end(),&1);
    }

}