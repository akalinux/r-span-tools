use common_range_tools::Intersector;

fn main() {
    // RangeInclusive used to make this more readable.
    let src = [1..=4, 0..=3, 3..=11, 10..=22];
    // Forwards
    println!("Forwards");
    for r in Intersector::num_from(&src) {
        println!("Common Range: {}->{}", r.start(), r.end());
    }
    // Output will be
    //  Forwards
    //  Common Range: 0->0
    //  Common Range: 1->2
    //  Common Range: 3->3
    //  Common Range: 4->4
    //  Common Range: 5->9
    //  Common Range: 10->11
    //  Common Range: 12->22

    // add a small bumper to the output
    print!("\n\n");
    // Backwards
    println!("Backwards");
    for r in Intersector::num_from(&src).rev() {
        println!("Common Range: {}->{}", r.start(), r.end());
    }
    // Outout will be
    //  Backwards
    //  Common Range: 12->22
    //  Common Range: 10->11
    //  Common Range: 5->9
    //  Common Range: 4->4
    //  Common Range: 3->3
    //  Common Range: 1->2
    //  Common Range: 0->0
}
