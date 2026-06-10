// Import the Intersector
use common_range_tools::Intersector;

fn main() {
    let mut isec = Intersector::num(
        1, // step
        1, // rebound
        0, // min
        8, // max
    );
    let range: std::ops::Range<i32> = 1..4;
    isec.add_range(&range);

    let range_inclusive: std::ops::RangeInclusive<i32> = 3..=5;
    isec.add_range(&range_inclusive);

    let min_to_end: std::ops::RangeToInclusive<i32> = ..=7;
    isec.add_range(&min_to_end);

    let min_max: std::ops::RangeFull = ..;
    isec.add_range(&min_max);

    for i in isec.into_iter() {
        println!("  Common Range {:^3}->{:^3}", i.start(), i.end());
    }
    // The output will be:
    //  Common Range  0 -> 0
    //  Common Range  1 -> 3
    //  Common Range  4 -> 5
    //  Common Range  6 -> 7
    //  Common Range  8 -> 8
}
