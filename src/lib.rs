

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
    D: Deref<Target=dyn SpanSet<T>> + Borrow<dyn SpanSet<T>>, 
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

    pub fn span_cmp_fn(&self) ->impl FnMut(&dyn SpanSet<T>,&dyn SpanSet<T>) ->Ordering {
        let lt: &fn(&T, &T) -> bool=&self.lt;
        return |a: &dyn SpanSet<T>, b:&dyn SpanSet<T>|{
            if lt(b.get_begin(),a.get_begin()) {
                return Ordering::Greater;
            } else if lt(a.get_begin(),b.get_begin()) {
                return Ordering::Less;

                // anything below this point both begin values are the same
            } else if lt(a.get_end(),b.get_end()) {
                return Ordering::Greater;
            } else if lt(b.get_end(),a.get_end()) {
                return Ordering::Less;
            }
            // if we get here, begin and end are equal
            return Ordering::Equal;
        }
    }

    
    pub fn span_cmp_vec_fn(&self) ->impl FnMut(&D,&D) ->Ordering {
        let mut cmp=self.span_cmp_fn();
        return move |a,b| cmp(a.borrow(),b.borrow())
    }

    pub fn next_span(&self,start: &dyn SpanSet<T>, list: &Vec<D>) ->Option<D> {
        let begin=(self.next_el)(start.get_end());
        let mut target: Option<&T>=None;
        let mut alt: Option<&T>=None;
        let lt=&self.lt;
        for check in list {
            if self.span_contains(check.borrow(), &begin) {
                let test=check.get_end();
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
            Some(end)=>Some((self.new_span)(&begin,end)),
            _=>{
                match alt {
                    Some(begin)=>{
                        target=None;

                        for check in list {
                            if self.span_contains(check.borrow(), begin) {
                                let end=check.get_end();
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
                            Some(end)=>{
                                Some((self.new_span)(begin,end))},
                            _=>None
                        }
                    },
                    _=>None
                }
            }
        }
    }


    pub fn span_contains(&self, check: &dyn SpanSet<T>, value: &T) -> bool {
        let lt=&self.lt;
        !(lt(value,check.get_begin()) || lt(check.get_end(),value))
    }

    pub fn contains_span(&self, a: &dyn SpanSet<T>, b: &dyn SpanSet<T>) ->bool {
        self.span_contains(a, b.get_begin()) || self.span_contains(a, b.get_end())
    }

    pub fn spans_overlap(&self, a: &dyn SpanSet<T>, b: &dyn SpanSet<T>) ->bool {
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

        assert!(tools.span_contains(&*span, &1));
        assert!(tools.span_contains(&*span, &0));
        assert!(!tools.span_contains(&*span, &2));
        assert!(!tools.span_contains(&*span, &-1));
        assert!(tools.spans_overlap(&*span, &*span))
    }

    #[test]
    fn span_cmp_ok() {
        let tools: Tools<i32,Box<dyn SpanSet<i32>>>=build_tools();
        let ns=&tools.new_span;
        let a=ns(&1,&2);
        let mut b=ns(&3,&3);
        let mut cmp=tools.span_cmp_fn();

        assert!(matches!(cmp(a.borrow(),b.borrow()),Ordering::Less));
        assert!(matches!(cmp(b.borrow(),a.borrow()),Ordering::Greater));
        assert!(matches!(cmp(a.borrow(),a.borrow()),Ordering::Equal));
        assert!(matches!(cmp(b.borrow(),b.borrow()),Ordering::Equal));

        b=ns(&1,&3);
        assert!(matches!(cmp(a.borrow(),b.borrow()),Ordering::Greater));
    }

    #[test]
    fn sort_vec_of_spans() {
        let tools: Tools<i32,Box<dyn SpanSet<i32>>>=build_tools();
        let new_span=&tools.new_span;
        let mut list=vec![
            new_span(&3,&4),
            new_span(&1,&2),
            new_span(&3,&3),
        ];
        list.sort_by(tools.span_cmp_vec_fn());

        let sane =vec![
            new_span(&1,&2),
            new_span(&3,&4),
            new_span(&3,&3),
        ];
        let mut cmp=tools.span_cmp_vec_fn();

        for (i,check) in list.iter().enumerate() {
            let expected=sane.get(i).unwrap();
            print!("  Got: {}->{}, expectred: {}->{}\n",
              check.get_begin(),
              check.get_end(),
              expected.get_begin(),
              expected.get_end(),
            );
            assert!(matches!(cmp(check,sane.get(i).unwrap()),Ordering::Equal));
        }

    }
}