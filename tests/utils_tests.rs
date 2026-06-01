#![cfg(test)]
mod util_tests {

    use common_range_tools::{
        GetBeginEnd, Mrs, MrsFactory, RangeRelation, auto_range, builder::NumberIncDecCpCmp,
        consolidate, first_range_begin_end, grow, last_range_begin_end, next_range_begin_end,
        next_smallest_range, previous_range_begin_end, previous_smallest_range,
        range_bounds_to_values, range_relation, sort_forward, sort_reverse,
    };

    #[test]
    fn test_first_range() {
        let t = NumberIncDecCpCmp::defaults();

        // Empty set test
        matches!(
            first_range_begin_end::<i32, NumberIncDecCpCmp<i32>, Mrs<i32>>(&[], &t),
            None
        );

        matches!(
            first_range_begin_end::<i32, NumberIncDecCpCmp<i32>, Mrs<i32>>(&[Mrs::new(0, -1)], &t),
            None
        );

        matches!(
            first_range_begin_end(&[Mrs::new(1, 1), Mrs::new(6, 7),], &t),
            Some((1, 1))
        );

        matches!(first_range_begin_end(&[Mrs::new(1, 1)], &t), Some((1, 1)));

        matches!(
            first_range_begin_end(&[Mrs::new(5, 7), Mrs::new(4, 7),], &t),
            Some((4, 5))
        );

        matches!(
            first_range_begin_end(&[Mrs::new(5, 7), Mrs::new(6, 7),], &t),
            Some((5, 6))
        );

        matches!(
            first_range_begin_end(
                &[
                    Mrs::new(5, 7),
                    Mrs::new(0, 2),
                    Mrs::new(0, 1),
                    Mrs::new(0, 0),
                    Mrs::new(2, -1), // this should be invalid
                ],
                &t
            ),
            Some((0, 0))
        );

        matches!(first_range_begin_end(&mrs_set_a(), &t), Some((0, 1)));
    }

    fn mrs_known_bad() -> Vec<Mrs<i32>> {
        return vec![
            Mrs::new(5, 7),
            Mrs::new(0, 2),
            Mrs::new(0, 1),
            Mrs::new(0, 0),
            Mrs::new(2, -1), // this should be invalid
        ];
    }
    #[test]
    fn last_range_test() {
        let t = NumberIncDecCpCmp::defaults();

        // Empty set test
        matches!(
            last_range_begin_end::<i32, NumberIncDecCpCmp<i32>, Mrs<i32>>(&[], &t),
            None
        );

        matches!(
            last_range_begin_end::<i32, NumberIncDecCpCmp<i32>, Mrs<i32>>(&[Mrs::new(0, -1)], &t),
            None
        );

        matches!(last_range_begin_end(&mrs_known_bad(), &t), Some((5, 7)));

        matches!(
            last_range_begin_end(&[Mrs::new(5, 7), Mrs::new(4, 7),], &t),
            Some((5, 7))
        );

        matches!(
            last_range_begin_end(&[Mrs::new(4, 11), Mrs::new(4, 7),], &t),
            Some((7, 11))
        );
        matches!(last_range_begin_end(&[Mrs::new(4, 11)], &t), Some((4, 11)));
        matches!(last_range_begin_end(&mrs_set_a(), &t), Some((19, 22)));
    }

    fn checkset_a() -> Vec<(i32, i32)> {
        return vec![
            (3, 3),
            (4, 5),
            (6, 6),
            (8, 11),
            (13, 15),
            (16, 19),
            (20, 22),
        ];
    }

    fn mrs_set_a() -> Vec<Mrs<i32>> {
        return vec![
            Mrs::new(0, 3),
            Mrs::new(1, 2),
            Mrs::new(4, 5),
            Mrs::new(4, 6),
            // gap 2 is 12-12
            Mrs::new(8, 11),
            // gap 1 is 7-7
            Mrs::new(13, 22),
            Mrs::new(15, 19),
        ];
    }

