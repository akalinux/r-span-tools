#![cfg(test)]

mod iter_tests {

    use std::ops::RangeInclusive;

    use common_range_tools::{
        Accumulate, BlanketIncDecCpCmp, DefaultValues, GetBeginEnd, IncDecCpCmp, Intersector, Mrs,
        OverlapIter, OwnedOverlapIter,
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

    struct TestCmp {}

    impl DefaultValues<Point, Point> for TestCmp {
        fn default_step(&self) -> Point {
            Point { x: 1 }
        }

        fn default_rebound(&self) -> Point {
            Point { x: 1 }
        }
    }
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
            return Point { x: i32::MIN };
        }

        fn max(&self) -> Point {
            return Point { x: i32::MAX };
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
    fn range_set() -> [RangeInclusive<i32>; 7] {
        let res: [RangeInclusive<i32>; 7] = [4..=5, 4..=6, 0..=3, 1..=2, 8..=11, 13..=22, 15..=19];
        return res;
    }

    #[test]
    fn iter_test() {
        let checkset = checkset();

        let src = mrs_set();
        let t = BlanketIncDecCpCmp::new();

        let iter = OverlapIter::new(src.as_slice(), &1, &t);
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

        let iter = OverlapIter::new(src.as_slice(), &1, &t);

        for (i, res) in iter.rev().enumerate() {
            assert_eq!(res.to_tuple(), checkset[i])
        }
    }

    #[test]
    fn iter_bi() {
        let fwd = checkset();
        let rev = checkset_rev();

        let src = mrs_set();
        let t = BlanketIncDecCpCmp::new();
        let s = OverlapIter::new(src.as_slice(), &1, &t);
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
    fn iter_from_vec_test() {
        let checkset = checkset();

        let src = mrs_set();
        let t = BlanketIncDecCpCmp::new();

        let iter = OverlapIter::from_vec(&src, &1, &t);
        for (i, res) in iter.enumerate() {
            assert_eq!(res.to_tuple(), checkset[i])
        }
    }

    #[test]
    fn owned_iter_test() {
        let checkset = checkset();

        let check = mrs_set();
        let t = BlanketIncDecCpCmp::new();
        let iter = OwnedOverlapIter::new(check, 1, t);
        for (i, res) in iter.enumerate() {
            assert_eq!(res, checkset[i])
        }
    }

    #[test]
    fn intersector_test() {
        let checkset = checkset();

        let check = range_set();
        let t = BlanketIncDecCpCmp::new();
        let iter = Intersector::new(&check, t.default_step(), t.default_rebound(), t);
        for (i, res) in iter.enumerate() {
            assert_eq!(res, checkset[i])
        }
    }

    #[test]
    fn intersector_defaults_test() {
        let checkset = checkset();

        let check = range_set();
        let iter = Intersector::defaults(&check);
        for (i, res) in iter.enumerate() {
            assert_eq!(res, checkset[i])
        }
    }

    #[test]
    fn accumulate() {
        let mut a = Accumulate::defaults();
        let src = range_set();
        for mrs in src {
            a.add_range(&mrs);
        }
        let checkset = checkset();
        for (i, res) in a.into_iter().enumerate() {
            assert_eq!(res, checkset[i])
        }
    }

    #[test]
    fn accumulate_struct() {
        let t = TestCmp {};
        let mut a = Accumulate::new(Point { x: 1 }, Point { x: 1 }, t);
        a.add_range(&(..Point { x: 2 }));
        a.add_range(&(Point { x: 1 }..Point { x: 3 }));
        a.add_range(&(Point { x: 3 }..=Point { x: 4 }));
        a.add_range(&(Point { x: 3 }..));
        #[allow(unused_variables)]
        a.add_ranges(&[..], |i, r| return true);

        let mut i = a.into_iter();

        assert_eq!(i.next().unwrap(), (Point { x: i32::MIN }, Point { x: 1 }));
        assert_eq!(i.next().unwrap(), (Point { x: 2 }, Point { x: 2 }));
        assert_eq!(i.next().unwrap(), (Point { x: 3 }, Point { x: 4 }));
        assert_eq!(i.next().unwrap(), (Point { x: 5 }, Point { x: i32::MAX }));
        matches!(i.next(), None);
    }
}
