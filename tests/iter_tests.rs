#![cfg(test)]

use std::rc::Rc;

use common_range_tools::{
    AnyIncDecCpCmp, Consolidate, ConsolidationOrder, NumberIncDecCpCmp, RangeRelation, RiFactory,
    sort_reverse,
};

use crate::iter_tests::mrs_set;

mod iter_tests {

    use std::ops::RangeInclusive;
    use std::{cell::RefCell, rc::Rc};

    use common_range_tools::{
        Accumulate, DefaultValues, GetBeginEnd, IncDecCpCmp, Mrs, MrsFactory, NumberIncDecCpCmp,
        OverlapIter, RiFactory,
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

    pub(crate) fn mrs_set() -> Vec<RangeInclusive<i32>> {
        return vec![
            RangeInclusive::new(0, 3),
            RangeInclusive::new(1, 2),
            RangeInclusive::new(4, 5),
            RangeInclusive::new(4, 6),
            // gap 1 is 7-7
            RangeInclusive::new(8, 11),
            // gap 2 is 12-12
            RangeInclusive::new(13, 22),
            RangeInclusive::new(15, 19),
        ];
    }

    #[test]
    fn iter_test() {
        let checkset = checkset();

        let src = mrs_set();
        let t = NumberIncDecCpCmp::defaults();

        let iter: OverlapIter<
            i32,
            i32,
            NumberIncDecCpCmp<i32>,
            RangeInclusive<i32>,
            Rc<RefCell<Vec<RangeInclusive<i32>>>>,
            Rc<NumberIncDecCpCmp<_>>,
            RiFactory<i32>,
            Rc<RiFactory<i32>>,
        > = OverlapIter::new(
            Rc::new(RefCell::new(src)),
            1,
            Rc::new(t),
            Rc::new(RiFactory::new()),
        );
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

        let src = mrs_set().iter().map(|v| v.clone().into()).collect();
        let t = NumberIncDecCpCmp::defaults();

        let iter: OverlapIter<
            i32,
            i32,
            NumberIncDecCpCmp<i32>,
            Mrs<i32>,
            Rc<RefCell<Vec<Mrs<i32>>>>,
            Rc<NumberIncDecCpCmp<i32>>,
            MrsFactory<i32>,
            Rc<MrsFactory<i32>>,
        > = OverlapIter::new(
            Rc::new(RefCell::new(src)),
            1,
            Rc::new(t),
            Rc::new(MrsFactory::new()),
        );

        for (i, res) in iter.rev().enumerate() {
            assert_eq!(res.to_tuple(), checkset[i])
        }
    }

    #[test]
    fn iter_bi() {
        let fwd = checkset();
        let rev = checkset_rev();

        let src = mrs_set();
        let t: NumberIncDecCpCmp<i32> = NumberIncDecCpCmp::defaults();

        let mut iter: OverlapIter<
            i32,
            i32,
            NumberIncDecCpCmp<i32>,
            RangeInclusive<i32>,
            Rc<RefCell<Vec<RangeInclusive<i32>>>>,
            Rc<NumberIncDecCpCmp<i32>>,
            RiFactory<i32>,
            Rc<RiFactory<i32>>,
        > = OverlapIter::new(
            Rc::new(RefCell::new(src)),
            1,
            Rc::new(t),
            Rc::new(RiFactory::new()),
        );

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
    fn bi_tests_with_any() {
        let mut a = Accumulate::any(1, 1, 0, 22);
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
    }
    #[test]
    fn bi_tests_with_num_defalts() {
        let mut a = Accumulate::num_defaults();
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
    }

    #[test]
    fn bi_tests_with_num() {
        let cmp = NumberIncDecCpCmp::defaults();
        let mut a = Accumulate::num(
            cmp.default_step(),
            cmp.default_rebound(),
            cmp.min(),
            cmp.max(),
        );
        assert_eq!(cmp.max(), a.get_cmp_mut().max());
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
    }

    #[test]
    fn accumulate_struct() {
        let t = TestCmp {};
        let list: Vec<Mrs<Point>> = Vec::new();
        let mut a = Accumulate::new(list, Point { x: 1 }, Point { x: 1 }, t, MrsFactory::new());

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
        a = Accumulate::new(
            Vec::new(),
            Point { x: 1 },
            Point { x: 1 },
            TestCmp {},
            MrsFactory::new(),
        );

        a.set_rebound(Point { x: 2 });
        assert_eq!(a.get_rebound(), &Point { x: 2 });
        a.set_step(Point { x: 2 });
        assert_eq!(a.get_step(), &Point { x: 2 });
    }
}

#[test]
fn consolidation_order_tests() {
    let mut check: Option<RangeRelation<()>> = None;
    matches!(ConsolidationOrder::Forward.check_direction(check), None);

    check = Some(RangeRelation::Before(()));
    matches!(ConsolidationOrder::Forward.check_direction(check), Some(()));
    check = Some(RangeRelation::Last(()));
    matches!(ConsolidationOrder::Forward.check_direction(check), Some(()));
    check = Some(RangeRelation::Overlap(()));
    matches!(ConsolidationOrder::Forward.check_direction(check), Some(()));
    check = Some(RangeRelation::After(()));
    matches!(ConsolidationOrder::Forward.check_direction(check), None);

    check = Some(RangeRelation::After(()));
    matches!(ConsolidationOrder::Reverse.check_direction(check), Some(()));
    check = Some(RangeRelation::Last(()));
    matches!(ConsolidationOrder::Reverse.check_direction(check), Some(()));
    check = Some(RangeRelation::Overlap(()));
    matches!(ConsolidationOrder::Reverse.check_direction(check), Some(()));
    check = Some(RangeRelation::Before(()));
    matches!(ConsolidationOrder::Reverse.check_direction(check), None);
}

#[test]
fn consolidator_forward_num_tests() {
    let cmp = Rc::new(NumberIncDecCpCmp::<i32>::defaults());
    let f = Rc::new(RiFactory::<i32>::new());

    let mut iter = Consolidate::num(ConsolidationOrder::Forward, mrs_set().into_iter(), cmp, f);

    assert_eq!(iter.next().unwrap().0, 0..=3);
    assert_eq!(iter.next().unwrap().0, 4..=6);
    assert_eq!(iter.next().unwrap().0, 8..=11);
    assert_eq!(iter.next().unwrap().0, 13..=22);
    assert!(iter.next().is_none());
}

#[test]
fn consolidator_any_defaults() {
    let mut iter: Consolidate<
        i32,
        i32,
        std::ops::RangeInclusive<i32>,
        RiFactory<i32>,
        std::vec::IntoIter<std::ops::RangeInclusive<i32>>,
        AnyIncDecCpCmp<i32, _>,
        Rc<RiFactory<i32>>,
        Rc<AnyIncDecCpCmp<i32, i32>>,
    > = Consolidate::any_defaults(mrs_set().into_iter(), 0, 22);

    assert_eq!(iter.next().unwrap().0, 0..=3);
    assert_eq!(iter.next().unwrap().0, 4..=6);
    assert_eq!(iter.next().unwrap().0, 8..=11);
    assert_eq!(iter.next().unwrap().0, 13..=22);
    assert!(iter.next().is_none());
}

#[test]
fn consolidator_num_defaults() {
    let mut iter = Consolidate::num_defaults(mrs_set().into_iter());

    assert_eq!(iter.next().unwrap().0, 0..=3);
    assert_eq!(iter.next().unwrap().0, 4..=6);
    assert_eq!(iter.next().unwrap().0, 8..=11);
    assert_eq!(iter.next().unwrap().0, 13..=22);
    assert!(iter.next().is_none());
}

#[test]
fn consolidator_reverse_tests() {
    let cmp = Rc::new(NumberIncDecCpCmp::<i32>::defaults());
    let f = Rc::new(RiFactory::<i32>::new());

    let mut src = mrs_set();
    src.sort_by(|a, b| sort_reverse(a, b, cmp.as_ref()));
    let mut iter = Consolidate::num(
        ConsolidationOrder::Reverse,
        src.into_iter(),
        cmp.clone(),
        f.clone(),
    );

    assert_eq!(iter.next().unwrap().0, 13..=22);
    assert_eq!(iter.next().unwrap().0, 8..=11);
    assert_eq!(iter.next().unwrap().0, 4..=6);
    assert_eq!(iter.next().unwrap().0, 0..=3);
    assert!(iter.next().is_none());
    iter = Consolidate::num(ConsolidationOrder::Reverse, mrs_set().into_iter(), cmp, f);
    assert!(iter.next().is_none());
}
