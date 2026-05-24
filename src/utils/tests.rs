#![cfg(test)]

use crate::{
    Mrs, builder::BlanketIncDecCpCmp, first_range_begin_end, last_range_begin_end,
    next_range_begin_end, range_bounds_to_values,
};

#[test]
fn test_first_range() {
    let t = BlanketIncDecCpCmp::new();

    // Empty set test
    assert_eq!(
        first_range_begin_end::<i32, i32, BlanketIncDecCpCmp, Mrs<i32>>(&[], &t),
        None
    );

    assert_eq!(
        first_range_begin_end::<i32, i32, BlanketIncDecCpCmp, Mrs<i32>>(&[Mrs::new(0, -1)], &t),
        None
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
            &t
        ),
        Some((0, 0))
    );
}

#[test]
fn test_last_range() {
    let t = BlanketIncDecCpCmp::new();

    // Empty set test
    assert_eq!(
        last_range_begin_end::<i32, i32, BlanketIncDecCpCmp, Mrs<i32>>(&[], &t),
        None
    );

    assert_eq!(
        last_range_begin_end::<i32, i32, BlanketIncDecCpCmp, Mrs<i32>>(&[Mrs::new(0, -1)], &t),
        None
    );

    assert_eq!(
        last_range_begin_end(
            &[
                Mrs::new(0, 1),
                Mrs::new(4, 7),
                Mrs::new(5, 5),
                Mrs::new(0, 0),
                Mrs::new(2, -1), // this should be invalid
            ],
            &t
        ),
        Some((5, 7))
    );
}

#[test]
fn test_next_span() {
    let mut checkset = vec![
        (3, 3),
        (4, 5),
        (6, 6),
        (8, 11),
        (13, 15),
        (16, 19),
        (20, 22),
    ];

    let mut check = vec![
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
    let mut point = 23;

    let t = BlanketIncDecCpCmp::new();
    assert_eq!(next_range_begin_end(&point, &check, &t), None,);

    checkset = vec![(8, 11), (13, 15), (16, 19), (20, 22)];

    // validate smallest default gap in reversal of
    point = 7;
    check = vec![
        // reversing  the order of the gap for coverage
        Mrs::new(15, 19),
        Mrs::new(13, 22),
        // order should never mater
        Mrs::new(8, 11),
    ];
    for (a, b) in checkset {
        assert_eq!(
            next_range_begin_end(&point, &check, &t),
            Some((a.clone(), b.clone()))
        );
        point = b + 1;
    }
    assert_eq!(next_range_begin_end(&23, &check, &t), None,);
}

#[test]
fn range_conversion() {
    let t = BlanketIncDecCpCmp::new();

    assert_eq!(range_bounds_to_values(&(1..=2), &1, &t), Some((1, 2)));
}
