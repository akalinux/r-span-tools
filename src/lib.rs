

pub struct Span<T > {
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

impl<T: Copy> Span<T> {
    pub fn new(begin: T, end: T) ->Self {
        Self { begin: begin, end: end, }
    }
}

pub struct Tools<T> {
  lt: fn(Box<T>,Box<T>) ->bool,
  next_el: fn(&T) ->T,
  new_span: fn(begin: &T,end: &T) -> Box<dyn SpanSet<T>>,
}

impl<T> Tools<T> {

    pub fn get_first<S: SpanSet<T>>(&self, list: &Vec<S>) -> Option<Box<dyn SpanSet<T>>> {
        match list.get(0) {
            Some(first)=>{
                let  begin=first.get_begin();
                let  end =first.get_end();
                return Some((self.new_span)(begin,end));
            }
            None=>None
        }
    }

}