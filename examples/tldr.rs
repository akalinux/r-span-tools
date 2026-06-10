// Import the Intersector
use common_range_tools::Intersector;

fn main() {
    let mut isec = Intersector::num_defaults();
    let range = 1..4;
    isec.add_range(&range);
    let range_inclusive = 3..=5;
    isec.add_range(&range_inclusive);
    let min_to_end = ..=7;
    isec.add_range(&min_to_end);
    let begin_to_max = 7..;
    isec.add_range(&begin_to_max);

    // Note 7.. and ..7 include our min and max all ready.. so this is a bit redundant
    // but works non the less.
    let min_to_max = ..;
    isec.add_range(&min_to_max);

    println!("//! |Start|End|");
    println!("//! |-----|---|");
    for i in isec.into_iter() {
        println!("//! |{:^14}|{:^14}|", i.start(), i.end());
    }
}
