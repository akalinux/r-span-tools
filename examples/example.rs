use common_range_tools::Intersector;

fn main() {
    // Forwards
    println!("Forwards");
    for r in Intersector::num_from(&[1..5, 0..4, 3..12, 10..23]) {
        println!("Common Range: {}->{}", r.start(), r.end());
    }
    // Output will be
    //  Forwards
    //  Common Range: 0->0
    //  Common Range: 1->3
    //  Common Range: 4->4
    //  Common Range: 5->10
    //  Common Range: 11->11
    //  Common Range: 12->22

    // add a small bumper to the output
    print!("\n\n");
    // Backwards
    println!("Backwards");
    for r in Intersector::num_from(&[1..5, 0..4, 3..12, 10..23]).rev() {
        println!("Common Range: {}->{}", r.start(), r.end());
    }
    // Outout will be
    //  Backwards
    //  Common Range: 12->22
    //  Common Range: 10->11
    //  Common Range: 5->9
    //  Common Range: 3->4
    //  Common Range: 1->2
    //  Common Range: 0->0
}
