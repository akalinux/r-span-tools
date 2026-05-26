#![cfg(test)]

use crate::{
    Mrs, builder::BlanketIncDecCpCmp, first_range_begin_end, last_range_begin_end,
    next_range_begin_end, next_smallest_range, previous_range_begin_end, previous_smallest_range,
    range_bounds_to_values,
};

#[test]
fn test_first_range() {
    let t = BlanketIncDecCpCmp::new();

    // Empty set test
    assert_eq!(
        first_range_begin_end::<i32, i32, BlanketIncDecCpCmp, Mrs<i32>>(&[], &1, &t),
        None
    );

    assert_eq!(
        first_range_begin_end::<i32, i32, BlanketIncDecCpCmp, Mrs<i32>>(&[Mrs::new(0, -1)], &1, &t),
        None
    );

    assert_eq!(
        first_range_begin_end(&[Mrs::new(1, 1), Mrs::new(6, 7),], &1, &t),
        Some((1, 1))
    );

    assert_eq!(
        first_range_begin_end(&[Mrs::new(1, 1)], &1, &t),
        Some((1, 1))
    );

    assert_eq!(
        first_range_begin_end(&[Mrs::new(5, 7), Mrs::new(4, 7),], &1, &t),
        Some((4, 4))
    );

    assert_eq!(
        first_range_begin_end(&[Mrs::new(5, 7), Mrs::new(6, 7),], &1, &t),
        Some((5, 5))
    );

    assert_eq!(
        first_range_begin_end(
            &[
                Mrs::new(5, 7),
                Mrs::new(0, 2),
                Mrs::new(0, 1),
                Mrs::new(0, 0),
                Mrs::new(2, -1), // this should be invalid
            ],
            &1,
            &t
        ),
        Some((0, 0))
    );

    assert_eq!(first_range_begin_end(&mrs_set_a(), &1, &t), Some((0, 0)));
}

#[test]
fn last_range_test() {
    let t = BlanketIncDecCpCmp::new();

    // Empty set test
    assert_eq!(
        last_range_begin_end::<i32, i32, BlanketIncDecCpCmp, Mrs<i32>>(&[], &1, &t),
        None
    );

    assert_eq!(
        last_range_begin_end::<i32, i32, BlanketIncDecCpCmp, Mrs<i32>>(&[Mrs::new(0, -1)], &1, &t),
        None
    );

    assert_eq!(
        last_range_begin_end(
            &[
                Mrs::new(5, 7),
                Mrs::new(0, 2),
                Mrs::new(0, 1),
                Mrs::new(0, 0),
                Mrs::new(2, -1), // this should be invalid
            ],
            &1,
            &t
        ),
        Some((5, 7))
    );

    assert_eq!(
        last_range_begin_end(&[Mrs::new(5, 7), Mrs::new(4, 7),], &1, &t),
        Some((6, 7))
    );

    assert_eq!(
        last_range_begin_end(&[Mrs::new(4, 11), Mrs::new(4, 7),], &1, &t),
        Some((8, 11))
    );
    assert_eq!(
        last_range_begin_end(&[Mrs::new(4, 11)], &1, &t),
        Some((4, 11))
    );
    assert_eq!(last_range_begin_end(&mrs_set_a(), &1, &t), Some((20, 22)));
}

