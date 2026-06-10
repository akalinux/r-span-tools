use common_range_tools::{CpCmp, IncDecCpCmp, Intersector, RiFactory};
#[derive(Clone, Copy, Debug)]
struct Point {
    p: i32,
}

const MIN: Point = Point { p: 0 };
const MAX: Point = Point { p: 8 };
struct CustomIncDecCpCmp {}

impl CpCmp<Point> for CustomIncDecCpCmp {
    // a way to copy or clone the current struct is required!
    fn cp(&self, v: &Point) -> Point {
        return v.clone();
    }
    // All compare operations can be derived from either lt or gt.
    // The only compare method that is required to be implemented
    // for this trait is the lt operator.
    fn lt(&self, a: &Point, b: &Point) -> bool {
        a.p < b.p
    }
    fn min(&self) -> Point {
        return MIN;
    }
    fn max(&self) -> Point {
        return MAX;
    }
    fn min_ref(&self) -> &Point {
        &MIN
    }
    fn max_ref(&self) -> &Point {
        &MAX
    }
}

impl IncDecCpCmp<Point, Point> for CustomIncDecCpCmp {
    fn inc(&self, a: &Point, b: &Point) -> Option<Point> {
        match a.p.checked_add(b.p) {
            Some(x) => Some(Point { p: x }),
            None => None,
        }
    }

    fn dec(&self, a: &Point, b: &Point) -> Option<Point> {
        match a.p.checked_sub(b.p) {
            Some(x) => Some(Point { p: x }),
            None => None,
        }
    }
}

fn main() {
    let t = CustomIncDecCpCmp {};

    let mut isec = Intersector::new(
        Vec::new(),       // Container for our internal ranges
        Point { p: 1 },   // step
        Point { p: 1 },   // Rebound value
        t,                // our compare instance
        RiFactory::new(), // Factory used to construct new ranges
    );

    // Note: an internal RangeInclusive instance is
    // generated for every range that is successfully added.
    // This means it is safe to drop the orginal range values
    // after they are loaded into the Intersector instance.
    isec.add_range(&(..Point { p: 2 }));
    isec.add_range(&(Point { p: 1 }..Point { p: 3 }));
    isec.add_range(&(Point { p: 3 }..=Point { p: 4 }));
    isec.add_range(&(Point { p: 3 }..));
    for r in isec.into_iter() {
        println!("X: {:?}, Y: {:?}", r.start(), r.end());
    }
    // Output will be:
    //  X: Point { p: 0 }, Y: Point { p: 0 }
    //  X: Point { p: 1 }, Y: Point { p: 1 }
    //  X: Point { p: 2 }, Y: Point { p: 2 }
    //  X: Point { p: 3 }, Y: Point { p: 4 }
    //  X: Point { p: 5 }, Y: Point { p: 8 }
}
