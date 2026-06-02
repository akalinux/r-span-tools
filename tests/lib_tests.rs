#![cfg(test)]
mod lip_tests {
    use std::ops::{Bound, RangeBounds, RangeInclusive};

    use common_range_tools::{ConsolidateMrsP, GetBeginEnd, Mrs, MrsP};
    use std::panic::catch_unwind;

    #[test]
    fn test_gets_begin_end() {
        assert_eq!(Mrs::new(0, 1).get_begin(), &0);
        assert_eq!(Mrs::new(0, 1).get_end(), &1);
    }

    #[test]
    fn test_mrsp_panic() {
        match catch_unwind(|| {
            let m = Mrs::new(1, 2);
            let p = MrsP::new(&m);
            p.to_tuple();
        }) {
            Ok(_) => panic!("test failed"),
            Err(_) => (),
        }
    }

    #[test]
    fn test_from() {
        assert_eq!(1..=3, RangeInclusive::from(Mrs::new(1, 3)));
        assert_eq!(1..=3, Mrs::new(1, 3).into());
        assert_eq!((1, 3), Mrs::new(1, 3).into());
    }

    #[test]
    fn test_range_bounds() {
        matches!(Mrs::new(0, 1).start_bound(), Bound::Included(&0));
        matches!(Mrs::new(0, 1).end_bound(), Bound::Included(&1));
        assert!(Mrs::new(0, 1).contains(&1));
        assert!(Mrs::new(0, 1).contains(&0));
        assert!(!Mrs::new(0, 1).contains(&-1));
    }

    #[test]
    fn consolidate_proxy_tests() {
        let mut p = ConsolidateMrsP::new((Mrs::new(0, 1), vec![(0, Mrs::new(0, 1))]));
        matches!(p.start_bound(), Bound::Included(&0));
        matches!(p.end_bound(), Bound::Included(&1));
        assert_eq!(p.to_tuple_ref(), (&0, &1));

        p = ConsolidateMrsP::new(p.as_src());
        p.src(); // coverage...
        assert_eq!(p.to_tuple(), (0, 1));
    }
}