fn checkset_a() -> Vec<(i32, i32)> {
    return vec![
        (3, 3),
        (4, 4),
        (5, 5),
        (6, 6),
        (8, 11),
        (13, 14),
        (15, 15),
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
        (16, 19),
        (15, 15),
        (13, 14),
        (8, 11),
        (6, 6),
        (5, 5),
        (4, 4),
        (3, 3),
        (2, 2),
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

fn checkset_b() -> Vec<(i32, i32)> {
    return vec![(8, 11), (13, 14), (15, 15), (16, 19), (20, 22)];
}

fn mrs_set_c() -> Vec<Mrs<i32>> {
    return vec![
        // reversing  the order of the gap for coverage
        Mrs::new(15, 15),
        Mrs::new(13, 13),
        // order should never mater
        Mrs::new(8, 8),
    ];
}

fn checkset_c() -> Vec<(i32, i32)> {
    return vec![(8, 8), (13, 13), (15, 15)];
}

fn mrs_set_d() -> Vec<Mrs<i32>> {
    return vec![Mrs::new(0, 20), Mrs::new(13, 15), Mrs::new(13, 13)];
}

fn checkset_d() -> Vec<(i32, i32)> {
    return vec![(0, 12), (13, 13), (14, 15), (16, 20)];
}

#[test]
fn next_range_begin_end_tests() {
    let t = BlanketIncDecCpCmp::new();
    let mut checkset = checkset_a();

    let mut check = mrs_set_a();
    let mut point = 3;
    for (a, b) in checkset {
        assert_eq!(
            next_range_begin_end(&point, &check, &1, &t),
            Some((a.clone(), b.clone()))
        );
        point = b + 1;
    }

    assert_eq!(next_range_begin_end(&23, &check, &1, &t), None,);

    checkset = checkset_b();

    // validate smallest default gap in reversal of
    point = 7;
    check = mrs_set_b();
    for (a, b) in checkset {
        assert_eq!(
            next_range_begin_end(&point, &check, &1, &t),
            Some((a.clone(), b.clone()))
        );
        point = b + 1;
    }
    assert_eq!(next_range_begin_end(&23, &check, &1, &t), None,);

    // validate single value set with gaps
    point = 8;
    check = mrs_set_c();
    checkset = checkset_c();
    for (a, b) in checkset {
        assert_eq!(
            next_range_begin_end(&point, &check, &1, &t),
            Some((a.clone(), b.clone()))
        );
        point = b + 1;
    }
    assert_eq!(next_range_begin_end(&23, &check, &1, &t), None,);

    // validate correct overlaps
    point = 0;
    check = mrs_set_d();
    checkset = checkset_d();
    for (a, b) in checkset {
        assert_eq!(
            next_range_begin_end(&point, &check, &1, &t),
            Some((a.clone(), b.clone()))
        );
        point = b + 1;
    }
    assert_eq!(next_range_begin_end(&point, &check, &1, &t), None,);
}

#[test]
fn range_conversion() {
    let t = BlanketIncDecCpCmp::new();

    assert_eq!(range_bounds_to_values(&(1..=2), &1, &t), Some((1, 2)));
    assert_eq!(
        range_bounds_to_values(&(1.0..f32::INFINITY), &1.0, &t),
        None
    );
}

#[test]
fn previous_smallest_range_test() {
    let t = BlanketIncDecCpCmp::new();
    let src = mrs_set_b();

    let mut valid = Vec::new();
    for s in src.as_slice() {
        valid.push(s);
    }
    let (begin, end, _, _) = previous_smallest_range(&0, &22, &valid, &t);

    assert_eq!((begin, end), (19, 22))
}

#[test]
fn previous_range_begin_end_tests() {
    let t = BlanketIncDecCpCmp::new();
    let checked = checkset_a_reversed();
    let src = mrs_set_a();

    let mut end = 19;

    for (a, z) in checked {
        println!("Checking: {}->{}", a, z);
        let res = previous_range_begin_end(&end, &src, &1, &t);
        assert_eq!(res, Some((a, z)));
        end = a - 1;
    }
}

#[test]
fn next_smallest_range_test() {
    let t = BlanketIncDecCpCmp::new();
    let mut src = mrs_set_a();

    let mut valid = Vec::new();
    for s in src.as_slice() {
        valid.push(s);
    }
    let (mut begin, mut end, _, _) = next_smallest_range(&0, &22, &valid, &t);
    assert_eq!((begin, end), (0, 1));

    (begin, end, _, _) = next_smallest_range(&0, &1, &valid, &t);
    assert_eq!((begin, end), (0, 1));

    src = vec![Mrs::new(5, 7), Mrs::new(4, 7)];
    let mut valid = Vec::new();
    for s in src.as_slice() {
        valid.push(s);
    }

    (begin, end, _, _) = next_smallest_range(&4, &7, &valid, &t);
    assert_eq!((begin, end), (4, 5));
}
