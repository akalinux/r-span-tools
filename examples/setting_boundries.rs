// Import the Intersector
use common_range_tools::Intersector;

fn main() {
    let mut isec = Intersector::num(
        1, // step
        1, // rebound
        0, // min
        8, // max
    );
    // 1 to 3
    let range: std::ops::Range<i32> = 1..4;
    isec.add_range(&range);

    // 3 to 5
    let range_inclusive: std::ops::RangeInclusive<i32> = 3..=5;
    isec.add_range(&range_inclusive);

    // 0 to 7
    let min_to_end: std::ops::RangeToInclusive<i32> = ..=7;
    isec.add_range(&min_to_end);

    // 7 to 8
    let begin_to_max: std::ops::RangeFrom<i32> = 7..;
    isec.add_range(&begin_to_max);

    // Note 7.. and ..7 include our min and max all ready!
    let min_to_max: std::ops::RangeFull = ..;
    isec.add_range(&min_to_max);
    for i in isec.into_iter() {
        println!("  Common Range {:^3}->{:^3}", i.start(), i.end());
    }
    // The output will be:
    //  Common Range  0 -> 0
    //  Common Range  1 -> 2
    //  Common Range  3 -> 3
    //  Common Range  4 -> 5
    //  Common Range  6 -> 6
    //  Common Range  7 -> 7
    //  Common Range  8 -> 8
}
