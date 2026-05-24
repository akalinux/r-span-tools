#![cfg(test)]
use std::ops::RangeInclusive;

use crate::{
    Accumulate, BlanketIncDecCpCmp, BoxedOverlapIter, DefaultValues, Intersector, Mrs, OverlapIter,
    OwnedOverlapIter,
};

fn checkset() -> [(i32, i32); 8] {
    return [
        (0, 2),
        (3, 3),
        (4, 5),
        (6, 6),
        (8, 11),
        (13, 15),
        (16, 19),
        (20, 22),
    ];
}
fn mrs_set() -> Vec<Mrs<i32>> {
    return vec![
        Mrs::new(4, 5),
        Mrs::new(4, 6),
        Mrs::new(0, 3),
        Mrs::new(1, 2),
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

    let mut src = mrs_set();
    let t = BlanketIncDecCpCmp::new();

    let iter = OverlapIter::new(src.as_mut_slice(), &1, &t);
    for (i, res) in iter.enumerate() {
        assert_eq!(res, checkset[i])
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
fn boxed_iter() {
    let src = mrs_set();
    let t = BlanketIncDecCpCmp::new();
    let step: i32 = <BlanketIncDecCpCmp as DefaultValues<i32, i32>>::default_step(&t);
    let checkset = checkset();

    for _ in 1..3 {
        let iter = BoxedOverlapIter::new(&src, &step, &t);
        for (i, res) in iter.enumerate() {
            assert_eq!(res, checkset[i])
        }
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
