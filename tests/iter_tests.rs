#![cfg(test)]

use std::ops::RangeInclusive;

use common_range_tools::{
    Column, Columns, Consolidate, ConsolidateChecker, ConsolidationOrder, GetBeginEnd, Intersector,
    Mrs, MrsFactory, NumberIncDecCpCmp, RangeRelation, RiFactory,
};

use crate::iter_tests::mrs_set;

mod iter_tests {

    use std::ops::RangeInclusive;

    use common_range_tools::{
        CpCmp, DefaultValues, GetBeginEnd, IncDecCpCmp, Intersector, Mrs, MrsFactory,
        NumberIncDecCpCmp, OverlapIter, RiFactory,
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

    impl CpCmp<Point> for TestCmp {
        fn cp(&self, v: &Point) -> Point {
            return v.clone();
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

    impl IncDecCpCmp<Point, Point> for TestCmp {
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

        let iter = OverlapIter::new(src, 1, t, RiFactory::new());
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

        let iter = OverlapIter::new(src, 1, t, MrsFactory::new());

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

        let mut iter = OverlapIter::new(src, 1, t, RiFactory::new());

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
        let mut a = Intersector::any(1, 1, 0, 22);
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
        let mut a = Intersector::num_defaults();
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
        let mut a = Intersector::num(
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
        let mut a = Intersector::new(list, Point { x: 1 }, Point { x: 1 }, t, MrsFactory::new());

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
        a = Intersector::new(
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
    matches!(
        ConsolidationOrder::Forward.check_direction(&RangeRelation::After(())),
        Err("Out of Forward Sequence, Expected: Before|Last|Overlap, got: After")
    );
    matches!(
        ConsolidationOrder::Reverse.check_direction(&&RangeRelation::Before(())),
        Err("Out of Reverse Sequence, Expected: After|Last|Overlap, got: Before")
    );

    for d in [ConsolidationOrder::Forward, ConsolidationOrder::Reverse] {
        for r in [RangeRelation::Overlap(()), RangeRelation::Last(())] {
            matches!(d.check_direction(&r), Ok(()));
        }
    }

    let t = NumberIncDecCpCmp::defaults();
    assert!(ConsolidationOrder::Forward.is_beyond(&Mrs::new(0, 2), &Mrs::new(0, 1), &t));
    assert!(!ConsolidationOrder::Forward.is_beyond(&Mrs::new(0, 2), &Mrs::new(0, 3), &t));

    assert!(ConsolidationOrder::Reverse.is_beyond(&Mrs::new(0, 2), &Mrs::new(1, 1), &t));
    assert!(!ConsolidationOrder::Reverse.is_beyond(&Mrs::new(0, 2), &Mrs::new(0, 3), &t));
}

#[test]
fn consolidator_forward_num_tests() {
    let mut iter = Consolidate::num_defaults(mrs_set().into_iter());

    assert_eq!(iter.next().unwrap().unwrap().0, 0..=3);
    assert_eq!(iter.next().unwrap().unwrap().0, 4..=6);
    assert_eq!(iter.next().unwrap().unwrap().0, 8..=11);
    assert_eq!(iter.next().unwrap().unwrap().0, 13..=22);
    assert!(iter.next().is_none());
}

#[test]
fn consolidator_any_defaults() {
    let mut iter = Consolidate::any_defaults(mrs_set().into_iter(), 0, 22);

    assert_eq!(iter.next().unwrap().unwrap().0, 0..=3);
    assert_eq!(iter.next().unwrap().unwrap().0, 4..=6);
    assert_eq!(iter.next().unwrap().unwrap().0, 8..=11);
    assert_eq!(iter.next().unwrap().unwrap().0, 13..=22);
    assert!(iter.next().is_none());
}

#[test]
fn consolidate_check_tests() {
    let mut iter = Consolidate::num_defaults(mrs_set().into_iter())
        .to_consolidate_checker(ConsolidationOrder::Forward);

    let inputs = vec![
        vec![RangeInclusive::new(0, 3), RangeInclusive::new(1, 2)],
        vec![RangeInclusive::new(4, 5), RangeInclusive::new(4, 6)],
        // gap 1 is 7-7
        vec![RangeInclusive::new(8, 11)],
        vec![
            // gap 2 is 12-12
            RangeInclusive::new(13, 22),
            RangeInclusive::new(15, 19),
        ],
    ];

    let mut pos: usize = 0;
    for (i, check) in [0..=3, 4..=6, 8..=11, 13..=22].iter().enumerate() {
        let next = iter.next().unwrap();
        match next {
            Err(_) => panic!("Should not get an error"),
            Ok(raw) => {
                let (r, src) = raw.as_src();
                assert_eq!(r.to_tuple_ref(), check.to_tuple_ref());
                let check = &inputs[i];
                for (idx, cmp) in check.iter().enumerate() {
                    assert_eq!(src[idx].0, pos);
                    assert_eq!(src[idx].1.to_tuple_ref(), cmp.to_tuple_ref());
                    pos += 1;
                }
            }
        }
    }
    assert!(iter.next().is_none());
    iter = Consolidate::num_defaults(mrs_set().into_iter())
        .to_consolidate_checker(ConsolidationOrder::Reverse);
    match iter.next().unwrap() {
        Err(_) => (), // all is well
        Ok(_) => panic!("Expected to error out"),
    }
}

#[test]
fn next_range_wanted_tests() {
    assert!(ConsolidationOrder::Forward.wants_next(&RangeRelation::Last(())));
    assert!(ConsolidationOrder::Forward.wants_next(&RangeRelation::Overlap(())));
    assert!(ConsolidationOrder::Forward.wants_next(&RangeRelation::Before(())));
    assert!(!ConsolidationOrder::Forward.wants_next(&RangeRelation::After(())));

    assert!(ConsolidationOrder::Reverse.wants_next(&RangeRelation::Last(())));
    assert!(ConsolidationOrder::Reverse.wants_next(&RangeRelation::Overlap(())));
    assert!(ConsolidationOrder::Reverse.wants_next(&RangeRelation::After(())));
    assert!(!ConsolidationOrder::Reverse.wants_next(&RangeRelation::Before(())));
}

#[test]
fn colums_forward_num_defaults() {
    let cols = Columns::num_defaults();
    assert!(
        cols.add_column(vec![Mrs::new(1, 2), Mrs::new(1, 1), Mrs::new(3, 3)].into_iter())
            .is_ok(),
    );
    assert!(
        cols.add_column(vec![Mrs::new(2, 3), Mrs::new(2, 2)].into_iter())
            .is_ok(),
    );

    let mut iter = cols.into_iter();

    let (mut range, mut src) = iter.next().unwrap();
    assert_eq!(range.to_tuple(), (1, 2));
    assert_eq!(src.len(), 2);
    match &src[0] {
        Ok(rows) => {
            assert_eq!(rows.len(), 1);
            let con = rows[0].as_ref();
            assert_eq!(con.to_tuple_ref(), (&1, &2));
            let src = con.src();
            assert_eq!(src.len(), 2);
            assert_eq!(src[0].0, 0);
            assert_eq!(src[1].0, 1);
            assert_eq!(src[0].1.to_tuple_ref(), (&1, &2));
            assert_eq!(src[1].1.to_tuple_ref(), (&1, &1));
        }
        Err(msg) => panic!("Did not expect error, got: {}", msg),
    }

    match &src[1] {
        Ok(rows) => {
            assert_eq!(rows.len(), 1);
            let con = rows[0].as_ref();
            assert_eq!(con.to_tuple_ref(), (&2, &3));
            let src = con.src();
            assert_eq!(src.len(), 2);
            assert_eq!(src[0].0, 0);
            assert_eq!(src[1].0, 1);
            assert_eq!(src[0].1.to_tuple_ref(), (&2, &3));
            assert_eq!(src[1].1.to_tuple_ref(), (&2, &2));
        }
        Err(msg) => panic!("Did not expect error, got: {}", msg),
    }
    (range, src) = iter.next().unwrap();
    assert_eq!(range.to_tuple(), (3, 3));
    assert_eq!(src.len(), 2);

    match &src[0] {
        Ok(rows) => {
            let con = rows[0].as_ref();
            assert_eq!(con.to_tuple_ref(), (&3, &3));
            assert_eq!(rows.len(), 1);
            let src = con.src();
            assert_eq!(src.len(), 1);
            assert_eq!(src[0].0, 2);
            assert_eq!(src[0].1.to_tuple_ref(), (&3, &3));
        }
        Err(msg) => panic!("Did not expect error, got: {}", msg),
    }
}

#[test]
fn column_tests() {
    let t = NumberIncDecCpCmp::defaults();
    let con = Consolidate::new(
        vec![Mrs::new(1, 2), Mrs::new(1, 1), Mrs::new(3, 3)].into_iter(),
        t,
        RiFactory::new(),
    );
    let checker = ConsolidateChecker::new(ConsolidationOrder::Forward, con);

    let mut isec = Intersector::new(Vec::new(), 1, 1, t, MrsFactory::new());
    let res = Column::new(&mut isec, checker);
    assert!(res.is_ok());
    let mut col = unsafe { res.unwrap_unchecked() };
    let mut iter = isec.into_iter();
    let mut pos = iter.next().unwrap();
    assert_eq!(pos.to_tuple_ref(), (&1, &2));
    let mut row_res = col.update_column(&pos, &mut iter);
    assert!(row_res.is_ok());
    let mut row = unsafe { row_res.unwrap_unchecked() };
    assert_eq!(row.len(), 1);
    assert_eq!(row[0].as_ref().to_tuple_ref(), (&1, &2));
    assert_eq!(row[0].src().len(), 2);
    assert_eq!(&row[0].src()[0].0, &0);
    assert_eq!(&row[0].src()[1].0, &1);

    assert_eq!(row[0].src()[0].1.to_tuple_ref(), (&1, &2));
    assert_eq!(row[0].src()[1].1.to_tuple_ref(), (&1, &1));
    pos = iter.next().unwrap();
    assert_eq!(pos.to_tuple_ref(), (&3, &3));
    println!("  --- Round 2");
    row_res = col.update_column(&pos, &mut iter);
    assert!(row_res.is_ok());
    row = unsafe { row_res.unwrap_unchecked() };
    assert_eq!(row.len(), 1);
    assert_eq!(row[0].as_ref().to_tuple_ref(), (&3, &3));
    assert_eq!(row[0].src().len(), 1);
    assert_eq!(&row[0].src()[0].0, &2);
    assert_eq!(row[0].src()[0].1.to_tuple_ref(), (&3, &3));
    assert!(iter.next().is_none());
    assert!(!col.in_err());
    let inner = col.to_inner();
    assert!(inner.0.is_ok());
}

#[test]
fn check_position_forward() {
    let t = NumberIncDecCpCmp::defaults();
    assert_eq!(
        ConsolidationOrder::Forward.check_position(&Mrs::new(1, 2), &Mrs::new(1, 1), &t),
        (true, true)
    );
    assert_eq!(
        ConsolidationOrder::Forward.check_position(&Mrs::new(1, 2), &Mrs::new(1, 2), &t),
        (true, false)
    );
    assert_eq!(
        ConsolidationOrder::Forward.check_position(&Mrs::new(3, 3), &Mrs::new(1, 2), &t),
        (false, true)
    );
    assert_eq!(
        ConsolidationOrder::Forward.check_position(&Mrs::new(0, 0), &Mrs::new(1, 2), &t),
        (false, false)
    );
}

#[test]
fn check_position_reverse() {
    let t = NumberIncDecCpCmp::defaults();
    assert_eq!(
        ConsolidationOrder::Reverse.check_position(&Mrs::new(1, 2), &Mrs::new(1, 1), &t),
        (true, false)
    );
    assert_eq!(
        ConsolidationOrder::Reverse.check_position(&Mrs::new(1, 2), &Mrs::new(1, 2), &t),
        (true, false)
    );
    assert_eq!(
        ConsolidationOrder::Reverse.check_position(&Mrs::new(3, 3), &Mrs::new(1, 2), &t),
        (false, false)
    );
    assert_eq!(
        ConsolidationOrder::Reverse.check_position(&Mrs::new(0, 0), &Mrs::new(1, 2), &t),
        (false, true)
    );
}