    fn checkset_a_reversed() -> Vec<(i32, i32)> {
        return vec![
            (19, 21),
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

    fn mrs_set_b() -> Vec<Mrs<i32>> {
        return vec![
            // reversing  the order of the gap for coverage
            Mrs::new(15, 19),
            Mrs::new(13, 22),
            // order should never mater
            Mrs::new(8, 11),
            Mrs::new(11, 1),
        ];
    }
    #[test]
    fn next_range_begin_end_tests() {
        let t = NumberIncDecCpCmp::defaults();
        let checkset = checkset_a();

        let check = mrs_set_a();
        let mut point = 3;
        for (a, b) in checkset {
            assert_eq!(
                next_range_begin_end(&point, &check, &t),
                Some((a.clone(), b.clone()))
            );
            point = b + 1;
        }
    }

    #[test]
    fn range_conversion() {
        let t = NumberIncDecCpCmp::defaults();

        assert_eq!(range_bounds_to_values(&(1_i32..=2), &1, &t), Some((1, 2)));
        assert_eq!(
            range_bounds_to_values(&(1.0..f32::INFINITY), &1.0, &NumberIncDecCpCmp::defaults()),
            None
        );
    }

    #[test]
    fn previous_smallest_range_test() {
        let t = NumberIncDecCpCmp::defaults();
        let src = mrs_set_b();

        let (begin, end) = previous_smallest_range(&0, &22, &src, &t);

        assert_eq!((begin, end), (19, 22))
    }

    #[test]
    fn previous_range_begin_end_tests() {
        let t = NumberIncDecCpCmp::defaults();
        let checked = checkset_a_reversed();
        let src = mrs_set_a();

        let mut end = 21;

        for (a, z) in checked {
            let res = previous_range_begin_end(&end, &src, &t);
            assert_eq!(res, Some((a, z)));
            end = a - 1;
        }
    }

    #[test]
    fn next_smallest_range_test() {
        let t = NumberIncDecCpCmp::defaults();
        let mut src = mrs_set_a();

        let (mut begin, mut end) = next_smallest_range(&0, &22, &src, &t);
        assert_eq!((begin, end), (0, 1));

        (begin, end) = next_smallest_range(&0, &1, &src, &t);
        assert_eq!((begin, end), (0, 1));

        src = vec![Mrs::new(5, 7), Mrs::new(4, 7)];
        let mut valid = Vec::new();
        for s in src.as_slice() {
            valid.push(s);
        }

        (begin, end) = next_smallest_range(&4, &7, &src, &t);
        assert_eq!((begin, end), (4, 5));
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
    fn sort_more() {
        let mut set = mrs_set();
        set.push(Mrs::new(8, 11)); // force equal ctest
        let l = NumberIncDecCpCmp::defaults();
        set.sort_by(|a, b| sort_forward(a, b, &l));
        let expected = vec![
            Mrs::new(0, 3),
            Mrs::new(1, 2),
            Mrs::new(4, 6),
            Mrs::new(4, 5),
            Mrs::new(8, 11),
            Mrs::new(8, 11),
            Mrs::new(13, 22),
            Mrs::new(15, 19),
        ];
        for (i, exp) in expected.iter().enumerate() {
            assert_eq!(exp.to_tuple_ref(), set[i].to_tuple_ref())
        }
    }

    #[test]
    fn sort() {
        let l = NumberIncDecCpCmp::defaults();
        // Sorting in consolidation order.
        let correct = vec![
            Mrs::new(8, 11),
            Mrs::new(8, 9),
            Mrs::new(13, 22),
            Mrs::new(15, 20),
            Mrs::new(15, 20),
            Mrs::new(15, 19),
        ];
        let mut check = vec![
            Mrs::new(15, 20),
            Mrs::new(15, 19),
            Mrs::new(13, 22),
            Mrs::new(8, 11),
            Mrs::new(15, 20),
            Mrs::new(8, 9),
        ];
        check.sort_by(|a, b| sort_forward(a, b, &l));
        for (i, good) in correct.iter().enumerate() {
            assert_eq!(check[i].to_tuple_ref(), good.to_tuple_ref());
        }
    }

    #[test]
    fn sort_reverse_test() {
        let l = NumberIncDecCpCmp::defaults();
        // Sorting in consolidation order.
        let correct = vec![
            Mrs::new(13, 22),
            Mrs::new(15, 20),
            Mrs::new(16, 20),
            Mrs::new(16, 20),
            Mrs::new(15, 19),
            Mrs::new(8, 11),
            Mrs::new(8, 9),
            Mrs::new(8, 9),
        ];
        let mut check = vec![
            Mrs::new(13, 22),
            Mrs::new(16, 20),
            Mrs::new(8, 11),
            Mrs::new(8, 9),
            Mrs::new(15, 19),
            Mrs::new(15, 20),
            Mrs::new(16, 20),
            Mrs::new(8, 9),
        ];
        check.sort_by(|a, b| sort_reverse(a, b, &l));
        for (i, good) in correct.iter().enumerate() {
            assert_eq!(check[i].to_tuple_ref(), good.to_tuple_ref());
        }
    }

    #[test]
    fn relations_test() {
        let t = NumberIncDecCpCmp::defaults();
        // Compare Range Positional relationships
        assert!(matches!(
            range_relation(&Mrs::new(1, 2), &Mrs::new(1, 2), &t),
            RangeRelation::Overlap(())
        )); // a and b overlap
        assert!(matches!(
            range_relation(&Mrs::new(1, 1), &Mrs::new(1, 2), &t),
            RangeRelation::Overlap(())
        )); // a and b overlap
        assert!(matches!(
            range_relation(&Mrs::new(2, 2), &Mrs::new(1, 2), &t),
            RangeRelation::Overlap(())
        )); // a and b overlap
        assert!(matches!(
            range_relation(&Mrs::new(0, 0), &Mrs::new(1, 2), &t),
            RangeRelation::Before(())
        )); // a is before b
        assert!(matches!(
            range_relation(&Mrs::new(3, 4), &Mrs::new(1, 2), &t),
            RangeRelation::After(())
        )); // a is after b
    }

    pub fn known_bad_mrs() -> Vec<Mrs<i32>> {
        return vec![
            Mrs::new(i32::MIN, 2),
            Mrs::new(1, 2),
            Mrs::new(3, 4),
            Mrs::new(3, i32::MAX),
        ];
    }

    pub fn checked_bad() -> Vec<(i32, i32)> {
        return vec![(i32::MIN, 1), (2, 2), (3, 4), (5, i32::MAX)];
    }
    #[test]
    fn inf_bounds() {
        let t = NumberIncDecCpCmp::defaults();
        let bad = known_bad_mrs();
        let good = checked_bad();
        assert_eq!(first_range_begin_end(&bad, &t).unwrap(), good[0]);
        assert_eq!(next_range_begin_end(&2, &bad, &t).unwrap(), good[1]);
        assert_eq!(next_range_begin_end(&3, &bad, &t).unwrap(), good[2]);
        assert_eq!(next_range_begin_end(&5, &bad, &t).unwrap(), good[3]);
        matches!(next_range_begin_end(&i32::MAX, &bad, &t), None);
    }

    #[test]
    fn growth_tests() {
        let mut x = Mrs::new(1, 6);
        let mut y = Mrs::new(3, 9);
        let f = MrsFactory::new();
        let t = NumberIncDecCpCmp::defaults();

        assert_eq!(
            grow(&x, &y, &t, &f).unwrap().to_tuple(),
            Mrs::new(1, 9).to_tuple()
        );
        assert_eq!(
            grow(&y, &x, &t, &f).unwrap().to_tuple(),
            Mrs::new(1, 9).to_tuple()
        );
        x = Mrs::new(1, 6);
        y = Mrs::new(1, 9);
        assert_eq!(
            grow(&y, &x, &t, &f).unwrap().to_tuple(),
            Mrs::new(1, 9).to_tuple()
        );
        assert_eq!(
            grow(&x, &y, &t, &f).unwrap().to_tuple(),
            Mrs::new(1, 9).to_tuple()
        );
        x = Mrs::new(1, 6);
        y = Mrs::new(2, 5);
        assert_eq!(
            grow(&x, &y, &t, &f).unwrap().to_tuple(),
            Mrs::new(1, 6).to_tuple()
        );
        assert_eq!(
            grow(&y, &x, &t, &f).unwrap().to_tuple(),
            Mrs::new(1, 6).to_tuple()
        );
        assert_eq!(
            grow(&x, &x, &t, &f).unwrap().to_tuple(),
            Mrs::new(1, 6).to_tuple()
        );
    }

    #[test]
    fn auto_range_tests() {
        let f = MrsFactory::new();
        let mut t = NumberIncDecCpCmp::defaults();
        let mut res = auto_range(0, Mrs::new(1, 2), &t, &f);
        let (r, l) = res.unwrap();
        assert_eq!(r.to_tuple(), (1, 2));
        assert_eq!(l.len(), 1);
        assert_eq!(l[0].0, 0);
        assert_eq!(l[0].1.to_tuple_ref(), (&1, &2));
        t.set_min(7);
        res = auto_range(0, Mrs::new(1, 2), &t, &f);
        matches!(res, None);
    }

    #[test]
    fn consolidate_tests() {
        let f = MrsFactory::new();
        let t = NumberIncDecCpCmp::defaults();
        let mut iter = vec![Mrs::new(1, 2)].into_iter();
        let mut last = None;
        let (_, mut next) = consolidate(&mut last, &mut iter, &t, &f, 0);

        let mut res = next.unwrap();
        matches!(last, None);
        assert!(res.is_last());
        let (mut r, mut set) = res.last();
        assert_eq!(r.to_tuple(), (1, 2));
        assert_eq!(set.len(), 1);
        assert_eq!(&set[0].0, &0);
        assert_eq!(set[0].1.to_tuple_ref(), (&1, &2));
        iter = vec![
            // Block A
            Mrs::new(1, 2), // 0
            Mrs::new(2, 3), // 1
            // Block B
            Mrs::new(4, 4), // 2
            // Block C
            Mrs::new(1, 1), // 3
            // Block D
            Mrs::new(5, 6), // 4
            Mrs::new(6, 7), // 6
            // Block E
            Mrs::new(8, 8), // 7
        ]
        .into_iter();

        last = None;
        // Block A results
        let mut offset;
        (offset, next) = consolidate(&mut last, &mut iter, &t, &f, 0);
        assert!(!last.is_none());
        assert!(!next.is_none());
        res = next.unwrap();
        assert!(res.is_before());
        (r, set) = res.before();
        assert_eq!(r.to_tuple(), (1, 3));
        assert_eq!(set.len(), 2);
        assert_eq!(&set[0].0, &0);
        assert_eq!(set[0].1.to_tuple_ref(), (&1, &2));
        assert_eq!(&set[1].0, &1);
        assert_eq!(set[1].1.to_tuple_ref(), (&2, &3));

        // Block B
        (offset, next) = consolidate(&mut last, &mut iter, &t, &f, offset);
        assert!(!last.is_none());
        assert!(!next.is_none());

        res = next.unwrap();
        assert!(res.is_after());
        (r, set) = res.after();
        assert_eq!(r.to_tuple(), (4, 4));
        assert_eq!(set.len(), 1);
        assert_eq!(&set[0].0, &2);
        assert_eq!(set[0].1.to_tuple_ref(), (&4, &4));

        // Block C
        (offset, next) = consolidate(&mut last, &mut iter, &t, &f, offset);
        assert!(!last.is_none());
        assert!(!next.is_none());
        res = next.unwrap();
        assert!(res.is_before());
        (r, set) = res.before();
        assert_eq!(r.to_tuple(), (1, 1));
        assert_eq!(set.len(), 1);
        assert_eq!(set[0].1.to_tuple_ref(), (&1, &1));
        assert_eq!(&set[0].0, &3);

        // Block D
        (offset, next) = consolidate(&mut last, &mut iter, &t, &f, offset);
        assert!(!last.is_none());
        assert!(!next.is_none());
        res = next.unwrap();
        assert!(res.is_before());
        (r, set) = res.before();
        assert_eq!(r.to_tuple(), (5, 7));
        assert_eq!(set.len(), 2);
        assert_eq!(&set[0].0, &4);
        assert_eq!(set[0].1.to_tuple_ref(), (&5, &6));
        assert_eq!(&set[1].0, &5);
        assert_eq!(set[1].1.to_tuple_ref(), (&6, &7));

        (_, next) = consolidate(&mut last, &mut iter, &t, &f, offset);
        assert!(last.is_none());
        assert!(!next.is_none());
        res = next.unwrap();
        assert!(res.is_last());
        (r, set) = res.last();
        assert_eq!(r.to_tuple(), (8, 8));
        assert_eq!(set.len(), 1);
        assert_eq!(&set[0].0, &6);
        assert_eq!(set[0].1.to_tuple_ref(), (&8, &8));
        (_, next) = consolidate(&mut last, &mut iter, &t, &f, offset);
        assert!(last.is_none());
        assert!(next.is_none());
    }
}
