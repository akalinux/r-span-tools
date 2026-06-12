#![cfg(test)]

use std::ops::RangeInclusive;

use common_range_tools::{
    AnyIncDecCpCmp, Column, Columns, Consolidate, ConsolidateChecker, ConsolidationOrder,
    GetBeginEnd, Intersector, Mrs, MrsFactory, NumberIncDecCpCmp, RangeRelation, RiFactory,
};

use crate::iter_tests::mrs_set;

mod iter_tests {

    use std::ops::RangeInclusive;

    use common_range_tools::{
        CpCmp, DefaultValues, GetBeginEnd, IncDecCpCmp, Intersector, Mrs, MrsFactory,
        NumberIncDecCpCmp, OverlapIter, RiFactory,
    };

    fn checkset() -> Vec<(i32, i32)> {
        return vec![
            (0, 0),   // 0
            (1, 2),   // 1
            (3, 3),   // 2
            (4, 5),   // 3
            (6, 6),   // 4
            (8, 11),  // 5
            (13, 14), // 6
            (15, 19), // 7
            (20, 22), // 8
        ];
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

    fn checkset_rev() -> [(i32, i32); 9] {
        return [
            (20, 22),
            (15, 19),
            (13, 14),
            (8, 11),
            (6, 6),
            (4, 5),
            (3, 3),
            (1, 2),
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
        fn cp_v(&self, v: &Point) -> Point {
            return *v;
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Point {
        x: i32,
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
        assert_eq!(t.cp_v(&Point { x: 2 }), Point { x: 2 });
        let list: Vec<Mrs<Point>> = Vec::new();
        let mut a = Intersector::new(list, Point { x: 1 }, Point { x: 1 }, t, MrsFactory::new());

        a.add_range(&(..=Point { x: 2 }));
        a.add_range(&(Point { x: 1 }..=Point { x: 3 }));
        a.add_range(&(Point { x: 3 }..=Point { x: 4 }));
        a.add_range(&(Point { x: 3 }..));
        #[allow(unused_variables)]
        let mut i = a.into_iter();

        assert_eq!(
            i.next().unwrap().to_tuple(),
            (Point { x: i32::MIN }, Point { x: 0 })
        );
        assert_eq!(
            i.next().unwrap().to_tuple(),
            (Point { x: 1 }, Point { x: 2 })
        );
        assert_eq!(
            i.next().unwrap().to_tuple(),
            (Point { x: 3 }, Point { x: 3 })
        );
        assert_eq!(
            i.next().unwrap().to_tuple(),
            (Point { x: 4 }, Point { x: 4 })
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

    for rel in [ConsolidationOrder::Forward, ConsolidationOrder::Reverse].into_iter() {
        assert!(rel.check_direction(&RangeRelation::Invalid(())).is_err());
    }

    assert!(
        ConsolidationOrder::Reverse
            .check_direction(&RangeRelation::After(()))
            .is_ok()
    );
    assert!(
        ConsolidationOrder::Forward
            .check_direction(&RangeRelation::After(()))
            .is_err()
    );
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
    let mut iter = Consolidate::any_defaults(mrs_set().into_iter(), 0, 22, 1);

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

    let (mut range, _, mut src) = iter.next().unwrap();
    assert_eq!(range.to_tuple(), (1, 1));
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
            assert_eq!(rows.len(), 0);
        }
        Err(msg) => panic!("Did not expect error, got: {}", msg),
    }
    (range, _, src) = iter.next().unwrap();
    assert_eq!(range.to_tuple(), (2, 2));
    assert_eq!(src.len(), 2);

    match &src[0] {
        Ok(rows) => {
            let con = rows[0].as_ref();
            assert_eq!(con.to_tuple_ref(), (&1, &2));
            assert_eq!(rows.len(), 1);
            let src = con.src();
            assert_eq!(src.len(), 2);
            assert_eq!(src[0].0, 0);
            assert_eq!(src[0].1.to_tuple_ref(), (&1, &2));
            assert_eq!(src[1].0, 1);
            assert_eq!(src[1].1.to_tuple_ref(), (&1, &1));
        }
        Err(msg) => panic!("Did not expect error, got: {}", msg),
    }

    match &src[1] {
        Ok(rows) => {
            let con = rows[0].as_ref();
            assert_eq!(con.to_tuple_ref(), (&2, &3));
            assert_eq!(rows.len(), 1);
            let src = con.src();
            assert_eq!(src.len(), 2);
            assert_eq!(src[0].0, 0);
            assert_eq!(src[0].1.to_tuple_ref(), (&2, &3));
            assert_eq!(src[1].0, 1);
            assert_eq!(src[1].1.to_tuple_ref(), (&2, &2));
        }
        Err(msg) => panic!("Did not expect error, got: {}", msg),
    }
    (range, _, src) = iter.next().unwrap();
    assert_eq!(range.to_tuple(), (3, 3));
    assert_eq!(src.len(), 2);
    assert!(iter.next().is_none());
}

#[test]
fn column_tests_positive() {
    let t = NumberIncDecCpCmp::defaults();
    let con = Consolidate::new(
        vec![Mrs::new(1, 2), Mrs::new(1, 1), Mrs::new(3, 3)].into_iter(),
        t,
        RiFactory::new(),
        1,
    );
    let checker = ConsolidateChecker::new(ConsolidationOrder::Forward, con);

    let mut isec = Intersector::new(Vec::new(), 1, 1, t, MrsFactory::new());
    let res = Column::new(&mut isec, checker);
    assert!(res.is_ok());
    let mut col = unsafe { res.unwrap_unchecked() };
    let mut iter = isec.into_iter();
    let mut pos = iter.next().unwrap();
    assert_eq!(pos.to_tuple_ref(), (&1, &2));
    assert!(!col.update_column(&pos, &mut iter, false));
    let mut row = unsafe { col.filter_column(&pos).unwrap_unchecked() };
    assert_eq!(row.len(), 1);
    assert_eq!(row[0].as_ref().to_tuple_ref(), (&1, &2));
    assert_eq!(row[0].src().len(), 2);
    assert_eq!(&row[0].src()[0].0, &0);
    assert_eq!(&row[0].src()[1].0, &1);

    assert_eq!(row[0].src()[0].1.to_tuple_ref(), (&1, &2));
    assert_eq!(row[0].src()[1].1.to_tuple_ref(), (&1, &1));
    assert!(iter.next().is_none());
    assert_eq!(iter.ln().1.unwrap().to_tuple_ref(), (&1, &2));

    assert!(col.update_column(&pos, &mut iter, true));
    pos = iter.recompute_next().unwrap();
    assert_eq!(pos.to_tuple_ref(), (&3, &3));
    row = unsafe { col.filter_column(&pos).unwrap_unchecked() };
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
fn columns_any_constructor() {
    assert!(
        Columns::any(ConsolidationOrder::Forward, AnyIncDecCpCmp::new(1, 3), 1, 1)
            .add_column(vec![Mrs::new(1, 2), Mrs::new(1, 1), Mrs::new(3, 3)].into_iter())
            .is_ok()
    );
    assert!(
        Columns::any(ConsolidationOrder::Forward, AnyIncDecCpCmp::new(1, 1), 1, 1)
            .add_column(vec![Mrs::new(1, 2), Mrs::new(1, 1), Mrs::new(3, 3)].into_iter())
            .is_err()
    );
    assert!(
        Columns::any(ConsolidationOrder::Reverse, AnyIncDecCpCmp::new(0, 9), 1, 1)
            .add_column(vec![Mrs::new(1, 2), Mrs::new(1, 1), Mrs::new(3, 3)].into_iter())
            .is_err()
    );

    let cols = Columns::any(ConsolidationOrder::Forward, AnyIncDecCpCmp::new(0, 9), 1, 1);
    assert!(
        cols.add_column(vec![Mrs::new(1, 2), Mrs::new(1, 1), Mrs::new(3, 3)].into_iter())
            .is_ok()
    );
    assert!(
        cols.add_column(vec![Mrs::new(-11, 2), Mrs::new(1, 1), Mrs::new(3, 3)].into_iter())
            .is_err()
    );
    assert!(cols.add_column(vec![].into_iter()).is_err());
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

#[test]
fn constructor_num_sr_tests() {
    Intersector::num_sr(1);
}
#[test]
fn consoldaite_with_gap() {
    for (ia, ib, a, b) in [
        (
            vec![Mrs::new(3, 4), Mrs::new(5, 6)].into_iter(),
            vec![Mrs::new(1, 2), Mrs::new(7, 8)].into_iter(),
            0,
            1,
        ),
        (
            vec![Mrs::new(1, 2), Mrs::new(7, 8)].into_iter(),
            vec![Mrs::new(3, 4), Mrs::new(5, 6)].into_iter(),
            1,
            0,
        ),
    ] {
        let cols = Columns::num_defaults();
        assert!(cols.add_column(ia).is_ok());
        assert!(cols.add_column(ib).is_ok());
        let mut iter = cols.into_iter();

        let (mut range, _, mut cols) = iter.next().unwrap();
        assert_eq!(range.to_tuple(), (1, 2));
        assert_eq!(cols.len(), 2);
        if let Ok(col) = &cols[a] {
            assert_eq!(col.len(), 0);
        }
        if let Ok(col) = &cols[b] {
            assert_eq!(col.len(), 1);
            assert_eq!(col[0].as_ref().to_tuple_ref(), (&1, &2));
        }

        (range, _, cols) = iter.next().unwrap();
        assert_eq!(range.to_tuple(), (3, 4));
        if let Ok(col) = &cols[a] {
            assert_eq!(col.len(), 1);
            assert_eq!(col[0].as_ref().to_tuple_ref(), (&3, &4));
        }
        if let Ok(col) = &cols[b] {
            assert_eq!(col.len(), 0);
        }

        (range, _, cols) = iter.next().unwrap();
        assert_eq!(range.to_tuple(), (5, 6));
        if let Ok(col) = &cols[a] {
            assert_eq!(col.len(), 1);
            assert_eq!(col[0].as_ref().to_tuple_ref(), (&5, &6));
        }
        if let Ok(col) = &cols[b] {
            assert_eq!(col.len(), 0);
        }

        (range, _, cols) = iter.next().unwrap();
        assert_eq!(range.to_tuple(), (7, 8));
        if let Ok(col) = &cols[a] {
            assert_eq!(col.len(), 0);
        }
        if let Ok(col) = &cols[b] {
            assert_eq!(col.len(), 1);
            assert_eq!(col[0].as_ref().to_tuple_ref(), (&7, &8));
        }
        assert!(iter.next().is_none());
    }
}

#[test]
fn num_defaults_columns_rev() {
    for (ia, ib, a, b) in [
        (
            vec![Mrs::new(5, 6), Mrs::new(3, 4)].into_iter(),
            vec![Mrs::new(7, 8), Mrs::new(1, 2)].into_iter(),
            0,
            1,
        ),
        (
            vec![Mrs::new(7, 8), Mrs::new(1, 2)].into_iter(),
            vec![Mrs::new(5, 6), Mrs::new(3, 4)].into_iter(),
            1,
            0,
        ),
    ] {
        let cols = Columns::num_defaults_rev();
        assert!(cols.add_column(ia).is_ok());
        assert!(cols.add_column(ib).is_ok());
        let mut iter = cols.into_iter();

        let (mut range, _, mut cols) = iter.next().unwrap();
        assert_eq!(range.to_tuple(), (7, 8));
        assert_eq!(cols.len(), 2);
        if let Ok(col) = &cols[a] {
            assert_eq!(col.len(), 0);
        }
        if let Ok(col) = &cols[b] {
            assert_eq!(col.len(), 1);
            assert_eq!(col[0].as_ref().to_tuple_ref(), (&7, &8));
        }
        (range, _, cols) = iter.next().unwrap();
        assert_eq!(range.to_tuple(), (5, 6));
        if let Ok(col) = &cols[a] {
            assert_eq!(col.len(), 1);
            assert_eq!(col[0].as_ref().to_tuple_ref(), (&5, &6));
        }
        if let Ok(col) = &cols[b] {
            assert_eq!(col.len(), 0);
        }
        (range, _, cols) = iter.next().unwrap();
        assert_eq!(range.to_tuple(), (3, 4));
        if let Ok(col) = &cols[a] {
            assert_eq!(col.len(), 1);
            assert_eq!(col[0].as_ref().to_tuple_ref(), (&3, &4));
        }
        if let Ok(col) = &cols[b] {
            assert_eq!(col.len(), 0);
        }
        (range, _, cols) = iter.next().unwrap();
        assert_eq!(range.to_tuple(), (1, 2));
        if let Ok(col) = &cols[a] {
            assert_eq!(col.len(), 0);
        }
        if let Ok(col) = &cols[b] {
            assert_eq!(col.len(), 1);
            assert_eq!(col[0].as_ref().to_tuple_ref(), (&1, &2));
        }
        assert!(iter.next().is_none());
    }
}

#[test]
fn sticky_columns_test() {
    let cols = Columns::num_defaults();
    assert!(cols.add_column(vec![Mrs::new(0, 3)].into_iter()).is_ok());
    assert!(
        cols.add_column(
            vec![
                Mrs::new(0, 0),
                Mrs::new(1, 1),
                Mrs::new(2, 2),
                Mrs::new(3, 3),
            ]
            .into_iter()
        )
        .is_ok()
    );
    for (i, row) in cols.into_iter().enumerate() {
        assert_eq!(row.0.to_tuple_ref(), (&i, &i));
    }
    let cols = Columns::num_sr(1);
    assert!(cols.add_column(vec![Mrs::new(0, 3)].into_iter()).is_ok());

    assert!(
        cols.add_column(
            vec![
                Mrs::new(0, 0),
                Mrs::new(1, 1),
                Mrs::new(2, 2),
                Mrs::new(3, 3),
            ]
            .into_iter()
        )
        .is_ok()
    );
    for (i, row) in cols.into_iter().enumerate() {
        assert_eq!(row.0.to_tuple_ref(), (&i, &i));
    }
    let cols = Columns::num_sr(1);
    assert!(cols.add_column(vec![Mrs::new(0, 3)].into_iter()).is_ok());
    // code coverage!
    cols.into_iter().into_inner();
}

#[test]
fn test_redo_next() {
    let mut isec = Intersector::num_defaults();
    isec.add_range(&(1..=1));
    isec.add_range(&(1..=1));
    let mut iter = isec.into_iter();
    let mut next = iter.next();
    assert!(next.is_some());
    assert_eq!(next.unwrap().to_tuple(), (1, 1));
    next = iter.next();
    assert!(next.is_none());
    assert!(iter.update_column(1, 1..=2).is_ok());
    next = iter.recompute_next();
    assert!(next.is_some());
    assert_eq!(next.unwrap().to_tuple(), (2, 2));
    assert!(iter.next().is_none());
    assert!(iter.update_column(0, 3..=3).is_ok());
    next = iter.recompute_next();
    assert!(next.is_some());
    assert_eq!(next.unwrap().to_tuple(), (3, 3));
    assert!(iter.next().is_none());
}

#[test]
fn test_redo_next_rev() {
    let mut isec = Intersector::num_defaults();
    isec.add_range(&(3..=3));
    isec.add_range(&(3..=3));
    let mut iter = isec.into_iter();

    let mut back = iter.next_back();
    assert!(back.is_some());
    assert_eq!(back.unwrap().to_tuple(), (3, 3));
    assert!(iter.next_back().is_none());
    assert!(iter.update_column(1, 2..=3).is_ok());
    back = iter.recompute_back();
    assert!(back.is_some());
    assert_eq!(back.unwrap().to_tuple(), (2, 2));
    assert!(iter.next_back().is_none());
    assert!(iter.update_column(0, 1..=3).is_ok());
    back = iter.recompute_back();
    assert!(back.is_some());
    assert_eq!(back.unwrap().to_tuple(), (1, 1));
    assert!(iter.next_back().is_none());
}

#[test]
fn test_redo_next_overlaps() {
    let mut isec = Intersector::num_defaults();
    isec.add_range(&(2..=3));
    isec.add_range(&(1..=2));
    let mut iter = isec.into_iter();
    let mut next = iter.next();
    assert!(next.is_some());
    assert_eq!(next.unwrap().to_tuple(), (1, 1));
    assert!(iter.update_column(1, 2..=3).is_ok());
    next = iter.recompute_next();
    assert!(next.is_some());
    assert_eq!(next.unwrap().to_tuple(), (2, 3));

    assert!(iter.next().is_none());
}

#[test]
fn from_tests() {
    let cmp = [0..=0, 1..=2, 3..=5];
    for (i, r) in Intersector::num_from(&[0..=2, 1..=5]).enumerate() {
        assert_eq!(r.to_tuple_ref(), cmp[i].to_tuple_ref());
    }
    for (i, r) in Intersector::num_sr_from(1, &[0..=2, 1..=5]).enumerate() {
        assert_eq!(r.to_tuple_ref(), cmp[i].to_tuple_ref());
    }
    for (i, r) in Intersector::any_from(1, 1, 0, 5, &[0..=2, 1..=5]).enumerate() {
        assert_eq!(r.to_tuple_ref(), cmp[i].to_tuple_ref());
    }
    let cmp = [5..=5, 1..=4, 0..=0];
    for (i, r) in Intersector::num_from(&[0..=4, 1..=5]).rev().enumerate() {
        assert_eq!(r.to_tuple_ref(), cmp[i].to_tuple_ref());
    }
}

#[test]
fn consolidate_error_tests() {
    let mut checker = ConsolidateChecker::new(
        ConsolidationOrder::Forward,
        Consolidate::num_defaults([0..=3, 2..=5, 6..=6, 7..=7, 1..=1].into_iter()),
    );
    let mut next = checker.next().unwrap();
    assert!(next.is_ok());
    let mut row = unsafe { next.unwrap_unchecked() };
    assert_eq!(row.to_tuple_ref(), (&0, &5));
    next = checker.next().unwrap();
    assert!(next.is_ok());
    row = unsafe { next.unwrap_unchecked() };
    assert_eq!(row.to_tuple_ref(), (&6, &6));

    next = checker.next().unwrap();

    assert!(next.is_err());

    (_, row) = unsafe { next.unwrap_err_unchecked() };
    assert_eq!(row.to_tuple_ref(), (&1, &7));
    assert_eq!(row.src().len(), 2);
    assert_eq!(row.src()[0], (3, 7..=7));
    assert_eq!(row.src()[1], (4, 1..=1));
    assert!(checker.next().is_none());
}

#[test]
fn columns_negative_test_out_of_order() {
    let cols = Columns::num_defaults();
    assert!(
        cols.add_column(vec![0..=3, 2..=5, 6..=6, 7..=7, 1..=1, 8..=9].into_iter())
            .is_ok()
    );
    let mut iter = cols.into_iter();
    let mut row = iter.next().unwrap();
    assert_eq!(row.0, 0..=5);

    row = iter.next().unwrap();
    assert!(row.1.is_ok());
    assert_eq!(row.0, 6..=6);
    row = iter.next().unwrap();
    assert!(row.1.is_err());
    assert_eq!(row.0, 7..=7);
    assert!(row.2[0].is_err());
    assert!(row.1.is_err());
    assert_eq!(iter.get_column(0).unwrap().get_rows()[0].src().len(), 2);
    row = iter.next().unwrap();
    assert!(row.2[0].is_ok());
    assert!(row.1.is_ok());
    assert_eq!(row.0, 8..=9);
}

#[test]
fn from_examples() {
    let mut isec = Intersector::num(
        1, // step
        1, // rebound
        0, // min
        8, // max
    );
    let range: std::ops::Range<i32> = 1..4;
    assert!(isec.add_range(&range).is_some());

    let range_inclusive: std::ops::RangeInclusive<i32> = 3..=5;
    assert!(isec.add_range(&range_inclusive).is_some());

    let min_to_end: std::ops::RangeToInclusive<i32> = ..=7;
    assert!(isec.add_range(&min_to_end).is_some());

    let min_max: std::ops::RangeFull = ..;
    assert!(isec.add_range(&min_max).is_some());
    println!("First:");
    let mut iter = isec.into_iter();

    println!("Second:");
    assert_eq!(iter.next().unwrap().to_tuple_ref(), (&0, &0));

    println!("Third:");
    assert_eq!(iter.next().unwrap().to_tuple_ref(), (&1, &2));

    println!("Fourth:");
    assert_eq!(iter.next().unwrap().to_tuple_ref(), (&3, &3));
    assert_eq!(iter.next().unwrap().to_tuple_ref(), (&4, &5));
    assert_eq!(iter.next().unwrap().to_tuple_ref(), (&6, &7));
    assert_eq!(iter.next().unwrap().to_tuple_ref(), (&8, &8));
}

#[test]
fn reverse_iter_complex_test() {
    let mut isec = Intersector::num(
        1, // step
        1, // rebound
        0, // min
        8, // max
    );
    let range: std::ops::Range<i32> = 1..4;
    assert!(isec.add_range(&range).is_some());

    let range_inclusive: std::ops::RangeInclusive<i32> = 3..=5;
    assert!(isec.add_range(&range_inclusive).is_some());

    let min_to_end: std::ops::RangeToInclusive<i32> = ..=7;
    assert!(isec.add_range(&min_to_end).is_some());

    let min_max: std::ops::RangeFull = ..;
    assert!(isec.add_range(&min_max).is_some());
    println!("First:");
    let mut iter = isec.into_iter().rev();
    println!("Second:");
    assert_eq!(iter.next().unwrap().to_tuple_ref(), (&8, &8));

    println!("Third:");
    assert_eq!(iter.next().unwrap().to_tuple_ref(), (&6, &7));

    println!("Fourth:");
    assert_eq!(iter.next().unwrap().to_tuple_ref(), (&4, &5));
    assert_eq!(iter.next().unwrap().to_tuple_ref(), (&3, &3));
    assert_eq!(iter.next().unwrap().to_tuple_ref(), (&1, &2));
    assert_eq!(iter.next().unwrap().to_tuple_ref(), (&0, &0));
}
