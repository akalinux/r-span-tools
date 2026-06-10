// Import the Intersector
use common_range_tools::Intersector;

fn main() {
    let mut isec = Intersector::num(
        1, // step
        1, // rebound
        0, // min
        8, // max
    );
    let range = 1..4;
    isec.add_range(&range);
    let range_inclusive = 3..=5;
    isec.add_range(&range_inclusive);
    let min_to_end = ..=7;
    isec.add_range(&min_to_end);

    println!("//! |Start|End|");
    println!("//! |-----|---|");
    for i in isec.into_iter() {
        println!("//! |{:^14}|{:^14}|", i.start(), i.end());
    }
}
