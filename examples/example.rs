use common_range_tools::Intersector;
use std::ops::RangeInclusive;

fn main() {
    // RangeInclusive used to make this more readable.
    let src = [1..=4, 0..=3, 3..=11, 10..=22];

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
    print!("\n");

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

    // add a small bumper to the output
    print!("\n");

    // This creates an iterator for both the intersection and a ref to the source range.
    let mut iter = Intersector::num_from_ol(&src);
    println!("Foward with source Ranges");
    overlaps_info(&mut iter);
    // Output will be:
    //  Foward with source Ranges
    //    Common Range:  0->0  Count: 1 Ranges: 0->3
    //    Common Range:  1->2  Count: 2 Ranges: 1->4, 0->3
    //    Common Range:  3->3  Count: 3 Ranges: 1->4, 0->3, 3->11
    //    Common Range:  4->4  Count: 2 Ranges: 1->4, 3->11
    //    Common Range:  5->9  Count: 1 Ranges: 3->11
    //    Common Range: 10->11 Count: 2 Ranges: 3->11, 10->22
    //    Common Range: 12->22 Count: 1 Ranges: 10->22

    // add a small bumper to the output
    print!("\n");

    // we can just reset the iterator
    iter.reset();
    println!("Reverse, with source Ranges");
    // now we set it to reverse
    overlaps_info(&mut &mut iter.rev());
    // Output will be:
    //  Reverse, with source Ranges
    //    Common Range: 12->22 Count: 1 Ranges: 10->22
    //    Common Range: 10->11 Count: 2 Ranges: 3->11, 10->22
    //    Common Range:  5->9  Count: 1 Ranges: 3->11
    //    Common Range:  4->4  Count: 2 Ranges: 1->4, 3->11
    //    Common Range:  3->3  Count: 3 Ranges: 1->4, 0->3, 3->11
    //    Common Range:  1->2  Count: 2 Ranges: 1->4, 0->3
    //    Common Range:  0->0  Count: 1 Ranges: 0->3
}

// Format the output for our  Intersections with the source Ranges!
fn overlaps_info<'a, I: Iterator<Item = &'a RangeInclusive<i32>>>(
    iter: &mut impl Iterator<Item = (RangeInclusive<i32>, I)>,
) {
    for (r, isec) in iter {
        print!(
            "  Common Range: {:^6}",
            format!("{}->{}", r.start(), r.end())
        );
        let mut txt = Vec::new();
        // grab all of our overlapping ranges!
        for src_range in isec {
            txt.push(format!("{}->{}", src_range.start(), src_range.end()));
        }
        print!(" Count: {}", txt.len());
        println!(" Ranges: {}", txt.join(", "));
    }
}
