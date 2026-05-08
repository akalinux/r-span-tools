
use std::ops::Deref;
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

impl<T> Span<T> {
    pub fn new(begin: T, end: T) ->Self {
        Self { begin: begin, end: end, }
    }
}

pub struct Tools<T,D> 
  {
  lt: fn(&T,&T) ->bool,
  next_el: fn(&T) ->T,
  new_span: fn(begin: &T,end: &T) -> D 
}

impl<T,D> Tools<T,D> 
  where 
    D: Deref<Target=dyn SpanSet<T>>, 
    {
    pub fn new(lt: fn(&T,&T) ->bool,next_el: fn(&T)->T, new_span: fn(&T,&T)->D) ->Self {
        Self {
            lt: lt,
            new_span: new_span,
            next_el: next_el,
        }
    }
    
    pub fn get_first(&self, list: &Vec<D>) -> Option<D> {
        match list.get(0) {
            Some(first)=>{
                let mut  begin=first.get_begin();
                let mut end =first.get_end();
                for i in 1..(list.len() -1) {
                    if let Some(next)=list.get(i) {
                        let mut cmp=next.get_begin();
                        if (self.lt)(cmp,begin) {
                            begin=cmp
                        }
                        cmp=next.get_end();
                        if (self.lt)(cmp,end) {
                            end=cmp
                        }
                    }
                }
                return Some((self.new_span)(begin,end));
            }
            None=>None
        }
    }


    pub fn next_span(&self,start: &D, list: &Vec<D>) ->Option<(D,Vec<usize>)> {
        let begin=(self.next_el)(start.get_end());
        let mut target: Option<&T>=None;
        let mut alt: Option<&T>=None;
        let lt=&self.lt;
        let mut overlaps: Vec<usize>=Vec::new();
        for (i,check ) in list.iter().enumerate() {
            if self.span_contains(&*check, &begin) {
                let test=check.get_end();
                overlaps.push(i);
                match target {
                    Some(cmp)=>{
                        if lt(test,cmp) {
                            target=Some(test)
                        }
                    }
                    _=>target=Some(test)
                }
            } else {
                let end=check.get_end();
                if lt(&begin,end) {
                    match alt {
                        Some(cmp)=>{
                            if lt(end,cmp) {
                                alt=Some(end)
                            }
                        },
                        _=>alt=Some(end)
                    }
                }
            }
        }
        match target {
            Some(end)=>Some(((self.new_span)(&begin,end),overlaps)),
            _=>{
                match alt {
                    Some(begin)=>{
                        target=None;

                        for (i,check ) in list.iter().enumerate() {
                            if self.span_contains(check, begin) {
                                let end=check.get_end();
                                overlaps.push(i);
                                match target {
                                    Some(cmp)=> {
                                        if lt(end,cmp) {
                                            target=Some(end)
                                        }
                                    }
                                    _=>target=Some(end)
                                }
                            }
                        }
                        match target {
                            Some(end)=>Some(((self.new_span)(begin,end),overlaps)),
                            _=>None
                        }
                    },
                    _=>None
                }
            }
        }
    }


    pub fn span_contains(&self, check: &D, value: &T) -> bool {
        let lt=&self.lt;
        !(lt(value,check.get_begin()) || lt(check.get_end(),value))
    }

    pub fn contains_span(&self, a: &D, b: &D) ->bool {
        self.span_contains(a, b.get_begin()) || self.span_contains(a, b.get_end())
    }

    pub fn spans_overlap(&self, a: &D, b: &D) ->bool {
        self.contains_span(a, b) || self.contains_span(b, a)
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

    fn build_tools() ->Tools<i32,Box<dyn SpanSet<i32>>> {
        return Tools::new(
            |a,b| a<b, 
            |c |c+1,
            |a ,b | { 
                let x: Box<dyn SpanSet<i32>>=Box::new(Span::new(a+0,b+0));
                return x;
            }
        )
    }
    #[test]
    fn new_tools_ok() {
        let tools: Tools<i32,Box<dyn SpanSet<i32>>>=build_tools();
        let span=(tools.new_span)(&0,&1);
        assert_eq!(span.get_begin(),&0);
        assert_eq!(span.get_end(),&1);

        assert!(tools.span_contains(&span, &1));
        assert!(tools.span_contains(&span, &0));
        assert!(!tools.span_contains(&span, &2));
        assert!(!tools.span_contains(&span, &-1));
        assert!(tools.spans_overlap(&span, &span))
    }
}