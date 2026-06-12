use common_range_tools::{GetBeginEnd, GetBeginEndOption, NumberIncDecCpCmp, OverlapIter};

struct MyFactory {}

#[derive(Clone, Copy, Debug)]
struct MyRange {
    a: i32,
    b: i32,
}

impl GetBeginEndOption<i32, MyRange> for MyFactory {
    fn factory(&self, opt: Option<(i32, i32)>) -> Option<MyRange> {
        match opt {
            Some((a, b)) => Some(MyRange { a, b }),
            None => None,
        }
    }

    fn new_range(&self, src: (i32, i32)) -> MyRange {
        return MyRange { a: src.0, b: src.1 };
    }
}
impl GetBeginEnd<i32> for MyRange {
    fn get_begin(&self) -> &i32 {
        return &self.a;
    }

    fn get_end(&self) -> &i32 {
        return &self.b;
    }

    fn to_tuple(self) -> (i32, i32) {
        return (self.a, self.b);
    }
}

fn main() {
    for r in OverlapIter::new(
        vec![
            MyRange { a: 1, b: 4 },
            MyRange { a: 3, b: 5 },
            MyRange { a: 4, b: 6 },
        ],
        1,
        NumberIncDecCpCmp::defaults(),
        MyFactory {},
    ) {
        println!("{:?}", r);
    }
}
