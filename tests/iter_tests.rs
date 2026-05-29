#![cfg(test)]

mod iter_tests {

    use std::{cell::RefCell, rc::Rc};

    use common_range_tools::{
        Accumulate, AccumulateDefaults, Accumulator, BlanketIncDecCpCmp, GetBeginEnd, IncDecCpCmp,
        Mrs, OverlapIter,
    };

    fn checkset() -> [(i32, i32); 9] {
        return [
            (0, 1),   // 0
            (2, 2),   // 1
            (3, 3),   // 2
            (4, 5),   // 3
            (6, 6),   // 4
            (8, 11),  // 5
            (13, 15), // 6
            (16, 19), // 7
            (20, 22), // 8
        ];
    }

    fn checkset_rev() -> [(i32, i32); 9] {
        return [
            (19, 22),
            (15, 18),
            (13, 14),
            (8, 11),
            (5, 6),
            (4, 4),
            (2, 3),
            (1, 1),
            (0, 0),
        ];
    }

    const MIN: Point = Point { x: i32::MIN };
    const MAX: Point = Point { x: i32::MAX };
    struct TestCmp {}

    impl IncDecCpCmp<Point, Point> for TestCmp {
        fn cp(&self, v: &Point) -> Point {
            return v.clone();
        }

        fn inc(&self, a: &Point, b: &Point) -> Option<Point> {
            match a.x.checked_add(b.x) {
                Some(x) => Some(Point { x }),
                None => None,
            }
        }

        fn dec(&self, a: &Point, b: &Point) -> Option<Point> {
            match a.x.checked_sub(b.x) {
                Some(x) => Some(Point { x }),
                None => None,
            }
        }

        fn lt(&self, a: &Point, b: &Point) -> bool {
            a.x < b.x
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

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Point {
        x: i32,
    }

    pub(crate) fn mrs_set() -> Vec<Mrs<i32>> {
        return vec![
            Mrs::new(0, 3),
            Mrs::new(1, 2),
            Mrs::new(4, 5),
            Mrs::new(4, 6),
            // gap 1 is 7-7
            Mrs::new(8, 11),
            // gap 2 is 12-12
            Mrs::new(13, 22),
            Mrs::new(15, 19),
        ];
    }

    #[test]
    fn iter_test() {
        let checkset = checkset();

        let src = mrs_set();
        let t = BlanketIncDecCpCmp::new();

        let iter: OverlapIter<
            i32,
            i32,
            BlanketIncDecCpCmp<i32>,
            Mrs<i32>,
            Rc<RefCell<Vec<Mrs<i32>>>>,
            Rc<BlanketIncDecCpCmp<i32>>,
        > = OverlapIter::new(Rc::new(RefCell::new(src)), 1, Rc::new(t));
        let mut count = 0;

        for (i, res) in iter.enumerate() {
            count = i;
            assert_eq!(res.to_tuple(), checkset[i])
        }
        assert_eq!(count, 8);
    }

    #[test]
    fn iter_test_rev() {
        let checkset = checkset_rev();

        let src = mrs_set();
        let t = BlanketIncDecCpCmp::new();

        let iter: OverlapIter<
            i32,
            i32,
            BlanketIncDecCpCmp<i32>,
            Mrs<i32>,
            Rc<RefCell<Vec<Mrs<i32>>>>,
            Rc<BlanketIncDecCpCmp<i32>>,
        > = OverlapIter::new(Rc::new(RefCell::new(src)), 1, Rc::new(t));

        for (i, res) in iter.rev().enumerate() {
            assert_eq!(res.to_tuple(), checkset[i])
        }
    }

    #[test]
    fn iter_bi() {
        let fwd = checkset();
        let rev = checkset_rev();

        let src = mrs_set();
        let t: BlanketIncDecCpCmp<i32> = BlanketIncDecCpCmp::new();
        let s: OverlapIter<
            i32,
            i32,
            BlanketIncDecCpCmp<i32>,
            Mrs<i32>,
            Rc<RefCell<Vec<Mrs<i32>>>>,
            Rc<BlanketIncDecCpCmp<i32>>,
        > = OverlapIter::new(Rc::new(RefCell::new(src)), 1, Rc::new(t));
        let mut iter = s.into_iter();

        assert_eq!(iter.next().unwrap().to_tuple(), fwd[0]);
        assert_eq!(iter.next_back().unwrap().to_tuple(), rev[0]);

        assert_eq!(iter.next().unwrap().to_tuple(), fwd[1]);
        assert_eq!(iter.next_back().unwrap().to_tuple(), rev[1]);

        assert_eq!(iter.next().unwrap().to_tuple(), fwd[2]);
        assert_eq!(iter.next_back().unwrap().to_tuple(), rev[2]);
        assert_eq!(iter.next().unwrap().to_tuple(), fwd[3]);
        matches!(iter.next_back(), None);
    }

    #[test]
    fn bi_tests_with_defaults() {
        let mut a = AccumulateDefaults::new();
        for r in mrs_set() {
            let (_, check) = a.add_range(&r).unwrap();
            assert_eq!(check.to_tuple_ref(), r.to_tuple_ref());
        }
        let mut iter = a.into_iter();

        let fwd = checkset();
        let rev = checkset_rev();

        assert_eq!(iter.next().unwrap().to_tuple(), fwd[0]);
        assert_eq!(iter.next_back().unwrap().to_tuple(), rev[0]);

        assert_eq!(iter.next().unwrap().to_tuple(), fwd[1]);
        assert_eq!(iter.next_back().unwrap().to_tuple(), rev[1]);

        assert_eq!(iter.next().unwrap().to_tuple(), fwd[2]);
        assert_eq!(iter.next_back().unwrap().to_tuple(), rev[2]);
        assert_eq!(iter.next().unwrap().to_tuple(), fwd[3]);
        matches!(iter.next_back(), None);
        let mut a = AccumulateDefaults::new();
        assert_eq!(a.get_rebound(), &1);
        assert_eq!(a.get_step(), &1);
        a.set_rebound(2);
        a.set_step(2);
        assert_eq!(a.get_rebound(), &2);
        assert_eq!(a.get_step(), &2);
    }

    #[test]
    fn accumulate_struct() {
        let t = TestCmp {};
        let list: Vec<Mrs<Point>> = Vec::new();
        let mut a = Accumulate::new(list, Point { x: 1 }, Point { x: 1 }, t);

        a.add_range(&(..Point { x: 2 }));
        a.add_range(&(Point { x: 1 }..Point { x: 3 }));
        a.add_range(&(Point { x: 3 }..=Point { x: 4 }));
        a.add_range(&(Point { x: 3 }..));
        #[allow(unused_variables)]
        let mut i = a.into_iter();

        assert_eq!(
            i.next().unwrap().to_tuple(),
            (Point { x: i32::MIN }, Point { x: 1 })
        );
        assert_eq!(
            i.next().unwrap().to_tuple(),
            (Point { x: 2 }, Point { x: 2 })
        );
        assert_eq!(
            i.next().unwrap().to_tuple(),
            (Point { x: 3 }, Point { x: 4 })
        );
        assert_eq!(
            i.next().unwrap().to_tuple(),
            (Point { x: 5 }, Point { x: i32::MAX })
        );
        matches!(i.next(), None);
        a = Accumulate::new(Vec::new(), Point { x: 1 }, Point { x: 1 }, TestCmp {});

        a.set_rebound(Point { x: 2 });
        assert_eq!(a.get_rebound(), &Point { x: 2 });
        a.set_step(Point { x: 2 });
        assert_eq!(a.get_step(), &Point { x: 2 });
    }
}
