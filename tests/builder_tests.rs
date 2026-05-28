#![cfg(test)]
mod builder_tests {

    use std::ops::Bound;

    use common_range_tools::{
        DefaultValues, GetBeginEnd, Mrs,
        builder::{BlanketIncDecCpCmp, IncDecCpCmp},
    };

    #[test]
    fn inc_dec_behavior() {
        let l = BlanketIncDecCpCmp::new();

        // i32 Increment examples
        assert_eq!(l.inc(&1, &2), Some(3)); // Number went up by 2!
        assert_eq!(l.inc(&1, &0), None); // Number did not go up
        assert_eq!(l.inc(&0, &-2), None); // Number did not go up
        assert_eq!(l.inc(&i32::MAX, &1), None); // Catch overflow

        // i32 Decrement examples
        assert_eq!(l.dec(&1, &2), Some(-1)); // Number went down by 2!
        assert_eq!(l.dec(&0, &0), None); // Number did not go down
        assert_eq!(l.dec(&0, &-2), None); // Number did not go down
        assert_eq!(l.dec(&i32::MIN, &1), None); // Catch undeflow

        // u32 Increment examples
        assert_eq!(l.inc(&1_u32, &2_u32), Some(3)); // Number went up by 2!
        assert_eq!(l.inc(&0_u32, &0), None); // Number did not go up
        assert_eq!(l.inc(&u32::MAX, &1), None); // Catch overflow

        // i32 Decrement examples
        assert_eq!(l.dec(&3_u32, &2), Some(1)); // Number went down by 2!
        assert_eq!(l.dec(&3_u32, &0), None); // Number did not go down
        assert_eq!(l.dec(&u32::MIN, &1), None); // Catch undeflow

        // f32 Increment examples
        assert_eq!(l.inc(&0.2, &0.5), Some(0.7));
        assert_eq!(l.inc(&1.7, &-0.5), None);
        assert_eq!(l.inc(&f32::INFINITY, &0.5), None);
        assert_eq!(l.inc(&f32::INFINITY, &f32::INFINITY), None);
        assert_eq!(l.inc(&1.0, &f32::INFINITY), Some(f32::INFINITY));
        assert_eq!(l.inc(&1.0, &f32::NEG_INFINITY), None);

        // f32 Decrement examples
        assert_eq!(l.dec(&0.5, &0.5), Some(-0.0));
        assert_eq!(l.dec(&1.7, &-0.5), None);
        assert_eq!(l.dec(&f32::INFINITY, &0.5), None);
        assert_eq!(l.dec(&f32::INFINITY, &f32::INFINITY), None);
        assert_eq!(l.dec(&1.0, &f32::INFINITY), Some(f32::NEG_INFINITY));
        assert_eq!(l.dec(&1.0, &f32::NEG_INFINITY), None);
    }

    #[test]
    fn misc_tests() {
        let t = BlanketIncDecCpCmp::new();
        assert_eq!(u8::MAX, t.max());
        assert_eq!(u8::MIN, t.min());
        assert_eq!(f32::MAX, t.max());
        assert_eq!(f32::MIN, t.min());
        assert_eq!(i32::MAX, t.max());
        assert_eq!(i32::MIN, t.min());
        assert_eq!(1_u8, t.cp(&1));
        assert_eq!(1.0_f32, t.cp(&1.0));
        assert_eq!(1, t.cp(&1));
        assert_eq!(1, t.default_step());
        assert_eq!(1, t.default_rebound());
        assert_eq!(1_u8, t.default_step());
        assert_eq!(1_u8, t.default_rebound());
        assert_eq!(1.0, t.default_step());
        assert_eq!(1.0, t.default_rebound());
        assert_eq!(Mrs::new(1, 2).to_tuple(), (1, 2));
    }
    #[test]
    fn inc_dec_compare_all() {
        let l = BlanketIncDecCpCmp::new();

        // positive examples
        assert!(l.lt(&1, &2));
        assert!(l.le(&1, &2));
        assert!(l.eq(&2, &2));
        assert!(l.gt(&4, &3));
        assert!(l.ge(&4, &3));
        assert!(l.ne(&4, &3));

        // negative examples
        assert!(!l.ne(&4, &4));
        assert!(!l.eq(&3, &4));
        assert!(!l.le(&6, &5));
        assert!(!l.ge(&4, &5));
        assert!(!l.lt(&4, &3));
        assert!(!l.gt(&4, &5));

        assert!(l.lt(&1.0, &3.0));
        assert!(l.lt(&0_u8, &2));

        // Contains Examples
        assert!(l.contains(&1, &3, &1));
        assert!(!l.contains(&1, &2, &0));
        assert!(!l.contains(&0, &0, &1));
        assert!(!l.contains(&0, &0, &2));

        // Overlap Examples
        assert!(l.overlap(&1, &2, &0, &1));
        assert!(!l.overlap(&1, &2, &0, &0));
    }

    #[test]
    fn test_rebound_start_and_end() {
        let l = BlanketIncDecCpCmp::new();

        assert_eq!(l.rebound_start(Bound::Excluded(&1), &1), Some(2));
        assert_eq!(l.rebound_start(Bound::Included(&1), &1), Some(1));
        assert_eq!(l.rebound_start(Bound::Unbounded, &1), Some(i32::MIN));
        assert_eq!(l.rebound_start(Bound::Excluded(&i32::MAX), &1), None);

        assert_eq!(l.rebound_end(Bound::Excluded(&1), &1), Some(0));
        assert_eq!(l.rebound_end(Bound::Included(&1), &1), Some(1));
        assert_eq!(l.rebound_end(Bound::Unbounded, &1), Some(i32::MAX));
        assert_eq!(l.rebound_end(Bound::Excluded(&i32::MIN), &1), None);
    }
}
